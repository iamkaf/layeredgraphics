use lg_core::{
    Command, Document, DocumentSession, OutputFormat, load_kgfx_bytes, render_document_encoded, save_kgfx_bytes,
};
use napi::bindgen_prelude::{Buffer, Error, Result, Status};
use napi_derive::napi;

#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

#[napi]
pub struct NativeDocument {
    session: DocumentSession,
}

#[napi]
impl NativeDocument {
    #[napi(constructor)]
    pub fn new(width: u32, height: u32, dpi: f64) -> NativeDocument {
        NativeDocument {
            session: DocumentSession::new(Document::new(width, height, dpi as f32)),
        }
    }

    #[napi(factory)]
    pub fn open(bytes: Buffer) -> Result<NativeDocument> {
        Ok(NativeDocument {
            session: DocumentSession::new(load_kgfx_bytes(bytes.as_ref()).map_err(node_error)?),
        })
    }

    #[napi]
    pub fn execute(&mut self, operations_json: String, label: Option<String>) -> Result<String> {
        let commands = parse_commands(&operations_json)?;
        let result = self
            .session
            .execute(label.unwrap_or_else(|| "Execute commands".to_owned()), &commands)
            .map_err(node_error)?;
        serde_json::to_string(&result).map_err(node_error)
    }

    #[napi]
    pub fn undo(&mut self) -> Result<String> {
        serde_json::to_string(&self.session.undo().map_err(node_error)?).map_err(node_error)
    }

    #[napi]
    pub fn redo(&mut self) -> Result<String> {
        serde_json::to_string(&self.session.redo().map_err(node_error)?).map_err(node_error)
    }

    #[napi(getter)]
    pub fn history_state(&self) -> Result<String> {
        serde_json::to_string(&self.session.state()).map_err(node_error)
    }

    #[napi(getter)]
    pub fn manifest(&self) -> Result<String> {
        serde_json::to_string(&self.session.document().manifest).map_err(node_error)
    }

    #[napi]
    pub fn provide_linked_asset(&mut self, id: String, bytes: Buffer) -> Result<()> {
        self.session
            .document_mut()
            .provide_linked_asset(&id, bytes.to_vec())
            .map_err(node_error)
    }

    #[napi]
    pub fn export_kgfx(&self) -> Result<Buffer> {
        Ok(save_kgfx_bytes(self.session.document()).map_err(node_error)?.into())
    }

    #[napi]
    pub fn render(&self, format: String, options_json: Option<String>) -> Result<Buffer> {
        let format = match format.as_str() {
            "png" => OutputFormat::Png,
            "jpg" | "jpeg" => OutputFormat::Jpeg,
            "webp" => OutputFormat::Webp,
            _ => return Err(Error::new(Status::InvalidArg, "format must be png, jpg, jpeg, or webp")),
        };
        let options = options_json
            .map(|value| serde_json::from_str(&value).map_err(node_error))
            .transpose()?
            .unwrap_or_default();
        Ok(render_document_encoded(self.session.document(), &options, format)
            .map_err(node_error)?
            .into())
    }
}

fn parse_commands(value: &str) -> Result<Vec<Command>> {
    let value: serde_json::Value = serde_json::from_str(value).map_err(node_error)?;
    if value.is_array() {
        serde_json::from_value(value).map_err(node_error)
    } else {
        Ok(vec![serde_json::from_value(value).map_err(node_error)?])
    }
}

fn node_error(error: impl std::fmt::Display) -> Error {
    Error::new(Status::GenericFailure, error.to_string())
}
