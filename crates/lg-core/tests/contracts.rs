use base64::Engine;
use lg_core::{
    BlendMode, ChangeImpact, Command, Document, Layer, LayerCommon, LayerKind, OutputFormat, RenderOptions,
    RetainedRenderer, Rgba, Transform, command_schema, document_schema, execute_commands, load_kgfx, load_kgfx_bytes,
    migrate_manifest_value, render_document, render_document_encoded, save_kgfx, save_kgfx_bytes,
};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Write};

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
fn reducer_errors_identify_the_failing_command_index() {
    let mut document = Document::new(8, 8, 72.0);
    let layer = Layer {
        common: common("duplicate"),
        kind: LayerKind::Fill {
            width: 1,
            height: 1,
            color: Rgba(0, 0, 0, 0),
        },
    };
    let error = execute_commands(
        &mut document,
        &[
            Command::LayerAdd {
                layer: layer.clone(),
                parent_id: None,
                index: None,
            },
            Command::LayerAdd {
                layer,
                parent_id: None,
                index: None,
            },
        ],
    )
    .unwrap_err();
    assert!(error.to_string().contains("command 1"));
    assert!(document.manifest.layers.is_empty());
}

#[test]
fn omitted_layer_id_is_generated_and_returned() {
    let mut document = Document::new(8, 8, 72.0);
    let command: Command = serde_json::from_value(serde_json::json!({
        "op": "layerAdd",
        "layer": { "name": "Generated", "type": "fill", "width": 1, "height": 1, "color": [1,2,3,255] }
    }))
    .unwrap();
    let result = execute_commands(&mut document, &[command]).unwrap();
    assert_eq!(result.changed_layers.len(), 1);
    assert!(!result.changed_layers[0].is_empty());
    assert_eq!(result.changes[0].layer_ids, result.changed_layers);
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
            author: None,
        }],
    )
    .unwrap();

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("roundtrip.kgfx");
    save_kgfx(&path, &document).unwrap();
    let loaded = load_kgfx(&path).unwrap();
    assert_eq!(loaded.manifest, document.manifest);
    assert_eq!(loaded.asset_bytes.get("fixture").unwrap().as_slice(), payload);
    let memory = save_kgfx_bytes(&loaded).unwrap();
    assert_eq!(load_kgfx_bytes(&memory).unwrap(), loaded);
}

#[test]
fn identical_asset_bytes_deduplicate_payload_without_merging_ids() {
    let mut document = Document::new(8, 8, 72.0);
    let bytes = base64::engine::general_purpose::STANDARD.encode(b"same payload");
    execute_commands(
        &mut document,
        &[
            Command::AssetAdd {
                id: "logical-a".to_owned(),
                media_type: "application/octet-stream".to_owned(),
                bytes_base64: bytes.clone(),
                original_name: Some("a.bin".to_owned()),
                author: Some(serde_json::json!({"author": "A"})),
            },
            Command::AssetAdd {
                id: "logical-b".to_owned(),
                media_type: "application/octet-stream".to_owned(),
                bytes_base64: bytes,
                original_name: Some("b.bin".to_owned()),
                author: None,
            },
        ],
    )
    .unwrap();
    let packed = save_kgfx_bytes(&document).unwrap();
    let zip = zip::ZipArchive::new(Cursor::new(&packed)).unwrap();
    assert_eq!(zip.len(), 2, "manifest plus one shared content payload");
    let loaded = load_kgfx_bytes(&packed).unwrap();
    assert!(loaded.manifest.assets.contains_key("logical-a"));
    assert!(loaded.manifest.assets.contains_key("logical-b"));
    assert_eq!(
        loaded.manifest.assets["logical-a"].author,
        Some(serde_json::json!({"author": "A"}))
    );
}

#[test]
fn migrates_schema_zero_manifest() {
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/migrations/v0-manifest.json")).unwrap();
    let report = migrate_manifest_value(&mut value).unwrap().unwrap();
    assert_eq!(report.from_version, 0);
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["canvas"]["width"], 32);
    assert_eq!(value["canvas"]["height"], 16);
    assert_eq!(value["canvas"]["dpi"], 72.0);
    assert_eq!(value["canvas"]["color"], serde_json::json!([0, 0, 0, 0]));
    assert_eq!(value["extensions"]["com.layeredgraphics.fixture"]["preserved"], true);
}

#[test]
fn checked_in_schemas_match_the_authoritative_rust_types() {
    let document: serde_json::Value =
        serde_json::from_str(include_str!("../../../spec/document/v1.schema.json")).unwrap();
    let commands: serde_json::Value =
        serde_json::from_str(include_str!("../../../spec/commands/v1.schema.json")).unwrap();
    assert_eq!(document, serde_json::to_value(document_schema()).unwrap());
    assert_eq!(commands, serde_json::to_value(command_schema()).unwrap());
}

