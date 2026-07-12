use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub manifest: Manifest,
    #[serde(skip)]
    pub asset_bytes: BTreeMap<String, Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: u32,
    pub id: String,
    pub revision: u64,
    pub canvas: Canvas,
    #[serde(default)]
    pub layers: Vec<Layer>,
    #[serde(default)]
    pub assets: BTreeMap<String, Asset>,
    #[serde(default)]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    #[serde(default = "default_dpi")]
    pub dpi: f32,
}

fn default_dpi() -> f32 {
    72.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: String,
    pub media_type: String,
    pub byte_length: u64,
    pub sha256: String,
    pub source: AssetSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AssetSource {
    Embedded { path: String },
    Linked { reference: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    #[serde(flatten)]
    pub common: LayerCommon,
    #[serde(flatten)]
    pub kind: LayerKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayerCommon {
    pub id: String,
    pub name: String,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub blend_mode: BlendMode,
    #[serde(default)]
    pub transform: Transform,
}

fn default_true() -> bool {
    true
}

fn default_opacity() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LayerKind {
    Image {
        asset_id: String,
    },
    Fill {
        width: u32,
        height: u32,
        color: Rgba,
    },
    Text {
        text: String,
        font_asset_id: String,
        font_size: f32,
        color: Rgba,
    },
    Group {
        #[serde(default)]
        children: Vec<Layer>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Transform {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default = "default_scale")]
    pub scale_x: f32,
    #[serde(default = "default_scale")]
    pub scale_y: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
}

fn default_scale() -> f32 {
    1.0
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ValidationDiagnostic {
    pub severity: ValidationSeverity,
    pub code: String,
    pub message: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
    Error,
    Warning,
}

impl Document {
    pub fn new(width: u32, height: u32, dpi: f32) -> Self {
        Self {
            manifest: Manifest {
                schema_version: SCHEMA_VERSION,
                id: uuid::Uuid::new_v4().to_string(),
                revision: 0,
                canvas: Canvas { width, height, dpi },
                layers: Vec::new(),
                assets: BTreeMap::new(),
                extensions: BTreeMap::new(),
            },
            asset_bytes: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> Vec<ValidationDiagnostic> {
        let mut out = Vec::new();
        if self.manifest.schema_version != SCHEMA_VERSION {
            out.push(error(
                "schema.unsupported",
                "Unsupported schema version",
                "schemaVersion",
            ));
        }
        if self.manifest.canvas.width == 0 || self.manifest.canvas.height == 0 {
            out.push(error(
                "canvas.empty",
                "Canvas dimensions must be greater than zero",
                "canvas",
            ));
        }
        if !self.manifest.canvas.dpi.is_finite() || self.manifest.canvas.dpi <= 0.0 {
            out.push(error(
                "canvas.dpi",
                "DPI must be a positive finite number",
                "canvas.dpi",
            ));
        }

        let mut ids = BTreeSet::new();
        validate_layers(&self.manifest.layers, "layers", self, &mut ids, &mut out);
        for (id, asset) in &self.manifest.assets {
            if id != &asset.id {
                out.push(error(
                    "asset.idMismatch",
                    "Asset map key must match its id",
                    format!("assets.{id}.id"),
                ));
            }
            if let AssetSource::Embedded { .. } = asset.source {
                match self.asset_bytes.get(id) {
                    Some(bytes) if bytes.len() as u64 != asset.byte_length => out.push(error(
                        "asset.lengthMismatch",
                        "Embedded asset byte length does not match manifest",
                        format!("assets.{id}"),
                    )),
                    None => out.push(error(
                        "asset.missing",
                        "Embedded asset payload is missing",
                        format!("assets.{id}"),
                    )),
                    _ => {}
                }
            }
        }
        out
    }

    pub fn find_layer(&self, id: &str) -> Option<&Layer> {
        find_layer(&self.manifest.layers, id)
    }

    pub fn find_layer_mut(&mut self, id: &str) -> Option<&mut Layer> {
        find_layer_mut(&mut self.manifest.layers, id)
    }
}

fn validate_layers(
    layers: &[Layer],
    path: &str,
    doc: &Document,
    ids: &mut BTreeSet<String>,
    out: &mut Vec<ValidationDiagnostic>,
) {
    for (index, layer) in layers.iter().enumerate() {
        let at = format!("{path}.{index}");
        if !ids.insert(layer.common.id.clone()) {
            out.push(error(
                "layer.duplicateId",
                "Layer ids must be unique",
                format!("{at}.id"),
            ));
        }
        if !(0.0..=1.0).contains(&layer.common.opacity) || !layer.common.opacity.is_finite() {
            out.push(error(
                "layer.opacity",
                "Opacity must be between 0 and 1",
                format!("{at}.opacity"),
            ));
        }
        if !layer.common.transform.scale_x.is_finite()
            || !layer.common.transform.scale_y.is_finite()
            || layer.common.transform.scale_x == 0.0
            || layer.common.transform.scale_y == 0.0
        {
            out.push(error(
                "layer.scale",
                "Scale must be finite and non-zero",
                format!("{at}.transform"),
            ));
        }
        match &layer.kind {
            LayerKind::Image { asset_id } => require_asset(doc, asset_id, &at, "image", out),
            LayerKind::Text {
                font_asset_id,
                font_size,
                ..
            } => {
                require_asset(doc, font_asset_id, &at, "font", out);
                if !font_size.is_finite() || *font_size <= 0.0 {
                    out.push(error(
                        "text.fontSize",
                        "Font size must be positive",
                        format!("{at}.fontSize"),
                    ));
                }
            }
            LayerKind::Fill { width, height, .. } if *width == 0 || *height == 0 => {
                out.push(error("fill.empty", "Fill dimensions must be greater than zero", at));
            }
            LayerKind::Group { children } => validate_layers(children, &format!("{at}.children"), doc, ids, out),
            _ => {}
        }
    }
}

fn require_asset(doc: &Document, id: &str, at: &str, role: &str, out: &mut Vec<ValidationDiagnostic>) {
    if !doc.manifest.assets.contains_key(id) {
        out.push(error(
            "layer.missingAsset",
            format!("Layer references missing {role} asset '{id}'"),
            at.to_owned(),
        ));
    }
}

fn error(code: impl Into<String>, message: impl Into<String>, path: impl Into<String>) -> ValidationDiagnostic {
    ValidationDiagnostic {
        severity: ValidationSeverity::Error,
        code: code.into(),
        message: message.into(),
        path: path.into(),
    }
}

pub(crate) fn find_layer<'a>(layers: &'a [Layer], id: &str) -> Option<&'a Layer> {
    for layer in layers {
        if layer.common.id == id {
            return Some(layer);
        }
        if let LayerKind::Group { children } = &layer.kind
            && let Some(found) = find_layer(children, id)
        {
            return Some(found);
        }
    }
    None
}

pub(crate) fn find_layer_mut<'a>(layers: &'a mut [Layer], id: &str) -> Option<&'a mut Layer> {
    for layer in layers {
        if layer.common.id == id {
            return Some(layer);
        }
        if let LayerKind::Group { children } = &mut layer.kind
            && let Some(found) = find_layer_mut(children, id)
        {
            return Some(found);
        }
    }
    None
}
