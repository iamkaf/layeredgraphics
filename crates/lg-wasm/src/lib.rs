use lg_core::{
    Command, Document, DocumentSession, OutputFormat, RenderOptions, RetainedRenderer, Sampling, load_kgfx_bytes,
    rasterize_layer_source, render_document_encoded, render_document_png, save_kgfx_bytes,
};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

#[wasm_bindgen]
pub fn validate_kgfx(bytes: &[u8]) -> Result<String, JsError> {
    let document = load_kgfx_bytes(bytes).map_err(js_error)?;
    serde_json::to_string(&document.validate()).map_err(js_error)
}

#[wasm_bindgen]
pub fn inspect_kgfx(bytes: &[u8]) -> Result<String, JsError> {
    let document = load_kgfx_bytes(bytes).map_err(js_error)?;
    serde_json::to_string(&document.manifest).map_err(js_error)
}

#[wasm_bindgen]
pub fn render_kgfx_png(bytes: &[u8], scale: f32) -> Result<Vec<u8>, JsError> {
    let document = load_kgfx_bytes(bytes).map_err(js_error)?;
    render_document_png(
        &document,
        &RenderOptions {
            layer_id: None,
            scale,
            ..RenderOptions::default()
        },
    )
    .map_err(js_error)
}

#[wasm_bindgen(js_name = Document)]
pub struct WasmDocument {
    session: DocumentSession,
    renderer: RefCell<RetainedRenderer>,
}

#[wasm_bindgen(js_class = Document)]
impl WasmDocument {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32, dpi: f32) -> WasmDocument {
        WasmDocument {
            session: DocumentSession::new(Document::new(width, height, dpi)),
            renderer: RefCell::new(RetainedRenderer::default()),
        }
    }

    #[wasm_bindgen(js_name = open)]
    pub fn open(bytes: &[u8]) -> Result<WasmDocument, JsError> {
        Ok(WasmDocument {
            session: DocumentSession::new(load_kgfx_bytes(bytes).map_err(js_error)?),
            renderer: RefCell::new(RetainedRenderer::default()),
        })
    }

    pub fn execute(&mut self, operations_json: &str, label: Option<String>) -> Result<String, JsError> {
        let commands = parse_commands(operations_json)?;
        let result = self
            .session
            .execute(label.unwrap_or_else(|| "Execute commands".to_owned()), &commands)
            .map_err(js_error)?;
        self.renderer.borrow_mut().invalidate(self.session.document(), &result);
        serde_json::to_string(&result).map_err(js_error)
    }

    pub fn undo(&mut self) -> Result<String, JsError> {
        let state = self.session.undo().map_err(js_error)?;
        self.renderer.borrow_mut().clear();
        serde_json::to_string(&state).map_err(js_error)
    }

    pub fn redo(&mut self) -> Result<String, JsError> {
        let state = self.session.redo().map_err(js_error)?;
        self.renderer.borrow_mut().clear();
        serde_json::to_string(&state).map_err(js_error)
    }

    #[wasm_bindgen(js_name = historyState)]
    pub fn history_state(&self) -> Result<String, JsError> {
        serde_json::to_string(&self.session.state()).map_err(js_error)
    }

    pub fn manifest(&self) -> Result<String, JsError> {
        serde_json::to_string(&self.session.document().manifest).map_err(js_error)
    }

    #[wasm_bindgen(js_name = provideLinkedAsset)]
    pub fn provide_linked_asset(&mut self, id: &str, bytes: &[u8]) -> Result<(), JsError> {
        self.session
            .document_mut()
            .provide_linked_asset(id, bytes.to_vec())
            .map_err(js_error)?;
        self.renderer.borrow_mut().clear();
        Ok(())
    }

    #[wasm_bindgen(js_name = exportKgfx)]
    pub fn export_kgfx(&self) -> Result<Vec<u8>, JsError> {
        save_kgfx_bytes(self.session.document()).map_err(js_error)
    }

    pub fn render(&self, format: &str, options_json: Option<String>) -> Result<Vec<u8>, JsError> {
        let format = match format {
            "png" => OutputFormat::Png,
            "jpg" | "jpeg" => OutputFormat::Jpeg,
            "webp" => OutputFormat::Webp,
            _ => return Err(JsError::new("format must be png, jpg, jpeg, or webp")),
        };
        let options = options_json
            .map(|value| serde_json::from_str(&value).map_err(js_error))
            .transpose()?
            .unwrap_or_default();
        render_document_encoded(self.session.document(), &options, format).map_err(js_error)
    }

    #[wasm_bindgen(js_name = renderRgba)]
    pub fn render_rgba(&self, options_json: Option<String>) -> Result<Vec<u8>, JsError> {
        let options = options_json
            .map(|value| serde_json::from_str(&value).map_err(js_error))
            .transpose()?
            .unwrap_or_default();
        let image = self
            .renderer
            .borrow_mut()
            .render(self.session.document(), &options)
            .map_err(js_error)?;
        Ok(pack_raster(image.width(), image.height(), image.into_raw()))
    }

    #[wasm_bindgen(js_name = renderRetained)]
    pub fn render_retained(&self, format: &str, options_json: Option<String>) -> Result<Vec<u8>, JsError> {
        let format = parse_format(format)?;
        let options = options_json
            .map(|value| serde_json::from_str(&value).map_err(js_error))
            .transpose()?
            .unwrap_or_default();
        self.renderer
            .borrow_mut()
            .render_encoded(self.session.document(), &options, format)
            .map_err(js_error)
    }

    #[wasm_bindgen(js_name = retainedMetrics)]
    pub fn retained_metrics(&self) -> Result<String, JsError> {
        serde_json::to_string(&self.renderer.borrow().metrics()).map_err(js_error)
    }

    #[wasm_bindgen(js_name = rasterizeLayer)]
    pub fn rasterize_layer(&self, id: &str, sampling: &str) -> Result<Vec<u8>, JsError> {
        let sampling = match sampling {
            "nearest" => Sampling::Nearest,
            "smooth" => Sampling::Smooth,
            _ => return Err(JsError::new("sampling must be nearest or smooth")),
        };
        let raster = rasterize_layer_source(self.session.document(), id, sampling).map_err(js_error)?;
        Ok(pack_raster(raster.width, raster.height, raster.rgba))
    }
}

fn parse_format(format: &str) -> Result<OutputFormat, JsError> {
    match format {
        "png" => Ok(OutputFormat::Png),
        "jpg" | "jpeg" => Ok(OutputFormat::Jpeg),
        "webp" => Ok(OutputFormat::Webp),
        _ => Err(JsError::new("format must be png, jpg, jpeg, or webp")),
    }
}

fn pack_raster(width: u32, height: u32, rgba: Vec<u8>) -> Vec<u8> {
    let mut packed = Vec::with_capacity(8 + rgba.len());
    packed.extend_from_slice(&width.to_le_bytes());
    packed.extend_from_slice(&height.to_le_bytes());
    packed.extend_from_slice(&rgba);
    packed
}

fn parse_commands(value: &str) -> Result<Vec<Command>, JsError> {
    let value: serde_json::Value = serde_json::from_str(value).map_err(js_error)?;
    if value.is_array() {
        serde_json::from_value(value).map_err(js_error)
    } else {
        Ok(vec![serde_json::from_value(value).map_err(js_error)?])
    }
}

fn js_error(error: impl std::fmt::Display) -> JsError {
    JsError::new(&error.to_string())
}
