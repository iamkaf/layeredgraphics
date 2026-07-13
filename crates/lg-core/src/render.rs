use crate::document::{BlendMode, Document, Layer, LayerKind, Rgba};
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use image::{DynamicImage, ImageFormat, RgbaImage};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Cursor;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderOptions {
    #[serde(default)]
    pub layer_id: Option<String>,
    #[serde(default = "default_scale")]
    pub scale: f32,
    #[serde(default)]
    pub sampling: Sampling,
    #[serde(default)]
    pub background: Option<Rgba>,
}

fn default_scale() -> f32 {
    1.0
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            layer_id: None,
            scale: 1.0,
            sampling: Sampling::Nearest,
            background: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Sampling {
    #[default]
    Nearest,
    Smooth,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Png,
    Jpeg,
    Webp,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("invalid render options: {0}")]
    InvalidOptions(String),
    #[error("layer '{0}' does not exist")]
    MissingLayer(String),
    #[error("asset '{0}' is missing")]
    MissingAsset(String),
    #[error("cannot decode image asset '{id}': {message}")]
    ImageDecode { id: String, message: String },
    #[error("cannot decode font asset '{id}': {message}")]
    FontDecode { id: String, message: String },
    #[error("image encoding failed: {0}")]
    Encode(String),
}

#[derive(Debug, Clone)]
pub struct LayerRaster {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetainedRenderMetrics {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_evictions: u64,
    pub cache_bytes: u64,
    pub entries: usize,
}

pub struct RetainedRenderer {
    sources: BTreeMap<String, Arc<Surface>>,
    metrics: RetainedRenderMetrics,
    memory_budget: u64,
    revision: Option<u64>,
}

impl Default for RetainedRenderer {
    fn default() -> Self {
        Self::new(256 * 1024 * 1024)
    }
}

impl RetainedRenderer {
    pub fn new(memory_budget: u64) -> Self {
        Self {
            sources: BTreeMap::new(),
            metrics: RetainedRenderMetrics::default(),
            memory_budget,
            revision: None,
        }
    }

    pub fn render(&mut self, doc: &Document, options: &RenderOptions) -> Result<RgbaImage, RenderError> {
        if self.revision.is_some_and(|revision| revision != doc.manifest.revision) {
            self.clear();
        }
        self.revision = Some(doc.manifest.revision);
        validate_options(options)?;
        let mut canvas = initial_canvas(doc, options);
        if let Some(id) = &options.layer_id {
            let layer = doc
                .find_layer(id)
                .ok_or_else(|| RenderError::MissingLayer(id.clone()))?;
            self.render_layer(doc, layer, &mut canvas, (0.0, 0.0), 1.0, options.sampling)?;
        } else {
            for layer in &doc.manifest.layers {
                self.render_layer(doc, layer, &mut canvas, (0.0, 0.0), 1.0, options.sampling)?;
            }
        }
        finish_surface(canvas, options)
    }

    pub fn render_encoded(
        &mut self,
        doc: &Document,
        options: &RenderOptions,
        format: OutputFormat,
    ) -> Result<Vec<u8>, RenderError> {
        encode_image(self.render(doc, options)?, format)
    }

    pub fn invalidate(&mut self, doc: &Document, result: &crate::CommandResult) {
        if self.revision.is_some_and(|revision| revision != result.from_revision) {
            self.clear();
        }
        for change in &result.changes {
            match change.impact {
                crate::ChangeImpact::Metadata => continue,
                crate::ChangeImpact::Global => {
                    self.clear();
                    continue;
                }
                _ => {}
            }
            if change.reason == "layerRemoved" || change.reason == "layerStackChanged" {
                self.clear();
                continue;
            }
            for id in &change.layer_ids {
                let root = root_layer_id(&doc.manifest.layers, id).unwrap_or(id);
                let is_nested = root != id;
                let source_changed = matches!(
                    change.reason.as_str(),
                    "layerSourceChanged" | "layerAdded" | "assetChanged"
                );
                if is_nested || source_changed {
                    self.evict(root);
                    self.evict(id);
                }
            }
        }
        self.revision = Some(result.revision);
    }

    pub fn clear(&mut self) {
        self.metrics.cache_evictions += self.sources.len() as u64;
        self.sources.clear();
        self.metrics.cache_bytes = 0;
        self.metrics.entries = 0;
        self.revision = None;
    }

