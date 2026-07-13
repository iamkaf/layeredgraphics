use crate::{AssetSource, Document, Layer, LayerKind, Rect, RenderOptions, render_document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInspection {
    pub layer_count: usize,
    pub asset_count: usize,
    pub canvas_bounds: Rect,
    pub visible_content_bounds: Option<Rect>,
    pub layers: Vec<LayerInspection>,
    pub assets: Vec<AssetInspection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerInspection {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub declared_bounds: Rect,
    pub transformed_bounds: Rect,
    pub visible_content_bounds: Option<Rect>,
    pub children: Vec<LayerInspection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetInspection {
    pub id: String,
    pub media_type: String,
    pub embedded: bool,
    pub resolved: bool,
    pub reference: Option<String>,
    pub byte_length: u64,
}

pub fn inspect_document(doc: &Document, analyze_pixels: bool) -> DocumentInspection {
    let visible_content_bounds = analyze_pixels
        .then(|| {
            render_document(doc, &RenderOptions::default())
                .ok()
                .and_then(|image| alpha_bounds(image.as_raw(), image.width(), image.height()))
        })
        .flatten();
    DocumentInspection {
        layer_count: count_layers(&doc.manifest.layers),
        asset_count: doc.manifest.assets.len(),
        canvas_bounds: Rect {
            x: 0,
            y: 0,
            width: doc.manifest.canvas.width,
            height: doc.manifest.canvas.height,
        },
        visible_content_bounds,
        layers: doc
            .manifest
            .layers
            .iter()
            .map(|layer| inspect_layer(doc, layer, analyze_pixels))
            .collect(),
        assets: doc
            .manifest
            .assets
            .values()
            .map(|asset| AssetInspection {
                id: asset.id.clone(),
                media_type: asset.media_type.clone(),
                embedded: matches!(asset.source, AssetSource::Embedded { .. }),
                resolved: doc.asset_bytes.contains_key(&asset.id),
                reference: match &asset.source {
                    AssetSource::Linked { reference } => Some(reference.clone()),
                    AssetSource::Embedded { .. } => None,
                },
                byte_length: asset.byte_length,
            })
            .collect(),
    }
}

fn inspect_layer(doc: &Document, layer: &Layer, analyze_pixels: bool) -> LayerInspection {
    let (kind, width, height, children) = match &layer.kind {
        LayerKind::Image { asset_id } => {
            let dimensions = doc
                .asset_bytes
                .get(asset_id)
                .and_then(|bytes| crate::document::image_dimensions_limited(bytes).ok())
                .unwrap_or((0, 0));
            ("image", dimensions.0, dimensions.1, Vec::new())
        }
        LayerKind::Fill { width, height, .. } => ("fill", *width, *height, Vec::new()),
        LayerKind::Text { text, font_size, .. } => (
            "text",
            (text.lines().map(str::len).max().unwrap_or(0) as f32 * font_size * 0.65).ceil() as u32,
            (text.lines().count().max(1) as f32 * font_size * 1.3).ceil() as u32,
            Vec::new(),
        ),
        LayerKind::Group { children } => (
            "group",
            doc.manifest.canvas.width,
            doc.manifest.canvas.height,
            children
                .iter()
                .map(|child| inspect_layer(doc, child, analyze_pixels))
                .collect(),
        ),
    };
    let declared_bounds = Rect {
        x: 0,
        y: 0,
        width,
        height,
    };
    let transformed_bounds = Rect {
        x: layer.common.transform.x.floor() as i32,
        y: layer.common.transform.y.floor() as i32,
        width: (width as f32 * layer.common.transform.scale_x.abs()).ceil() as u32,
        height: (height as f32 * layer.common.transform.scale_y.abs()).ceil() as u32,
    };
    let visible_content_bounds = analyze_pixels
        .then(|| {
            render_document(
                doc,
                &RenderOptions {
                    layer_id: Some(layer.common.id.clone()),
                    ..RenderOptions::default()
                },
            )
            .ok()
            .and_then(|image| alpha_bounds(image.as_raw(), image.width(), image.height()))
        })
        .flatten();
    LayerInspection {
        id: layer.common.id.clone(),
        name: layer.common.name.clone(),
        kind: kind.to_owned(),
        declared_bounds,
        transformed_bounds,
        visible_content_bounds,
        children,
    }
}

fn alpha_bounds(pixels: &[u8], width: u32, height: u32) -> Option<Rect> {
    let mut left = width;
    let mut top = height;
    let mut right = 0;
    let mut bottom = 0;
    let mut found = false;
    for y in 0..height {
        for x in 0..width {
            if pixels[(y as usize * width as usize + x as usize) * 4 + 3] == 0 {
                continue;
            }
            found = true;
            left = left.min(x);
            top = top.min(y);
            right = right.max(x + 1);
            bottom = bottom.max(y + 1);
        }
    }
    found.then_some(Rect {
        x: left as i32,
        y: top as i32,
        width: right - left,
        height: bottom - top,
    })
}

fn count_layers(layers: &[Layer]) -> usize {
    layers
        .iter()
        .map(|layer| {
            1 + match &layer.kind {
                LayerKind::Group { children } => count_layers(children),
                _ => 0,
            }
        })
        .sum()
}