#[test]
fn public_examples_validate_against_checked_in_json_schemas() {
    let command_schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../spec/commands/v1.schema.json")).unwrap();
    let document_schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../spec/document/v1.schema.json")).unwrap();
    let operations: serde_json::Value =
        serde_json::from_str(include_str!("../../../apps/site/public/api-equivalence.ops.json")).unwrap();
    assert!(
        jsonschema::validator_for(&command_schema)
            .unwrap()
            .is_valid(&operations)
    );
    let linked = serde_json::json!([{
        "op": "assetLink", "id": "linked", "mediaType": "image/png", "reference": "memory://linked",
        "byteLength": 4, "sha256": "abcd", "originalName": "linked.png"
    }]);
    assert!(jsonschema::validator_for(&command_schema).unwrap().is_valid(&linked));
    let parsed: Vec<Command> = serde_json::from_value(linked).unwrap();
    assert!(
        matches!(parsed[0], Command::AssetLink { ref media_type, byte_length: 4, .. } if media_type == "image/png")
    );
    let legacy_snake: Command = serde_json::from_value(serde_json::json!({
        "op": "assetAdd", "id": "legacy", "media_type": "application/octet-stream", "bytes_base64": "", "original_name": "old.bin"
    })).unwrap();
    assert!(matches!(legacy_snake, Command::AssetAdd { .. }));
    let manifest = serde_json::to_value(Document::new(32, 16, 72.0).manifest).unwrap();
    assert!(jsonschema::validator_for(&document_schema).unwrap().is_valid(&manifest));
}

#[test]
fn hostile_archives_and_future_versions_fail_without_large_allocations() {
    assert!(load_kgfx_bytes(b"not a zip archive").is_err());
    let future = archive_with_manifest(
        serde_json::json!({
            "schemaVersion": 999, "id": "future", "revision": 0,
            "canvas": { "width": 1, "height": 1, "dpi": 72, "color": [0,0,0,0] },
            "layers": [], "assets": {}, "extensions": {}
        }),
        &[],
    );
    assert!(
        load_kgfx_bytes(&future)
            .unwrap_err()
            .to_string()
            .contains("future schema")
    );

    let unsafe_path = archive_with_manifest(
        serde_json::json!({
            "schemaVersion": 1, "id": "unsafe", "revision": 0,
            "canvas": { "width": 1, "height": 1, "dpi": 72, "color": [0,0,0,0] },
            "layers": [],
            "assets": { "bad": { "id": "bad", "mediaType": "application/octet-stream", "byteLength": 0,
              "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
              "source": { "type": "embedded", "path": "../escape" } } },
            "extensions": {}
        }),
        &[],
    );
    assert!(
        load_kgfx_bytes(&unsafe_path)
            .unwrap_err()
            .to_string()
            .contains("unsafe embedded asset path")
    );

    let oversized = archive_with_manifest(
        serde_json::json!({
            "schemaVersion": 1, "id": "oversized", "revision": 0,
            "canvas": { "width": 1, "height": 1, "dpi": 72, "color": [0,0,0,0] },
            "layers": [],
            "assets": { "huge": { "id": "huge", "mediaType": "application/octet-stream", "byteLength": 536870913_u64,
              "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
              "source": { "type": "embedded", "path": "assets/huge" } } },
            "extensions": {}
        }),
        &[("assets/huge", &[])],
    );
    assert!(
        load_kgfx_bytes(&oversized)
            .unwrap_err()
            .to_string()
            .contains("invalid size")
    );
}

