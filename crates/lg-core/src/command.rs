use crate::document::{Asset, AssetSource, BlendMode, Document, Layer, LayerKind, Rgba};
use base64::Engine;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "op", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Command {
    DocumentUpdate {
        #[serde(default)]
        width: Option<u32>,
        #[serde(default)]
        height: Option<u32>,
        #[serde(default)]
        dpi: Option<f32>,
        #[serde(default)]
        color: Option<Rgba>,
    },
    LayerAdd {
        layer: Layer,
        #[serde(default, alias = "parent_id")]
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
        #[serde(default, alias = "parent_id")]
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
        #[serde(alias = "media_type")]
        media_type: String,
        #[serde(alias = "bytes_base64")]
        bytes_base64: String,
        #[serde(default, alias = "original_name")]
        original_name: Option<String>,
        #[serde(default)]
        author: Option<serde_json::Value>,
    },
    AssetLink {
        id: String,
        #[serde(alias = "media_type")]
        media_type: String,
        reference: String,
        #[serde(alias = "byte_length")]
        byte_length: u64,
        sha256: String,
        #[serde(default, alias = "original_name")]
        original_name: Option<String>,
        #[serde(default)]
        author: Option<serde_json::Value>,
    },
    AssetRelink {
        id: String,
        reference: String,
        #[serde(alias = "byte_length")]
        byte_length: u64,
        sha256: String,
    },
    AssetRemove {
        id: String,
    },
    ExtensionSet {
        namespace: String,
        value: serde_json::Value,
    },
    ExtensionRemove {
        namespace: String,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
    pub from_revision: u64,
    pub revision: u64,
    pub applied: usize,
    pub changed_layers: Vec<String>,
    pub changed_assets: Vec<String>,
    pub changes: Vec<Invalidation>,
    pub warnings: Vec<CommandDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandDiagnostic {
    pub severity: crate::ValidationSeverity,
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ChangeImpact {
    Metadata,
    LocalPixels,
    Composite,
    Asset,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Invalidation {
    pub impact: ChangeImpact,
    pub reason: String,
    pub full_render: bool,
    pub layer_ids: Vec<String>,
    pub asset_ids: Vec<String>,
    pub regions: Vec<Rect>,
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("command {index}: {message}")]
    Invalid { index: usize, message: String },
    #[error("transaction validation failed: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandFailure {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_index: Option<usize>,
}

impl CommandError {
    pub fn failure(&self) -> CommandFailure {
        match self {
            Self::Invalid { index, message } => CommandFailure {
                code: "command.invalid".to_owned(),
                message: message.clone(),
                command_index: Some(*index),
            },
            Self::Validation(message) => CommandFailure {
                code: "transaction.validation".to_owned(),
                message: message.clone(),
                command_index: None,
            },
        }
    }
}

pub fn execute_commands(doc: &mut Document, commands: &[Command]) -> Result<CommandResult, CommandError> {
    if commands.len() > 10_000 {
        return Err(CommandError::Validation(
            "command batch exceeds the supported resource limit".to_owned(),
        ));
    }
    let from_revision = doc.manifest.revision;
    let mut next = doc.clone();
    let mut changed_layers = Vec::new();
    let mut changed_assets = Vec::new();
    let mut changes = Vec::new();
    for (index, command) in commands.iter().enumerate() {
        let mut invalidation = classify_change(&next, command);
        apply(&mut next, command, &mut changed_layers, &mut changed_assets)
            .map_err(|message| CommandError::Invalid { index, message })?;
        if matches!(command, Command::LayerAdd { layer, .. } if layer.common.id.is_empty())
            && let Some(generated) = changed_layers.last()
        {
            invalidation.layer_ids = vec![generated.clone()];
            if let Some(bounds) = rough_layer_bounds(&next, generated) {
                invalidation.regions.push(bounds);
            }
        }
        add_after_region(&next, command, &mut invalidation);
        changes.push(invalidation);
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
        from_revision,
        revision: doc.manifest.revision,
        applied: commands.len(),
        changed_layers,
        changed_assets,
        changes,
        warnings: Vec::new(),
    })
}

fn classify_change(doc: &Document, command: &Command) -> Invalidation {
    let mut change = Invalidation {
        impact: ChangeImpact::Composite,
        reason: "compositionChanged".to_owned(),
        full_render: true,
        layer_ids: Vec::new(),
        asset_ids: Vec::new(),
        regions: Vec::new(),
    };
    match command {
        Command::DocumentUpdate { .. } => {
            change.impact = ChangeImpact::Global;
            change.reason = "canvasChanged".to_owned();
        }
        Command::LayerAdd { layer, .. } => {
            change.impact = ChangeImpact::LocalPixels;
            change.reason = "layerAdded".to_owned();
            change.full_render = false;
            change.layer_ids.push(layer.common.id.clone());
        }
        Command::LayerUpdate { id, set } => {
            change.layer_ids.push(id.clone());
            if let Some(bounds) = rough_layer_bounds(doc, id) {
                change.regions.push(bounds);
            }
            let metadata_only = set.name.is_some()
                && set.visible.is_none()
                && set.opacity.is_none()
                && set.blend_mode.is_none()
                && set.x.is_none()
                && set.y.is_none()
                && set.scale_x.is_none()
                && set.scale_y.is_none()
                && set.asset_id.is_none()
                && set.text.is_none()
                && set.font_asset_id.is_none()
                && set.font_size.is_none()
                && set.color.is_none()
                && set.width.is_none()
                && set.height.is_none();
            if metadata_only {
                change.impact = ChangeImpact::Metadata;
                change.reason = "layerMetadataChanged".to_owned();
                change.full_render = false;
                change.regions.clear();
            } else if set.x.is_some() || set.y.is_some() || set.scale_x.is_some() || set.scale_y.is_some() {
                change.impact = ChangeImpact::LocalPixels;
                change.reason = "layerTransformChanged".to_owned();
                change.full_render = false;
            } else if set.asset_id.is_some()
                || set.text.is_some()
                || set.font_asset_id.is_some()
                || set.font_size.is_some()
                || set.color.is_some()
                || set.width.is_some()
                || set.height.is_some()
            {
                change.impact = ChangeImpact::LocalPixels;
                change.reason = "layerSourceChanged".to_owned();
                change.full_render = false;
            } else {
                change.reason = "layerCompositingChanged".to_owned();
            }
        }
        Command::LayerRemove { id } => {
            change.reason = "layerRemoved".to_owned();
            change.layer_ids.push(id.clone());
            if let Some(bounds) = rough_layer_bounds(doc, id) {
                change.regions.push(bounds);
            }
        }
        Command::LayerMove { id, .. } => {
            change.reason = "layerStackChanged".to_owned();
            change.layer_ids.push(id.clone());
        }
        Command::AssetAdd { id, .. }
        | Command::AssetLink { id, .. }
        | Command::AssetRelink { id, .. }
        | Command::AssetRemove { id } => {
            change.impact = ChangeImpact::Asset;
            change.reason = "assetChanged".to_owned();
            change.asset_ids.push(id.clone());
            change.layer_ids = layers_referencing_asset(&doc.manifest.layers, id);
        }
        Command::ExtensionSet { .. } | Command::ExtensionRemove { .. } => {
            change.impact = ChangeImpact::Metadata;
            change.reason = "extensionMetadataChanged".to_owned();
            change.full_render = false;
        }
    }
    change
}

fn add_after_region(doc: &Document, command: &Command, change: &mut Invalidation) {
    let id = match command {
        Command::LayerAdd { layer, .. } => Some(layer.common.id.as_str()),
        Command::LayerUpdate { id, .. } => Some(id.as_str()),
        _ => None,
    };
    if change.impact != ChangeImpact::Metadata
        && let Some(id) = id
        && let Some(bounds) = rough_layer_bounds(doc, id)
        && !change.regions.contains(&bounds)
    {
        change.regions.push(bounds);
    }
}

fn rough_layer_bounds(doc: &Document, id: &str) -> Option<Rect> {
    let layer = doc.find_layer(id)?;
    rough_bounds_for_layer(doc, layer)
}

fn rough_bounds_for_layer(doc: &Document, layer: &Layer) -> Option<Rect> {
    let (width, height) = match &layer.kind {
        LayerKind::Fill { width, height, .. } => (*width, *height),
        LayerKind::Image { asset_id } => doc
            .asset_bytes
            .get(asset_id)
            .and_then(|bytes| crate::document::image_dimensions_limited(bytes).ok())
            .unwrap_or((doc.manifest.canvas.width, doc.manifest.canvas.height)),
        LayerKind::Text { text, font_size, .. } => (
            (text.lines().map(str::len).max().unwrap_or(0) as f32 * font_size * 0.65).ceil() as u32,
            (text.lines().count().max(1) as f32 * font_size * 1.3).ceil() as u32,
        ),
        LayerKind::Group { children } => {
            let mut union: Option<Rect> = None;
            for child in children {
                if let Some(bounds) = rough_bounds_for_layer(doc, child) {
                    union = Some(match union {
                        Some(current) => union_rect(&current, &bounds),
                        None => bounds,
                    });
                }
            }
            let bounds = union.unwrap_or(Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            });
            (bounds.width, bounds.height)
        }
    };
    Some(Rect {
        x: layer.common.transform.x.floor() as i32,
        y: layer.common.transform.y.floor() as i32,
        width: (width as f32 * layer.common.transform.scale_x.abs()).ceil() as u32,
        height: (height as f32 * layer.common.transform.scale_y.abs()).ceil() as u32,
    })
}

fn union_rect(a: &Rect, b: &Rect) -> Rect {
    let left = a.x.min(b.x);
    let top = a.y.min(b.y);
    let right = (a.x + a.width as i32).max(b.x + b.width as i32);
    let bottom = (a.y + a.height as i32).max(b.y + b.height as i32);
    Rect {
        x: left,
        y: top,
        width: right.saturating_sub(left) as u32,
        height: bottom.saturating_sub(top) as u32,
    }
}

fn layers_referencing_asset(layers: &[Layer], asset_id: &str) -> Vec<String> {
    let mut output = Vec::new();
    for layer in layers {
        match &layer.kind {
            LayerKind::Image { asset_id: value } if value == asset_id => output.push(layer.common.id.clone()),
            LayerKind::Text { font_asset_id, .. } if font_asset_id == asset_id => output.push(layer.common.id.clone()),
            LayerKind::Group { children } => output.extend(layers_referencing_asset(children, asset_id)),
            _ => {}
        }
    }
    output
}

fn apply(
    doc: &mut Document,
    command: &Command,
    changed_layers: &mut Vec<String>,
    changed_assets: &mut Vec<String>,
) -> Result<(), String> {
    match command {
        Command::DocumentUpdate {
            width,
            height,
            dpi,
            color,
        } => {
            if let Some(width) = width {
                doc.manifest.canvas.width = *width;
            }
            if let Some(height) = height {
                doc.manifest.canvas.height = *height;
            }
            if let Some(dpi) = dpi {
                doc.manifest.canvas.dpi = *dpi;
            }
            if let Some(color) = color {
                doc.manifest.canvas.color = *color;
            }
        }
        Command::LayerAdd {
            layer,
            parent_id,
            index,
        } => {
            let mut layer = layer.clone();
            if layer.common.id.is_empty() {
                layer.common.id = uuid::Uuid::new_v4().to_string();
            }
            if doc.find_layer(&layer.common.id).is_some() {
                return Err(format!("layer '{}' already exists", layer.common.id));
            }
            let id = layer.common.id.clone();
            let target = layer_list_mut(doc, parent_id.as_deref())?;
            let index = index.unwrap_or(target.len());
            if index > target.len() {
                return Err(format!("layer index {index} is outside 0..={}", target.len()));
            }
            target.insert(index, layer);
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
            author,
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
                author: author.clone(),
            };
            doc.manifest.assets.insert(id.clone(), asset);
            doc.asset_bytes.insert(id.clone(), bytes.into());
            changed_assets.push(id.clone());
        }
        Command::AssetLink {
            id,
            media_type,
            reference,
            byte_length,
            sha256,
            original_name,
            author,
        } => {
            doc.manifest.assets.insert(
                id.clone(),
                Asset {
                    id: id.clone(),
                    media_type: media_type.clone(),
                    byte_length: *byte_length,
                    sha256: sha256.clone(),
                    source: AssetSource::Linked {
                        reference: reference.clone(),
                    },
                    original_name: original_name.clone(),
                    author: author.clone(),
                },
            );
            doc.asset_bytes.remove(id);
            changed_assets.push(id.clone());
        }
        Command::AssetRelink {
            id,
            reference,
            byte_length,
            sha256,
        } => {
            let asset = doc
                .manifest
                .assets
                .get_mut(id)
                .ok_or_else(|| format!("asset '{id}' does not exist"))?;
            asset.source = AssetSource::Linked {
                reference: reference.clone(),
            };
            asset.byte_length = *byte_length;
            asset.sha256 = sha256.clone();
            doc.asset_bytes.remove(id);
            changed_assets.push(id.clone());
        }
        Command::AssetRemove { id } => {
            if doc.manifest.assets.remove(id).is_none() {
                return Err(format!("asset '{id}' does not exist"));
            }
            doc.asset_bytes.remove(id);
            changed_assets.push(id.clone());
        }
        Command::ExtensionSet { namespace, value } => {
            validate_namespace(namespace)?;
            doc.manifest.extensions.insert(namespace.clone(), value.clone());
        }
        Command::ExtensionRemove { namespace } => {
            validate_namespace(namespace)?;
            if doc.manifest.extensions.remove(namespace).is_none() {
                return Err(format!("extension namespace '{namespace}' does not exist"));
            }
        }
    }
    Ok(())
}

fn validate_namespace(namespace: &str) -> Result<(), String> {
    if namespace.contains('.') && !namespace.starts_with('.') && !namespace.ends_with('.') {
        Ok(())
    } else {
        Err("extension namespace must be a reverse-domain-style identifier".to_owned())
    }
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
