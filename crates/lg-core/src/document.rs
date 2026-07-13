use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Cursor;
use std::sync::Arc;

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_CANVAS_DIMENSION: u32 = 32_768;
pub const MAX_CANVAS_PIXELS: u64 = 268_435_456;
pub const MAX_LAYER_COUNT: usize = 10_000;
pub const MAX_LAYER_DEPTH: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub manifest: Manifest,
    #[serde(skip)]
    pub asset_bytes: BTreeMap<String, Arc<Vec<u8>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    #[serde(default = "default_dpi")]
    pub dpi: f32,
    #[serde(default)]
    pub color: Rgba,
}

fn default_dpi() -> f32 {
    72.0
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: String,
    pub media_type: String,
    pub byte_length: u64,
    pub sha256: String,
    pub source: AssetSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AssetSource {
    Embedded { path: String },
    Linked { reference: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    #[serde(flatten)]
    pub common: LayerCommon,
    #[serde(flatten)]
    pub kind: LayerKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayerCommon {
    #[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LayerKind {
    Image {
        #[serde(alias = "asset_id")]
        asset_id: String,
    },
    Fill {
        width: u32,
        height: u32,
        color: Rgba,
    },
    Text {
        text: String,
        #[serde(alias = "font_asset_id")]
        font_asset_id: String,
        #[serde(alias = "font_size")]
        font_size: f32,
        color: Rgba,
    },
    Group {
        #[serde(default)]
        children: Vec<Layer>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
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

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ValidationDiagnostic {
    pub severity: ValidationSeverity,
    pub code: String,
    pub message: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
    Error,
    Warning,
}

#[derive(Debug, thiserror::Error)]
pub enum AssetResolveError {
    #[error("linked asset '{id}' could not be resolved from '{reference}': {message}")]
    Resolver {
        id: String,
        reference: String,
        message: String,
    },
    #[error("linked asset '{id}' has {actual} bytes, expected {expected}")]
    Length { id: String, expected: u64, actual: u64 },
    #[error("linked asset '{id}' integrity check failed")]
    Integrity { id: String },
}

impl Document {
    pub fn new(width: u32, height: u32, dpi: f32) -> Self {
        Self {
            manifest: Manifest {
                schema_version: SCHEMA_VERSION,
                id: uuid::Uuid::new_v4().to_string(),
                revision: 0,
                canvas: Canvas {
                    width,
                    height,
                    dpi,
                    color: Rgba::default(),
                },
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
        if self.manifest.canvas.width > MAX_CANVAS_DIMENSION
            || self.manifest.canvas.height > MAX_CANVAS_DIMENSION
            || self.manifest.canvas.width as u64 * self.manifest.canvas.height as u64 > MAX_CANVAS_PIXELS
        {
            out.push(error(
                "canvas.resourceLimit",
                "Canvas dimensions exceed the supported resource limit",
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
        let mut layer_count = 0;
        validate_layers(
            &self.manifest.layers,
            "layers",
            self,
            &mut ids,
            &mut out,
            0,
            &mut layer_count,
        );
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

    pub fn resolve_linked_assets(
        &mut self,
        mut resolver: impl FnMut(&str) -> Result<Vec<u8>, String>,
    ) -> Result<usize, AssetResolveError> {
        let linked = self
            .manifest
            .assets
            .iter()
            .filter_map(|(id, asset)| match &asset.source {
                AssetSource::Linked { reference } => {
                    Some((id.clone(), reference.clone(), asset.byte_length, asset.sha256.clone()))
                }
                AssetSource::Embedded { .. } => None,
            })
            .collect::<Vec<_>>();
        let mut resolved = 0;
        for (id, reference, expected_length, expected_sha) in linked {
            let bytes = resolver(&reference).map_err(|message| AssetResolveError::Resolver {
                id: id.clone(),
                reference: reference.clone(),
                message,
            })?;
            if bytes.len() as u64 != expected_length {
                return Err(AssetResolveError::Length {
                    id,
                    expected: expected_length,
                    actual: bytes.len() as u64,
                });
            }
            if hex::encode(Sha256::digest(&bytes)) != expected_sha {
                return Err(AssetResolveError::Integrity { id });
            }
            self.asset_bytes.insert(id, Arc::new(bytes));
            resolved += 1;
        }
        Ok(resolved)
    }

    pub fn provide_linked_asset(&mut self, id: &str, bytes: Vec<u8>) -> Result<(), AssetResolveError> {
        let asset = self
            .manifest
            .assets
            .get(id)
            .ok_or_else(|| AssetResolveError::Resolver {
                id: id.to_owned(),
                reference: String::new(),
                message: "asset does not exist".to_owned(),
            })?;
        match &asset.source {
            AssetSource::Linked { .. } => {}
            AssetSource::Embedded { .. } => {
                return Err(AssetResolveError::Resolver {
                    id: id.to_owned(),
                    reference: String::new(),
                    message: "asset is embedded, not linked".to_owned(),
                });
            }
        }
        if bytes.len() as u64 != asset.byte_length {
            return Err(AssetResolveError::Length {
                id: id.to_owned(),
                expected: asset.byte_length,
                actual: bytes.len() as u64,
            });
        }
        if hex::encode(Sha256::digest(&bytes)) != asset.sha256 {
            return Err(AssetResolveError::Integrity { id: id.to_owned() });
        }
        self.asset_bytes.insert(id.to_owned(), Arc::new(bytes));
        Ok(())
    }
}

fn validate_layers(
    layers: &[Layer],
    path: &str,
    doc: &Document,
    ids: &mut BTreeSet<String>,
    out: &mut Vec<ValidationDiagnostic>,
    depth: usize,
    layer_count: &mut usize,
) {
    if depth > MAX_LAYER_DEPTH {
        out.push(error(
            "layer.depthLimit",
            "Layer nesting exceeds the supported resource limit",
            path,
        ));
        return;
    }
    for (index, layer) in layers.iter().enumerate() {
        *layer_count += 1;
        if *layer_count > MAX_LAYER_COUNT {
            out.push(error(
                "layer.countLimit",
                "Layer count exceeds the supported resource limit",
                path,
            ));
            return;
        }
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
            LayerKind::Group { children } => validate_layers(
                children,
                &format!("{at}.children"),
                doc,
                ids,
                out,
                depth + 1,
                layer_count,
            ),
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

pub(crate) fn decode_image_limited(bytes: &[u8]) -> image::ImageResult<image::DynamicImage> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    reader.limits(image_limits());
    reader.decode()
}

pub(crate) fn image_dimensions_limited(bytes: &[u8]) -> image::ImageResult<(u32, u32)> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    reader.limits(image_limits());
    reader.into_dimensions()
}

fn image_limits() -> image::Limits {
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_CANVAS_DIMENSION);
    limits.max_image_height = Some(MAX_CANVAS_DIMENSION);
    limits.max_alloc = Some(512 * 1024 * 1024);
    limits
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