    pub fn metrics(&self) -> RetainedRenderMetrics {
        let mut metrics = self.metrics.clone();
        metrics.entries = self.sources.len();
        metrics
    }

    fn render_layer(
        &mut self,
        doc: &Document,
        layer: &Layer,
        target: &mut Surface,
        parent: (f32, f32),
        parent_opacity: f32,
        sampling: Sampling,
    ) -> Result<(), RenderError> {
        if !layer.common.visible || layer.common.opacity <= 0.0 {
            return Ok(());
        }
        let source = self.source(doc, layer, sampling)?;
        composite_transformed(
            target,
            &source,
            parent.0 + layer.common.transform.x,
            parent.1 + layer.common.transform.y,
            layer.common.transform.scale_x,
            layer.common.transform.scale_y,
            parent_opacity * layer.common.opacity,
            layer.common.blend_mode,
            sampling,
        );
        Ok(())
    }

    fn source(&mut self, doc: &Document, layer: &Layer, sampling: Sampling) -> Result<Arc<Surface>, RenderError> {
        let key = format!("{}:{sampling:?}", layer.common.id);
        if let Some(source) = self.sources.get(&key) {
            self.metrics.cache_hits += 1;
            return Ok(Arc::clone(source));
        }
        self.metrics.cache_misses += 1;
        let surface = match &layer.kind {
            LayerKind::Group { children } => {
                let mut group = Surface::transparent(doc.manifest.canvas.width, doc.manifest.canvas.height);
                for child in children {
                    self.render_layer(doc, child, &mut group, (0.0, 0.0), 1.0, sampling)?;
                }
                group
            }
            _ => rasterize_source(doc, layer, sampling)?,
        };
        let bytes = surface.pixels.len() as u64;
        let source = Arc::new(surface);
        self.sources.insert(key, Arc::clone(&source));
        self.metrics.cache_bytes = self.metrics.cache_bytes.saturating_add(bytes);
        self.enforce_budget();
        Ok(source)
    }

    fn evict(&mut self, id: &str) {
        let prefix = format!("{id}:");
        let keys: Vec<_> = self
            .sources
            .keys()
            .filter(|key| key.starts_with(&prefix))
            .cloned()
            .collect();
        for key in keys {
            if let Some(source) = self.sources.remove(&key) {
                self.metrics.cache_bytes = self.metrics.cache_bytes.saturating_sub(source.pixels.len() as u64);
                self.metrics.cache_evictions += 1;
            }
        }
    }

    fn enforce_budget(&mut self) {
        while self.metrics.cache_bytes > self.memory_budget && self.sources.len() > 1 {
            let Some(key) = self.sources.keys().next().cloned() else {
                break;
            };
            if let Some(source) = self.sources.remove(&key) {
                self.metrics.cache_bytes = self.metrics.cache_bytes.saturating_sub(source.pixels.len() as u64);
                self.metrics.cache_evictions += 1;
            }
        }
    }
}

fn root_layer_id<'a>(layers: &'a [Layer], wanted: &str) -> Option<&'a str> {
    for layer in layers {
        if layer.common.id == wanted || contains_layer(layer, wanted) {
            return Some(&layer.common.id);
        }
    }
    None
}

fn contains_layer(layer: &Layer, wanted: &str) -> bool {
    match &layer.kind {
        LayerKind::Group { children } => children
            .iter()
            .any(|child| child.common.id == wanted || contains_layer(child, wanted)),
        _ => false,
    }
}

#[derive(Clone)]
struct Surface {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Surface {
    fn transparent(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width as usize * height as usize * 4],
        }
    }

    fn solid(width: u32, height: u32, color: Rgba) -> Self {
        let mut pixels = vec![0; width as usize * height as usize * 4];
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[color.0, color.1, color.2, color.3]);
        }
        Self { width, height, pixels }
    }
}

pub fn render_document(doc: &Document, options: &RenderOptions) -> Result<RgbaImage, RenderError> {
    validate_options(options)?;
    let mut canvas = initial_canvas(doc, options);
    if let Some(id) = &options.layer_id {
        let layer = doc
            .find_layer(id)
            .ok_or_else(|| RenderError::MissingLayer(id.clone()))?;
        render_layer(doc, layer, &mut canvas, 0.0, 0.0, 1.0, options.sampling)?;
    } else {
        for layer in &doc.manifest.layers {
            render_layer(doc, layer, &mut canvas, 0.0, 0.0, 1.0, options.sampling)?;
        }
    }
    finish_surface(canvas, options)
}

