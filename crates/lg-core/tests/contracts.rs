use base64::Engine;
use lg_core::{
    BlendMode, Command, Document, Layer, LayerCommon, LayerKind, Rgba, Transform, execute_commands, load_kgfx,
    save_kgfx,
};

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

#[test]
fn invalid_transaction_is_atomic() {
    let mut document = Document::new(64, 64, 72.0);
    let original = document.clone();
    let result = execute_commands(
        &mut document,
        &[
            Command::LayerAdd {
                layer: Layer {
                    common: common("background"),
                    kind: LayerKind::Fill {
                        width: 64,
                        height: 64,
                        color: Rgba(10, 20, 30, 255),
                    },
                },
                parent_id: None,
                index: None,
            },
            Command::LayerAdd {
                layer: Layer {
                    common: common("broken"),
                    kind: LayerKind::Image {
                        asset_id: "missing".to_owned(),
                    },
                },
                parent_id: None,
                index: None,
            },
        ],
    );
    assert!(result.is_err());
    assert_eq!(document, original);
}

#[test]
fn kgfx_round_trip_preserves_manifest_extensions_and_asset_bytes() {
    let mut document = Document::new(32, 16, 144.0);
    document
        .manifest
        .extensions
        .insert("com.example.fixture".to_owned(), serde_json::json!({ "answer": 42 }));
    let payload = b"fixture asset bytes";
    execute_commands(
        &mut document,
        &[Command::AssetAdd {
            id: "fixture".to_owned(),
            media_type: "application/octet-stream".to_owned(),
            bytes_base64: base64::engine::general_purpose::STANDARD.encode(payload),
            original_name: Some("fixture.bin".to_owned()),
        }],
    )
    .unwrap();

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("roundtrip.kgfx");
    save_kgfx(&path, &document).unwrap();
    let loaded = load_kgfx(&path).unwrap();
    assert_eq!(loaded.manifest, document.manifest);
    assert_eq!(loaded.asset_bytes.get("fixture").unwrap(), payload);
}
