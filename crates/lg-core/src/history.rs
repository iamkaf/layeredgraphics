use crate::{Command, CommandError, CommandResult, Document, execute_commands};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct DocumentSession {
    document: Document,
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    max_entries: usize,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub label: String,
    pub before: Document,
    pub after: Document,
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoryState {
    pub undo_count: usize,
    pub redo_count: usize,
    pub undo_label: Option<String>,
    pub redo_label: Option<String>,
    pub revision: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error("nothing to undo")]
    NothingToUndo,
    #[error("nothing to redo")]
    NothingToRedo,
}

impl DocumentSession {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            undo: Vec::new(),
            redo: Vec::new(),
            max_entries: 100,
        }
    }

    pub fn with_history_limit(document: Document, max_entries: usize) -> Self {
        Self {
            document,
            undo: Vec::new(),
            redo: Vec::new(),
            max_entries,
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut Document {
        &mut self.document
    }

    pub fn into_document(self) -> Document {
        self.document
    }

    pub fn execute(&mut self, label: impl Into<String>, commands: &[Command]) -> Result<CommandResult, HistoryError> {
        let before = self.document.clone();
        let result = execute_commands(&mut self.document, commands)?;
        if !commands.is_empty() && self.max_entries > 0 {
            self.undo.push(HistoryEntry {
                label: label.into(),
                before,
                after: self.document.clone(),
                commands: commands.to_vec(),
            });
            if self.undo.len() > self.max_entries {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
        Ok(result)
    }

    pub fn undo(&mut self) -> Result<HistoryState, HistoryError> {
        let entry = self.undo.pop().ok_or(HistoryError::NothingToUndo)?;
        self.document = entry.before.clone();
        self.redo.push(entry);
        Ok(self.state())
    }

    pub fn redo(&mut self) -> Result<HistoryState, HistoryError> {
        let entry = self.redo.pop().ok_or(HistoryError::NothingToRedo)?;
        self.document = entry.after.clone();
        self.undo.push(entry);
        Ok(self.state())
    }

    pub fn state(&self) -> HistoryState {
        HistoryState {
            undo_count: self.undo.len(),
            redo_count: self.redo.len(),
            undo_label: self.undo.last().map(|entry| entry.label.clone()),
            redo_label: self.redo.last().map(|entry| entry.label.clone()),
            revision: self.document.manifest.revision,
        }
    }
}

pub fn diff_documents(source: &Document, target: &Document) -> Vec<Command> {
    if source == target {
        return Vec::new();
    }
    let mut commands = Vec::new();
    if source.manifest.canvas != target.manifest.canvas {
        commands.push(Command::DocumentUpdate {
            width: Some(target.manifest.canvas.width),
            height: Some(target.manifest.canvas.height),
            dpi: Some(target.manifest.canvas.dpi),
            color: Some(target.manifest.canvas.color),
        });
    }
    for layer in source.manifest.layers.iter().rev() {
        commands.push(Command::LayerRemove {
            id: layer.common.id.clone(),
        });
    }
    for id in source.manifest.assets.keys() {
        if !target.manifest.assets.contains_key(id) || source.manifest.assets.get(id) != target.manifest.assets.get(id)
        {
            commands.push(Command::AssetRemove { id: id.clone() });
        }
    }
    for (id, asset) in &target.manifest.assets {
        if source.manifest.assets.get(id) == Some(asset) {
            continue;
        }
        match &asset.source {
            crate::AssetSource::Embedded { .. } => {
                if let Some(bytes) = target.asset_bytes.get(id) {
                    use base64::Engine;
                    commands.push(Command::AssetAdd {
                        id: id.clone(),
                        media_type: asset.media_type.clone(),
                        bytes_base64: base64::engine::general_purpose::STANDARD.encode(bytes.as_slice()),
                        original_name: asset.original_name.clone(),
                        author: asset.author.clone(),
                    });
                }
            }
            crate::AssetSource::Linked { reference } => commands.push(Command::AssetLink {
                id: id.clone(),
                media_type: asset.media_type.clone(),
                reference: reference.clone(),
                byte_length: asset.byte_length,
                sha256: asset.sha256.clone(),
                original_name: asset.original_name.clone(),
                author: asset.author.clone(),
            }),
        }
    }
    for namespace in source.manifest.extensions.keys() {
        if !target.manifest.extensions.contains_key(namespace) {
            commands.push(Command::ExtensionRemove {
                namespace: namespace.clone(),
            });
        }
    }
    for (namespace, value) in &target.manifest.extensions {
        if source.manifest.extensions.get(namespace) != Some(value) {
            commands.push(Command::ExtensionSet {
                namespace: namespace.clone(),
                value: value.clone(),
            });
        }
    }
    for (index, layer) in target.manifest.layers.iter().enumerate() {
        commands.push(Command::LayerAdd {
            layer: layer.clone(),
            parent_id: None,
            index: Some(index),
        });
    }
    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BlendMode, Layer, LayerCommon, LayerKind, Rgba, Transform};

    fn fill(id: &str, color: Rgba) -> Layer {
        Layer {
            common: LayerCommon {
                id: id.to_owned(),
                name: id.to_owned(),
                visible: true,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                transform: Transform::default(),
            },
            kind: LayerKind::Fill {
                width: 8,
                height: 8,
                color,
            },
        }
    }

    #[test]
    fn undo_and_redo_follow_transaction_boundaries() {
        let mut session = DocumentSession::new(Document::new(8, 8, 72.0));
        session
            .execute(
                "Add fill",
                &[Command::LayerAdd {
                    layer: fill("fill", Rgba(1, 2, 3, 255)),
                    parent_id: None,
                    index: None,
                }],
            )
            .unwrap();
        assert!(session.document().find_layer("fill").is_some());
        session.undo().unwrap();
        assert!(session.document().find_layer("fill").is_none());
        session.redo().unwrap();
        assert!(session.document().find_layer("fill").is_some());
    }

    #[test]
    fn diff_transforms_supported_graphical_state() {
        let source = Document::new(8, 8, 72.0);
        let mut target = source.clone();
        target.manifest.canvas.dpi = 144.0;
        target.manifest.layers.push(fill("fill", Rgba(9, 8, 7, 255)));
        let commands = diff_documents(&source, &target);
        let mut transformed = source.clone();
        execute_commands(&mut transformed, &commands).unwrap();
        assert_eq!(transformed.manifest.canvas, target.manifest.canvas);
        assert_eq!(transformed.manifest.layers, target.manifest.layers);
    }
}
