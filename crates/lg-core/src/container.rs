use crate::document::{AssetSource, Document, Manifest};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Cursor, Read, Seek, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;

const MAX_MANIFEST_BYTES: u64 = 16 * 1024 * 1024;
const MAX_ASSET_BYTES: u64 = 512 * 1024 * 1024;
const MAX_TOTAL_ASSET_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_ASSET_COUNT: usize = 10_000;

#[derive(Debug, thiserror::Error)]
pub enum KgfxError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid .kgfx archive: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("invalid manifest: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsafe or invalid document: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MigrationReport {
    pub from_version: u32,
    pub to_version: u32,
    pub steps: Vec<String>,
}

pub fn load_kgfx(path: impl AsRef<Path>) -> Result<Document, KgfxError> {
    let file = File::open(path)?;
    load_archive(file)
}

pub fn load_kgfx_bytes(bytes: &[u8]) -> Result<Document, KgfxError> {
    load_archive(Cursor::new(bytes))
}

fn load_archive<R: Read + Seek>(reader: R) -> Result<Document, KgfxError> {
    let mut archive = zip::ZipArchive::new(reader)?;
    let manifest: Manifest = {
        let mut entry = archive.by_name("manifest.json")?;
        if entry.size() > MAX_MANIFEST_BYTES {
            return Err(KgfxError::Invalid("manifest exceeds size limit".to_owned()));
        }
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut bytes)?;
        let mut value: serde_json::Value = serde_json::from_slice(&bytes)?;
        migrate_manifest_value(&mut value)?;
        serde_json::from_value(value)?
    };
    let mut asset_bytes = BTreeMap::new();
    if manifest.assets.len() > MAX_ASSET_COUNT {
        return Err(KgfxError::Invalid("asset count exceeds size limit".to_owned()));
    }
    let mut total_asset_bytes = 0_u64;
    for (id, asset) in &manifest.assets {
        if let AssetSource::Embedded { path } = &asset.source {
            if path.starts_with('/') || path.contains("..") || !path.starts_with("assets/") {
                return Err(KgfxError::Invalid(format!("unsafe embedded asset path '{path}'")));
            }
            let mut entry = archive.by_name(path)?;
            if entry.size() > MAX_ASSET_BYTES || entry.size() != asset.byte_length {
                return Err(KgfxError::Invalid(format!("invalid size for asset '{id}'")));
            }
            total_asset_bytes = total_asset_bytes.saturating_add(entry.size());
            if total_asset_bytes > MAX_TOTAL_ASSET_BYTES {
                return Err(KgfxError::Invalid("embedded assets exceed total size limit".to_owned()));
            }
            let mut bytes = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut bytes)?;
            let actual = format!("{:x}", Sha256::digest(&bytes));
            if actual != asset.sha256 {
                return Err(KgfxError::Invalid(format!("integrity check failed for asset '{id}'")));
            }
            asset_bytes.insert(id.clone(), bytes.into());
        }
    }
    let doc = Document { manifest, asset_bytes };
    let errors = doc.validate();
    if let Some(error) = errors.first() {
        return Err(KgfxError::Invalid(format!(
            "{} at {}: {}",
            error.code, error.path, error.message
        )));
    }
    Ok(doc)
}

pub fn save_kgfx(path: impl AsRef<Path>, doc: &Document) -> Result<(), KgfxError> {
    if let Some(error) = doc.validate().first() {
        return Err(KgfxError::Invalid(format!(
            "{} at {}: {}",
            error.code, error.path, error.message
        )));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let file = atomic_write_file::AtomicWriteFile::open(path)?;
        let mut file = write_archive(file, doc)?;
        file.flush()?;
        file.commit()?;
        Ok(())
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = path;
        Err(KgfxError::Invalid(
            "filesystem persistence is unavailable in WebAssembly; use save_kgfx_bytes".to_owned(),
        ))
    }
}

pub fn save_kgfx_bytes(doc: &Document) -> Result<Vec<u8>, KgfxError> {
    if let Some(error) = doc.validate().first() {
        return Err(KgfxError::Invalid(format!(
            "{} at {}: {}",
            error.code, error.path, error.message
        )));
    }
    let cursor = write_archive(Cursor::new(Vec::new()), doc)?;
    Ok(cursor.into_inner())
}

fn write_archive<W: Write + Seek>(writer: W, doc: &Document) -> Result<W, KgfxError> {
    let mut zip = zip::ZipWriter::new(writer);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("manifest.json", options)?;
    zip.write_all(&serde_json::to_vec_pretty(&doc.manifest)?)?;
    let mut written_paths = BTreeMap::<String, String>::new();
    for (id, asset) in &doc.manifest.assets {
        if let AssetSource::Embedded { path } = &asset.source {
            if let Some(previous_id) = written_paths.get(path) {
                let current = doc
                    .asset_bytes
                    .get(id)
                    .ok_or_else(|| KgfxError::Invalid(format!("missing bytes for asset '{id}'")))?;
                let previous = doc.asset_bytes.get(previous_id).expect("previous asset bytes exist");
                if current != previous {
                    return Err(KgfxError::Invalid(format!(
                        "assets '{previous_id}' and '{id}' share a path but not bytes"
                    )));
                }
                continue;
            }
            let bytes = doc
                .asset_bytes
                .get(id)
                .ok_or_else(|| KgfxError::Invalid(format!("missing bytes for asset '{id}'")))?;
            zip.start_file(path, options)?;
            zip.write_all(bytes)?;
            written_paths.insert(path.clone(), id.clone());
        }
    }
    Ok(zip.finish()?)
}

pub fn migrate_manifest_value(value: &mut serde_json::Value) -> Result<Option<MigrationReport>, KgfxError> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| KgfxError::Invalid("manifest root must be an object".to_owned()))?;
    let version = object
        .get("schemaVersion")
        .or_else(|| object.get("schema_version"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as u32;
    if version > crate::document::SCHEMA_VERSION {
        return Err(KgfxError::Invalid(format!(
            "unsupported future schema version {version}"
        )));
    }
    if version == crate::document::SCHEMA_VERSION {
        return Ok(None);
    }
    if version != 0 {
        return Err(KgfxError::Invalid(format!(
            "no migration path from schema version {version}"
        )));
    }
    if let Some(canvas) = object.get_mut("canvas").and_then(serde_json::Value::as_object_mut) {
        if let Some(width) = canvas.remove("w") {
            canvas.entry("width").or_insert(width);
        }
        if let Some(height) = canvas.remove("h") {
            canvas.entry("height").or_insert(height);
        }
        canvas.entry("dpi").or_insert(serde_json::json!(72.0));
        canvas.entry("color").or_insert(serde_json::json!([0, 0, 0, 0]));
    }
    object.remove("schema_version");
    object.insert(
        "schemaVersion".to_owned(),
        serde_json::json!(crate::document::SCHEMA_VERSION),
    );
    object.entry("revision").or_insert(serde_json::json!(0));
    object.entry("layers").or_insert(serde_json::json!([]));
    object.entry("assets").or_insert(serde_json::json!({}));
    object.entry("extensions").or_insert(serde_json::json!({}));
    Ok(Some(MigrationReport {
        from_version: 0,
        to_version: crate::document::SCHEMA_VERSION,
        steps: vec!["normalize canvas keys and add versioned defaults".to_owned()],
    }))
}
