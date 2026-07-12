use lg_core::{RenderOptions, load_kgfx_bytes, render_document_png};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

#[wasm_bindgen]
pub fn validate_kgfx(bytes: &[u8]) -> Result<String, JsError> {
    let document = load_kgfx_bytes(bytes).map_err(|error| JsError::new(&error.to_string()))?;
    serde_json::to_string(&document.validate()).map_err(|error| JsError::new(&error.to_string()))
}

#[wasm_bindgen]
pub fn inspect_kgfx(bytes: &[u8]) -> Result<String, JsError> {
    let document = load_kgfx_bytes(bytes).map_err(|error| JsError::new(&error.to_string()))?;
    serde_json::to_string(&document.manifest).map_err(|error| JsError::new(&error.to_string()))
}

#[wasm_bindgen]
pub fn render_kgfx_png(bytes: &[u8], scale: f32) -> Result<Vec<u8>, JsError> {
    let document = load_kgfx_bytes(bytes).map_err(|error| JsError::new(&error.to_string()))?;
    render_document_png(&document, &RenderOptions { layer_id: None, scale })
        .map_err(|error| JsError::new(&error.to_string()))
}