#[test]
fn rejected_safe_save_preserves_the_previous_valid_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("safe.kgfx");
    let valid = Document::new(8, 8, 72.0);
    save_kgfx(&path, &valid).unwrap();
    let before = std::fs::read(&path).unwrap();
    let mut invalid = valid;
    invalid.manifest.canvas.width = 0;
    assert!(save_kgfx(&path, &invalid).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

fn archive_with_manifest(value: serde_json::Value, entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    archive.start_file("manifest.json", options).unwrap();
    archive.write_all(&serde_json::to_vec(&value).unwrap()).unwrap();
    for (name, bytes) in entries {
        archive.start_file(*name, options).unwrap();
        archive.write_all(bytes).unwrap();
    }
    archive.finish().unwrap().into_inner()
}

#[test]
fn linked_asset_resolution_checks_length_and_integrity() {
    let bytes = b"linked bytes";
    let mut document = Document::new(8, 8, 72.0);
    execute_commands(
        &mut document,
        &[Command::AssetLink {
            id: "linked".to_owned(),
            media_type: "application/octet-stream".to_owned(),
            reference: "workspace://linked".to_owned(),
            byte_length: bytes.len() as u64,
            sha256: hex::encode(Sha256::digest(bytes)),
            original_name: None,
            author: None,
        }],
    )
    .unwrap();
    assert!(!document.asset_bytes.contains_key("linked"));
    assert_eq!(
        document
            .resolve_linked_assets(|reference| {
                assert_eq!(reference, "workspace://linked");
                Ok(bytes.to_vec())
            })
            .unwrap(),
        1
    );
    assert_eq!(document.asset_bytes["linked"].as_slice(), bytes);
}

#[test]
fn command_results_classify_metadata_and_local_invalidations() {
    let mut document = Document::new(64, 64, 72.0);
    execute_commands(
        &mut document,
        &[Command::LayerAdd {
            layer: Layer {
                common: common("fill"),
                kind: LayerKind::Fill {
                    width: 8,
                    height: 8,
                    color: Rgba(1, 2, 3, 255),
                },
            },
            parent_id: None,
            index: None,
        }],
    )
    .unwrap();
    let renamed = execute_commands(
        &mut document,
        &[Command::LayerUpdate {
            id: "fill".to_owned(),
            set: lg_core::LayerPatch {
                name: Some("Renamed".to_owned()),
                ..Default::default()
            },
        }],
    )
    .unwrap();
    assert_eq!(renamed.changes[0].impact, ChangeImpact::Metadata);
    assert!(!renamed.changes[0].full_render);
    let moved = execute_commands(
        &mut document,
        &[Command::LayerUpdate {
            id: "fill".to_owned(),
            set: lg_core::LayerPatch {
                x: Some(10.0),
                ..Default::default()
            },
        }],
    )
    .unwrap();
    assert_eq!(moved.changes[0].impact, ChangeImpact::LocalPixels);
    assert_eq!(moved.changes[0].regions.len(), 2);
}

#[test]
fn authoritative_renderer_encodes_all_phase_one_formats() {
    let mut document = Document::new(4, 4, 72.0);
    document.manifest.layers.push(Layer {
        common: common("fill"),
        kind: LayerKind::Fill {
            width: 4,
            height: 4,
            color: Rgba(20, 40, 60, 255),
        },
    });
    for (format, magic) in [
        (OutputFormat::Png, &b"\x89PNG"[..]),
        (OutputFormat::Jpeg, &b"\xff\xd8\xff"[..]),
        (OutputFormat::Webp, &b"RIFF"[..]),
    ] {
        let encoded = render_document_encoded(
            &document,
            &RenderOptions {
                sampling: lg_core::Sampling::Smooth,
                ..Default::default()
            },
            format,
        )
        .unwrap();
        assert!(encoded.starts_with(magic));
    }
}

#[test]
fn reference_renderer_honors_background_layer_selection_and_pixel_sampling() {
    let mut document = Document::new(2, 1, 72.0);
    document.manifest.canvas.color = Rgba(9, 8, 7, 255);
    let mut offset = common("one-pixel");
    offset.transform.x = 1.0;
    document.manifest.layers.push(Layer {
        common: offset,
        kind: LayerKind::Fill {
            width: 1,
            height: 1,
            color: Rgba(255, 0, 0, 255),
        },
    });
    let selected = render_document(
        &document,
        &RenderOptions {
            layer_id: Some("one-pixel".to_owned()),
            scale: 2.0,
            sampling: lg_core::Sampling::Nearest,
            background: None,
        },
    )
    .unwrap();
    assert_eq!(selected.dimensions(), (4, 2));
    assert_eq!(selected.get_pixel(0, 0).0, [9, 8, 7, 255]);
    assert_eq!(selected.get_pixel(1, 0).0, [9, 8, 7, 255]);
    assert_eq!(selected.get_pixel(2, 0).0, [255, 0, 0, 255]);
    assert_eq!(selected.get_pixel(3, 1).0, [255, 0, 0, 255]);
}

#[test]
fn retained_rendering_reuses_sources_and_matches_cold_output_after_changes() {
    let mut document = Document::new(32, 32, 72.0);
    for (id, color) in [("bottom", Rgba(20, 40, 60, 255)), ("top", Rgba(200, 80, 40, 180))] {
        document.manifest.layers.push(Layer {
            common: common(id),
            kind: LayerKind::Fill {
                width: 16,
                height: 16,
                color,
            },
        });
    }
    let options = RenderOptions::default();
    let mut retained = RetainedRenderer::new(1024 * 1024);
    assert_eq!(
        retained.render(&document, &options).unwrap(),
        render_document(&document, &options).unwrap()
    );
    assert_eq!(retained.metrics().cache_misses, 2);
    retained.render(&document, &options).unwrap();
    assert_eq!(retained.metrics().cache_hits, 2);

    let moved = execute_commands(
        &mut document,
        &[Command::LayerUpdate {
            id: "top".to_owned(),
            set: lg_core::LayerPatch {
                x: Some(8.0),
                ..Default::default()
            },
        }],
    )
    .unwrap();
    retained.invalidate(&document, &moved);
    assert_eq!(
        retained.render(&document, &options).unwrap(),
        render_document(&document, &options).unwrap()
    );
    assert_eq!(
        retained.metrics().cache_misses,
        2,
        "transform retained both source rasters"
    );

    let recolored = execute_commands(
        &mut document,
        &[Command::LayerUpdate {
            id: "top".to_owned(),
            set: lg_core::LayerPatch {
                color: Some(Rgba(1, 2, 3, 255)),
                ..Default::default()
            },
        }],
    )
    .unwrap();
    retained.invalidate(&document, &recolored);
    assert_eq!(
        retained.render(&document, &options).unwrap(),
        render_document(&document, &options).unwrap()
    );
    assert_eq!(
        retained.metrics().cache_misses,
        3,
        "only the changed source rerasterized"
    );
}

#[test]
fn incremental_output_matches_cold_after_deterministic_randomized_transactions() {
    let mut document = Document::new(48, 48, 72.0);
    for index in 0..8 {
        document.manifest.layers.push(Layer {
            common: common(&format!("random-{index}")),
            kind: LayerKind::Fill {
                width: 12 + index,
                height: 10 + index,
                color: Rgba((index * 20) as u8, (255 - index * 20) as u8, (index * 7) as u8, 180),
            },
        });
    }
    let mut retained = RetainedRenderer::default();
    let options = RenderOptions::default();
    let mut state = 0x5eed_u64;
    for step in 0..64 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let index = ((state >> 32) as usize) % 8;
        let set = match step % 3 {
            0 => lg_core::LayerPatch {
                x: Some(((state >> 16) % 30) as f32),
                y: Some(((state >> 24) % 30) as f32),
                ..Default::default()
            },
            1 => lg_core::LayerPatch {
                opacity: Some(0.2 + ((state >> 8) % 80) as f32 / 100.0),
                ..Default::default()
            },
            _ => lg_core::LayerPatch {
                color: Some(Rgba(state as u8, (state >> 8) as u8, (state >> 16) as u8, 200)),
                ..Default::default()
            },
        };
        let result = execute_commands(
            &mut document,
            &[Command::LayerUpdate {
                id: format!("random-{index}"),
                set,
            }],
        )
        .unwrap();
        retained.invalidate(&document, &result);
        assert_eq!(
            retained.render(&document, &options).unwrap(),
            render_document(&document, &options).unwrap(),
            "incremental output diverged at transaction {step}",
        );
    }
}