fn validate_options(options: &RenderOptions) -> Result<(), RenderError> {
    if !options.scale.is_finite() || options.scale <= 0.0 || options.scale > 16.0 {
        return Err(RenderError::InvalidOptions(
            "scale must be finite and between 0 and 16".to_owned(),
        ));
    }
    Ok(())
}

fn initial_canvas(doc: &Document, options: &RenderOptions) -> Surface {
    let color = options.background.unwrap_or(doc.manifest.canvas.color);
    if color == Rgba::default() {
        Surface::transparent(doc.manifest.canvas.width, doc.manifest.canvas.height)
    } else {
        Surface::solid(doc.manifest.canvas.width, doc.manifest.canvas.height, color)
    }
}

fn finish_surface(canvas: Surface, options: &RenderOptions) -> Result<RgbaImage, RenderError> {
    let image =
        RgbaImage::from_raw(canvas.width, canvas.height, canvas.pixels).expect("surface dimensions match pixels");
    if (options.scale - 1.0).abs() < f32::EPSILON {
        return Ok(image);
    }
    let width = ((image.width() as f32 * options.scale).round() as u32).max(1);
    let height = ((image.height() as f32 * options.scale).round() as u32).max(1);
    let filter = match options.sampling {
        Sampling::Nearest => image::imageops::FilterType::Nearest,
        Sampling::Smooth => image::imageops::FilterType::Lanczos3,
    };
    Ok(image::imageops::resize(&image, width, height, filter))
}

pub fn render_document_png(doc: &Document, options: &RenderOptions) -> Result<Vec<u8>, RenderError> {
    render_document_encoded(doc, options, OutputFormat::Png)
}

pub fn render_document_encoded(
    doc: &Document,
    options: &RenderOptions,
    format: OutputFormat,
) -> Result<Vec<u8>, RenderError> {
    encode_image(render_document(doc, options)?, format)
}

fn encode_image(image: RgbaImage, format: OutputFormat) -> Result<Vec<u8>, RenderError> {
    let mut bytes = Cursor::new(Vec::new());
    let format = match format {
        OutputFormat::Png => ImageFormat::Png,
        OutputFormat::Jpeg => ImageFormat::Jpeg,
        OutputFormat::Webp => ImageFormat::WebP,
    };
    DynamicImage::ImageRgba8(image)
        .write_to(&mut bytes, format)
        .map_err(|error| RenderError::Encode(error.to_string()))?;
    Ok(bytes.into_inner())
}

pub fn rasterize_layer_source(doc: &Document, id: &str, sampling: Sampling) -> Result<LayerRaster, RenderError> {
    let layer = doc
        .find_layer(id)
        .ok_or_else(|| RenderError::MissingLayer(id.to_owned()))?;
    let surface = rasterize_source(doc, layer, sampling)?;
    Ok(LayerRaster {
        width: surface.width,
        height: surface.height,
        rgba: surface.pixels,
    })
}

fn rasterize_source(doc: &Document, layer: &Layer, sampling: Sampling) -> Result<Surface, RenderError> {
    match &layer.kind {
        LayerKind::Image { asset_id } => {
            let bytes = doc
                .asset_bytes
                .get(asset_id)
                .ok_or_else(|| RenderError::MissingAsset(asset_id.clone()))?;
            let image = crate::document::decode_image_limited(bytes)
                .map_err(|error| RenderError::ImageDecode {
                    id: asset_id.clone(),
                    message: error.to_string(),
                })?
                .to_rgba8();
            Ok(Surface {
                width: image.width(),
                height: image.height(),
                pixels: image.into_raw(),
            })
        }
        LayerKind::Fill { width, height, color } => Ok(Surface::solid(*width, *height, *color)),
        LayerKind::Text {
            text,
            font_asset_id,
            font_size,
            color,
        } => render_text(doc, text, font_asset_id, *font_size, *color),
        LayerKind::Group { children } => {
            let mut group = Surface::transparent(doc.manifest.canvas.width, doc.manifest.canvas.height);
            for child in children {
                render_layer(doc, child, &mut group, 0.0, 0.0, 1.0, sampling)?;
            }
            Ok(group)
        }
    }
}

