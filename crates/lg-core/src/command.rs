use crate::document::{Asset, AssetSource, BlendMode, Document, Layer, LayerKind, Rgba};
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Command {
    DocumentUpdate {
        #[serde(default)]
        width: Option<u32>,
        #[serde(default)]
        height: Option<u32>,
        #[serde(default)]
        dpi: Option<f32>,
    },
    LayerAdd {
        layer: Layer,
        #[serde(default)]
        parent_id: Option<String>,
        #[serde(default)]
        index: Option<usize>,
    },
    LayerUpdate {
        id: String,
        set: LayerPatch,
    },
    LayerRemove {
        id: String,
    },
    LayerMove {
        id: String,
        #[serde(default)]
        parent_id: Option<String>,
        #[serde(default)]
        to: Option<usize>,
        #[serde(default)]
        above: Option<String>,
        #[serde(default)]
        below: Option<String>,
    },
    AssetAdd {
        id: String,
        media_type: String,
        bytes_base64: String,
        #[serde(default)]
        original_name: Option<String>,
    },
    AssetRemove {
        id: String,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayerPatch {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub visible: Option<bool>,
    #[serde(default)]
    pub opacity: Option<f32>,
    #[serde(default)]
    pub blend_mode: Option<BlendMode>,
    #[serde(default)]
    pub x: Option<f32>,
    #[serde(default)]
    pub y: Option<f32>,
    #[serde(default)]
    pub scale_x: Option<f32>,
    #[serde(default)]
    pub scale_y: Option<f32>,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub font_asset_id: Option<String>,
    #[serde(default)]
    pub font_size: Option<f32>,
    #[serde(default)]
    pub color: Option<Rgba>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
    pub revision: u64,
    pub applied: usize,
    pub changed_layers: Vec<String>,
    pub changed_assets: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("command {index}: {message}")]
    Invalid { index: usize, message: String },
    #[error("transaction validation failed: {0}")]
    Validation(String),
}

pub fn execute_commands(doc: &mut Document, commands: &[Command]) -> Result<CommandResult, CommandError> {
    let mut next = doc.clone();
    let mut changed_layers = Vec::new();
    let mut changed_assets = Vec::new();
    for (index, command) in commands.iter().enumerate() {
        apply(&mut next, command, &mut changed_layers, &mut changed_assets)
            .map_err(|message| CommandError::Invalid { index, message })?;
    }
    let diagnostics = next.validate();
    if let Some(error) = diagnostics.first() {
        return Err(CommandError::Validation(format!(
            "{} at {}: {}",
            error.code, error.path, error.message
        )));
    }
    if !commands.is_empty() {
        next.manifest.revision = doc.manifest.revision.saturating_add(1);
    }
    *doc = next;
    changed_layers.sort();
    changed_layers.dedup();
    changed_assets.sort();
    changed_assets.dedup();
    Ok(CommandResult {
        revision: doc.manifest.revision,
        applied: commands.len(),
        changed_layers,
        changed_assets,
    })
}

fn apply(
    doc: &mut Document,
    command: &Command,
    changed_layers: &mut Vec<String>,
    changed_assets: &mut Vec<String>,
) -> Result<(), String> {
    match command {
        Command::DocumentUpdate { width, height, dpi } => {
            if let Some(width) = width {
                doc.manifest.canvas.width = *width;
            }
            if let Some(height) = height {
                doc.manifest.canvas.height = *height;
            }
            if let Some(dpi) = dpi {
                doc.manifest.canvas.dpi = *dpi;
            }
        }
        Command::LayerAdd {
            layer,
            parent_id,
            index,
        } => {
            if doc.find_layer(&layer.common.id).is_some() {
                return Err(format!("layer '{}' already exists", layer.common.id));
            }
            let id = layer.common.id.clone();
            let target = layer_list_mut(doc, parent_id.as_deref())?;
            let index = index.unwrap_or(target.len());
            if index > target.len() {
                return Err(format!("layer index {index} is outside 0..={}", target.len()));
            }
            target.insert(index, layer.clone());
            changed_layers.push(id);
        }
        Command::LayerUpdate { id, set } => {
            let layer = doc
                .find_layer_mut(id)
                .ok_or_else(|| format!("layer '{id}' does not exist"))?;
            apply_patch(layer, set)?;
            changed_layers.push(id.clone());
        }
        Command::LayerRemove { id } => {
            remove_layer(&mut doc.manifest.layers, id).ok_or_else(|| format!("layer '{id}' does not exist"))?;
            changed_layers.push(id.clone());
        }
        Command::LayerMove {
            id,
            parent_id,
            to,
            above,
            below,
        } => {
            let layer =
                remove_layer(&mut doc.manifest.layers, id).ok_or_else(|| format!("layer '{id}' does not exist"))?;
            if parent_id.as_deref() == Some(id) {
                return Err("a layer cannot be its own parent".to_owned());
            }
            let target = layer_list_mut(doc, parent_id.as_deref())?;
            let destination = if let Some(to) = to {
                *to
            } else if let Some(above) = above {
                target
                    .iter()
                    .position(|layer| &layer.common.id == above)
                    .map(|index| index + 1)
                    .ok_or_else(|| format!("reference layer '{above}' is not in destination group"))?
            } else if let Some(below) = below {
                target
                    .iter()
                    .position(|layer| &layer.common.id == below)
                    .ok_or_else(|| format!("reference layer '{below}' is not in destination group"))?
            } else {
                target.len()
            };
            if destination > target.len() {
                return Err(format!(
                    "destination index {destination} is outside 0..={}",
                    target.len()
                ));
            }
            target.insert(destination, layer);
            changed_layers.push(id.clone());
        }
        Command::AssetAdd {
            id,
            media_type,
            bytes_base64,
            original_name,
        } => {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(bytes_base64)
                .map_err(|error| format!("invalid base64 asset bytes: {error}"))?;
            let sha = format!("{:x}", Sha256::digest(&bytes));
            let asset = Asset {
                id: id.clone(),
                media_type: media_type.clone(),
                byte_length: bytes.len() as u64,
                sha256: sha.clone(),
                source: AssetSource::Embedded {
                    path: format!("assets/{sha}"),
                },
                original_name: original_name.clone(),
            };
            doc.manifest.assets.insert(id.clone(), asset);
            doc.asset_bytes.insert(id.clone(), bytes);
            changed_assets.push(id.clone());
        }
        Command::AssetRemove { id } => {
            if doc.manifest.assets.remove(id).is_none() {
                return Err(format!("asset '{id}' does not exist"));
            }
            doc.asset_bytes.remove(id);
            changed_assets.push(id.clone());
        }
    }
    Ok(())
}

fn apply_patch(layer: &mut Layer, set: &LayerPatch) -> Result<(), String> {
    if let Some(name) = &set.name {
        layer.common.name = name.clone();
    }
    if let Some(visible) = set.visible {
        layer.common.visible = visible;
    }
    if let Some(opacity) = set.opacity {
        layer.common.opacity = opacity;
    }
    if let Some(blend_mode) = set.blend_mode {
        layer.common.blend_mode = blend_mode;
    }
    if let Some(x) = set.x {
        layer.common.transform.x = x;
    }
    if let Some(y) = set.y {
        layer.common.transform.y = y;
    }
    if let Some(scale_x) = set.scale_x {
        layer.common.transform.scale_x = scale_x;
    }
    if let Some(scale_y) = set.scale_y {
        layer.common.transform.scale_y = scale_y;
    }
    match &mut layer.kind {
        LayerKind::Image { asset_id } => {
            reject(
                set.text.is_some()
                    || set.font_asset_id.is_some()
                    || set.font_size.is_some()
                    || set.color.is_some()
                    || set.width.is_some()
                    || set.height.is_some(),
                "property is not valid for image layer",
            )?;
            if let Some(value) = &set.asset_id {
                *asset_id = value.clone();
            }
        }
        LayerKind::Fill { width, height, color } => {
            reject(
                set.asset_id.is_some() || set.text.is_some() || set.font_asset_id.is_some() || set.font_size.is_some(),
                "property is not valid for fill layer",
            )?;
            if let Some(value) = set.width {
                *width = value;
            }
            if let Some(value) = set.height {
                *height = value;
            }
            if let Some(value) = set.color {
                *color = value;
            }
        }
        LayerKind::Text {
            text,
            font_asset_id,
            font_size,
            color,
        } => {
            reject(
                set.asset_id.is_some() || set.width.is_some() || set.height.is_some(),
                "property is not valid for text layer",
            )?;
            if let Some(value) = &set.text {
                *text = value.clone();
            }
            if let Some(value) = &set.font_asset_id {
                *font_asset_id = value.clone();
            }
            if let Some(value) = set.font_size {
                *font_size = value;
            }
            if let Some(value) = set.color {
                *color = value;
            }
        }
        LayerKind::Group { .. } => {
            reject(
                set.asset_id.is_some()
                    || set.text.is_some()
                    || set.font_asset_id.is_some()
                    || set.font_size.is_some()
                    || set.color.is_some()
                    || set.width.is_some()
                    || set.height.is_some(),
                "content property is not valid for group layer",
            )?;
        }
    }
    Ok(())
}

fn reject(condition: bool, message: &str) -> Result<(), String> {
    if condition { Err(message.to_owned()) } else { Ok(()) }
}

fn layer_list_mut<'a>(doc: &'a mut Document, parent_id: Option<&str>) -> Result<&'a mut Vec<Layer>, String> {
    match parent_id {
        None => Ok(&mut doc.manifest.layers),
        Some(id) => match &mut doc
            .find_layer_mut(id)
            .ok_or_else(|| format!("parent layer '{id}' does not exist"))?
            .kind
        {
            LayerKind::Group { children } => Ok(children),
            _ => Err(format!("parent layer '{id}' is not a group")),
        },
    }
}

fn remove_layer(layers: &mut Vec<Layer>, id: &str) -> Option<Layer> {
    if let Some(index) = layers.iter().position(|layer| layer.common.id == id) {
        return Some(layers.remove(index));
    }
    for layer in layers {
        if let LayerKind::Group { children } = &mut layer.kind
            && let Some(found) = remove_layer(children, id)
        {
            return Some(found);
        }
    }
    None
}