#[test]
fn retained_cache_obeys_its_memory_budget() {
    let mut document = Document::new(64, 64, 72.0);
    for index in 0..4 {
        document.manifest.layers.push(Layer {
            common: common(&format!("fill-{index}")),
            kind: LayerKind::Fill {
                width: 32,
                height: 32,
                color: Rgba(index, index, index, 255),
            },
        });
    }
    let mut retained = RetainedRenderer::new(4096);
    retained.render(&document, &RenderOptions::default()).unwrap();
    assert!(retained.metrics().cache_bytes <= 4096);
    assert!(retained.metrics().cache_evictions >= 3);
}

#[test]
fn retained_renderer_rebuilds_if_an_invalidation_was_missed() {
    let mut document = Document::new(4, 4, 72.0);
    document.manifest.layers.push(Layer {
        common: common("fill"),
        kind: LayerKind::Fill {
            width: 4,
            height: 4,
            color: Rgba(1, 2, 3, 255),
        },
    });
    let mut retained = RetainedRenderer::default();
    retained.render(&document, &RenderOptions::default()).unwrap();
    execute_commands(
        &mut document,
        &[Command::LayerUpdate {
            id: "fill".to_owned(),
            set: lg_core::LayerPatch {
                color: Some(Rgba(9, 8, 7, 255)),
                ..Default::default()
            },
        }],
    )
    .unwrap();
    let rebuilt = retained.render(&document, &RenderOptions::default()).unwrap();
    assert_eq!(rebuilt, render_document(&document, &RenderOptions::default()).unwrap());
    assert!(retained.metrics().cache_evictions >= 1);
}

#[test]
fn checked_in_webgpu_shaders_parse_and_validate() {
    let source = include_str!("../../../packages/browser/src/presenter.ts");
    for name in ["SHADER", "COMPOSITE_SHADER"] {
        let marker = format!("const {name} = `");
        let shader = source.split_once(&marker).unwrap().1.split_once("`;\n").unwrap().0;
        let module = naga::front::wgsl::parse_str(shader).unwrap_or_else(|error| panic!("{name}: {error}"));
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
            .validate(&module)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }
}