fn render_layer(
    doc: &Document,
    layer: &Layer,
    target: &mut Surface,
    parent_x: f32,
    parent_y: f32,
    parent_opacity: f32,
    sampling: Sampling,
) -> Result<(), RenderError> {
    if !layer.common.visible || layer.common.opacity <= 0.0 {
        return Ok(());
    }
    let x = parent_x + layer.common.transform.x;
    let y = parent_y + layer.common.transform.y;
    let opacity = parent_opacity * layer.common.opacity;
    match &layer.kind {
        LayerKind::Image { .. } => {
            let source = rasterize_source(doc, layer, sampling)?;
            composite_transformed(
                target,
                &source,
                x,
                y,
                layer.common.transform.scale_x,
                layer.common.transform.scale_y,
                opacity,
                layer.common.blend_mode,
                sampling,
            );
        }
        LayerKind::Fill { .. } => {
            let source = rasterize_source(doc, layer, sampling)?;
            composite_transformed(
                target,
                &source,
                x,
                y,
                layer.common.transform.scale_x,
                layer.common.transform.scale_y,
                opacity,
                layer.common.blend_mode,
                sampling,
            );
        }
        LayerKind::Text { .. } => {
            let source = rasterize_source(doc, layer, sampling)?;
            composite_transformed(
                target,
                &source,
                x,
                y,
                layer.common.transform.scale_x,
                layer.common.transform.scale_y,
                opacity,
                layer.common.blend_mode,
                sampling,
            );
        }
        LayerKind::Group { children } => {
            let mut group = Surface::transparent(target.width, target.height);
            for child in children {
                render_layer(doc, child, &mut group, 0.0, 0.0, 1.0, sampling)?;
            }
            composite_transformed(
                target,
                &group,
                x,
                y,
                layer.common.transform.scale_x,
                layer.common.transform.scale_y,
                opacity,
                layer.common.blend_mode,
                sampling,
            );
        }
    }
    Ok(())
}

fn render_text(
    doc: &Document,
    text: &str,
    asset_id: &str,
    font_size: f32,
    color: Rgba,
) -> Result<Surface, RenderError> {
    let bytes = doc
        .asset_bytes
        .get(asset_id)
        .ok_or_else(|| RenderError::MissingAsset(asset_id.to_owned()))?;
    if bytes.starts_with(b"LGF1\n") {
        return render_bitmap_text(bytes, text, font_size, color).map_err(|message| RenderError::FontDecode {
            id: asset_id.to_owned(),
            message,
        });
    }
    let font = fontdue::Font::from_bytes(bytes.as_slice(), fontdue::FontSettings::default()).map_err(|message| {
        RenderError::FontDecode {
            id: asset_id.to_owned(),
            message: message.to_owned(),
        }
    })?;
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings::default());
    layout.append(&[&font], &TextStyle::new(text, font_size, 0));
    let width = layout
        .glyphs()
        .iter()
        .map(|glyph| glyph.x + glyph.width as f32)
        .fold(1.0, f32::max)
        .ceil() as u32;
    let height = layout
        .glyphs()
        .iter()
        .map(|glyph| glyph.y + glyph.height as f32)
        .fold(font_size, f32::max)
        .ceil() as u32;
    let mut surface = Surface::transparent(width.max(1), height.max(1));
    for glyph in layout.glyphs() {
        let (metrics, bitmap) = font.rasterize_config(glyph.key);
        for row in 0..metrics.height {
            for column in 0..metrics.width {
                let dx = glyph.x.floor() as i32 + column as i32;
                let dy = glyph.y.floor() as i32 + row as i32;
                if dx < 0 || dy < 0 || dx >= surface.width as i32 || dy >= surface.height as i32 {
                    continue;
                }
                let coverage = bitmap[row * metrics.width + column] as u16;
                let alpha = ((color.3 as u16 * coverage + 127) / 255) as u8;
                let index = (dy as usize * surface.width as usize + dx as usize) * 4;
                surface.pixels[index..index + 4].copy_from_slice(&[color.0, color.1, color.2, alpha]);
            }
        }
    }
    Ok(surface)
}

