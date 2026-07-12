use crate::document::{AssetSource, Document, Manifest};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Cursor, Read, Seek, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

const MAX_MANIFEST_BYTES: u64 = 16 * 1024 * 1024;
const MAX_ASSET_BYTES: u64 = 512 * 1024 * 1024;

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
        serde_json::from_slice(&bytes)?
    };
    let mut asset_bytes = BTreeMap::new();
    for (id, asset) in &manifest.assets {
        if let AssetSource::Embedded { path } = &asset.source {
            if path.starts_with('/') || path.contains("..") || !path.starts_with("assets/") {
                return Err(KgfxError::Invalid(format!("unsafe embedded asset path '{path}'")));
            }
            let mut entry = archive.by_name(path)?;
            if entry.size() > MAX_ASSET_BYTES || entry.size() != asset.byte_length {
                return Err(KgfxError::Invalid(format!("invalid size for asset '{id}'")));
            }
            let mut bytes = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut bytes)?;
            let actual = format!("{:x}", Sha256::digest(&bytes));
            if actual != asset.sha256 {
                return Err(KgfxError::Invalid(format!("integrity check failed for asset '{id}'")));
            }
            asset_bytes.insert(id.clone(), bytes);
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
    let path = path.as_ref();
    let temp = temporary_path(path);
    let file = File::create(&temp)?;
    let mut zip = zip::ZipWriter::new(file);
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
    let mut file = zip.finish()?;
    file.flush()?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temp, path).inspect_err(|_| {
        let _ = fs::remove_file(&temp);
    })?;
    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut value = path.as_os_str().to_owned();
    value.push(format!(".{}.tmp", std::process::id()));
    PathBuf::from(value)
}