fn render_bitmap_text(bytes: &[u8], text: &str, font_size: f32, color: Rgba) -> Result<Surface, String> {
    let source = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
    let mut glyphs = BTreeMap::<char, [u8; 7]>::new();
    for (line_number, line) in source.lines().enumerate().skip(1) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (character, rows) = line
            .split_once('=')
            .ok_or_else(|| format!("line {} must use CHARACTER=ROW,...", line_number + 1))?;
        let character = match character {
            "SPACE" => ' ',
            value => value
                .chars()
                .next()
                .ok_or_else(|| format!("line {} has no character", line_number + 1))?,
        };
        let values = rows
            .split(',')
            .map(|value| {
                u8::from_str_radix(value, 16).map_err(|_| format!("line {} has an invalid row", line_number + 1))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let values: [u8; 7] = values
            .try_into()
            .map_err(|_| format!("line {} must contain seven rows", line_number + 1))?;
        glyphs.insert(character, values);
    }
    let pixel = (font_size / 7.0).max(1.0);
    let advance = pixel * 6.0;
    let line_height = pixel * 9.0;
    let lines: Vec<&str> = text.lines().collect();
    let max_chars = lines.iter().map(|line| line.chars().count()).max().unwrap_or(0);
    let width = (max_chars as f32 * advance).ceil().max(1.0) as u32;
    let height = (lines.len().max(1) as f32 * line_height).ceil().max(1.0) as u32;
    let mut surface = Surface::transparent(width, height);
    for (line_index, line) in lines.iter().enumerate() {
        for (character_index, character) in line.chars().enumerate() {
            let normalized = if glyphs.contains_key(&character) {
                character
            } else {
                character.to_ascii_uppercase()
            };
            let Some(rows) = glyphs.get(&normalized).or_else(|| glyphs.get(&'?')) else {
                continue;
            };
            for (row, bits) in rows.iter().enumerate() {
                for column in 0..5 {
                    if bits & (1 << (4 - column)) == 0 {
                        continue;
                    }
                    let left = (character_index as f32 * advance + column as f32 * pixel).floor() as u32;
                    let top = (line_index as f32 * line_height + row as f32 * pixel).floor() as u32;
                    let right =
                        ((character_index as f32 * advance + (column + 1) as f32 * pixel).ceil() as u32).min(width);
                    let bottom =
                        ((line_index as f32 * line_height + (row + 1) as f32 * pixel).ceil() as u32).min(height);
                    for y in top..bottom {
                        for x in left..right {
                            let index = (y as usize * width as usize + x as usize) * 4;
                            surface.pixels[index..index + 4].copy_from_slice(&[color.0, color.1, color.2, color.3]);
                        }
                    }
                }
            }
        }
    }
    Ok(surface)
}

#[allow(clippy::too_many_arguments)]
fn composite_transformed(
    target: &mut Surface,
    source: &Surface,
    x: f32,
    y: f32,
    scale_x: f32,
    scale_y: f32,
    opacity: f32,
    blend_mode: BlendMode,
    sampling: Sampling,
) {
    let output_width = (source.width as f32 * scale_x.abs()).ceil() as i32;
    let output_height = (source.height as f32 * scale_y.abs()).ceil() as i32;
    if output_width <= 0 || output_height <= 0 {
        return;
    }
    let origin_x = x.round() as i32;
    let origin_y = y.round() as i32;
    for oy in 0..output_height {
        let ty = origin_y + oy;
        if ty < 0 || ty >= target.height as i32 {
            continue;
        }
        let normalized_y = (oy as f32 + 0.5) / output_height as f32;
        let normalized_y = if scale_y < 0.0 {
            1.0 - normalized_y
        } else {
            normalized_y
        };
        let sy = (normalized_y * source.height as f32)
            .floor()
            .clamp(0.0, source.height.saturating_sub(1) as f32) as u32;
        for ox in 0..output_width {
            let tx = origin_x + ox;
            if tx < 0 || tx >= target.width as i32 {
                continue;
            }
            let normalized_x = (ox as f32 + 0.5) / output_width as f32;
            let normalized_x = if scale_x < 0.0 {
                1.0 - normalized_x
            } else {
                normalized_x
            };
            let sx = (normalized_x * source.width as f32)
                .floor()
                .clamp(0.0, source.width.saturating_sub(1) as f32) as u32;
            let target_index = (ty as usize * target.width as usize + tx as usize) * 4;
            let sampled = match sampling {
                Sampling::Nearest => {
                    let source_index = (sy as usize * source.width as usize + sx as usize) * 4;
                    [
                        source.pixels[source_index],
                        source.pixels[source_index + 1],
                        source.pixels[source_index + 2],
                        source.pixels[source_index + 3],
                    ]
                }
                Sampling::Smooth => sample_bilinear(source, normalized_x, normalized_y),
            };
            blend_pixel(
                &mut target.pixels[target_index..target_index + 4],
                &sampled,
                opacity,
                blend_mode,
            );
        }
    }
}

fn sample_bilinear(source: &Surface, normalized_x: f32, normalized_y: f32) -> [u8; 4] {
    let x = (normalized_x * source.width as f32 - 0.5).clamp(0.0, source.width.saturating_sub(1) as f32);
    let y = (normalized_y * source.height as f32 - 0.5).clamp(0.0, source.height.saturating_sub(1) as f32);
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(source.width - 1);
    let y1 = (y0 + 1).min(source.height - 1);
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;
    let mut output = [0; 4];
    for (channel, output_channel) in output.iter_mut().enumerate() {
        let at =
            |px: u32, py: u32| source.pixels[(py as usize * source.width as usize + px as usize) * 4 + channel] as f32;
        let top = at(x0, y0) * (1.0 - tx) + at(x1, y0) * tx;
        let bottom = at(x0, y1) * (1.0 - tx) + at(x1, y1) * tx;
        *output_channel = (top * (1.0 - ty) + bottom * ty + 0.5) as u8;
    }
    output
}

fn blend_pixel(backdrop: &mut [u8], source: &[u8], opacity: f32, mode: BlendMode) {
    let source_alpha = source[3] as f32 / 255.0 * opacity.clamp(0.0, 1.0);
    let backdrop_alpha = backdrop[3] as f32 / 255.0;
    let output_alpha = source_alpha + backdrop_alpha * (1.0 - source_alpha);
    if output_alpha <= f32::EPSILON {
        backdrop.copy_from_slice(&[0, 0, 0, 0]);
        return;
    }
    for channel in 0..3 {
        let cs = source[channel] as f32 / 255.0;
        let cb = backdrop[channel] as f32 / 255.0;
        let blended = match mode {
            BlendMode::Normal => cs,
            BlendMode::Multiply => cs * cb,
        };
        let premultiplied = (1.0 - source_alpha) * backdrop_alpha * cb
            + (1.0 - backdrop_alpha) * source_alpha * cs
            + source_alpha * backdrop_alpha * blended;
        backdrop[channel] = ((premultiplied / output_alpha).clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    }
    backdrop[3] = (output_alpha.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LayerCommon, Transform};

    #[test]
    fn renders_bottom_to_top_with_opacity() {
        let mut doc = Document::new(2, 2, 72.0);
        doc.manifest.layers.push(Layer {
            common: common("bottom"),
            kind: LayerKind::Fill {
                width: 2,
                height: 2,
                color: Rgba(255, 0, 0, 255),
            },
        });
        let mut top = common("top");
        top.opacity = 0.5;
        doc.manifest.layers.push(Layer {
            common: top,
            kind: LayerKind::Fill {
                width: 2,
                height: 2,
                color: Rgba(0, 0, 255, 255),
            },
        });
        let image = render_document(&doc, &RenderOptions::default()).unwrap();
        assert_eq!(image.get_pixel(0, 0).0, [128, 0, 128, 255]);
    }

    #[test]
    fn multiply_blends_colors() {
        let mut pixel = [128, 128, 255, 255];
        blend_pixel(&mut pixel, &[128, 255, 128, 255], 1.0, BlendMode::Multiply);
        assert_eq!(pixel, [64, 128, 128, 255]);
    }

    #[test]
    fn hidden_layers_do_not_affect_output() {
        let mut doc = Document::new(1, 1, 72.0);
        let mut hidden = common("hidden");
        hidden.visible = false;
        doc.manifest.layers.push(Layer {
            common: hidden,
            kind: LayerKind::Fill {
                width: 1,
                height: 1,
                color: Rgba(255, 0, 0, 255),
            },
        });
        let image = render_document(&doc, &RenderOptions::default()).unwrap();
        assert_eq!(image.get_pixel(0, 0).0, [0, 0, 0, 0]);
    }

    fn common(id: &str) -> LayerCommon {
        LayerCommon {
            id: id.to_owned(),
            name: id.to_owned(),
            visible: true,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            transform: Transform::default(),
        }
    }
}
