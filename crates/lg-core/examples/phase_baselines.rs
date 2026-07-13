use base64::Engine;
use lg_core::{
    BlendMode, Command, Document, Layer, LayerCommon, LayerKind, LayerPatch, OutputFormat, RenderOptions,
    RetainedRenderer, Rgba, Sampling, Transform, execute_commands, load_kgfx_bytes, render_document, save_kgfx_bytes,
};
use serde::Serialize;
use std::hint::black_box;
use std::io::Cursor;
use std::time::{Duration, Instant};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    generated_at_unix_seconds: String,
    profile: &'static str,
    host: Host,
    measurements: Vec<Measurement>,
    peak_resident_bytes: Option<u64>,
}

#[derive(Serialize)]
struct Host {
    os: &'static str,
    arch: &'static str,
    logical_cpus: usize,
    cpu: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Measurement {
    category: &'static str,
    workload: &'static str,
    samples: usize,
    median_ms: f64,
    p95_ms: f64,
    throughput_per_second: f64,
    cache_hits: Option<u64>,
    cache_misses: Option<u64>,
    cache_evictions: Option<u64>,
    cache_bytes: Option<u64>,
}

fn main() {
    let mut measurements = Vec::new();

    let mut asset_doc = Document::new(256, 256, 72.0);
    execute_commands(
        &mut asset_doc,
        &[Command::AssetAdd {
            id: "payload".to_owned(),
            media_type: "application/octet-stream".to_owned(),
            bytes_base64: base64::engine::general_purpose::STANDARD.encode(vec![17_u8; 1024 * 1024]),
            original_name: Some("payload.bin".to_owned()),
            author: Some(serde_json::json!({ "tool": "phase-baselines" })),
        }],
    )
    .unwrap();
    let archive = save_kgfx_bytes(&asset_doc).unwrap();
    measurements.push(measure("phase1", "open-1mib-embedded", 15, || {
        black_box(load_kgfx_bytes(black_box(&archive)).unwrap());
    }));
    measurements.push(measure("phase1", "save-1mib-embedded", 10, || {
        black_box(save_kgfx_bytes(black_box(&asset_doc)).unwrap());
    }));
    if let Ok(cli) = std::env::var("LG_CLI") {
        measurements.push(measure("phase1", "cli-startup-version", 20, || {
            let output = std::process::Command::new(&cli).arg("--version").output().unwrap();
            assert!(output.status.success());
            black_box(output);
        }));
    }

    let mut shallow = fills(256, 256, 100);
    let shallow_ops: Vec<_> = (0..100)
        .map(|index| Command::LayerUpdate {
            id: format!("layer-{index}"),
            set: LayerPatch {
                opacity: Some(0.75),
                ..Default::default()
            },
        })
        .collect();
    measurements.push(measure("phase1", "execute-100-shallow-updates", 20, || {
        let mut value = shallow.clone();
        black_box(execute_commands(&mut value, &shallow_ops).unwrap());
    }));
    // Keep the allocation live so peak RSS includes a realistic document tree.
    black_box(&mut shallow);

    let deep = nested_document(220);
    measurements.push(measure("phase1", "execute-deep-leaf-update", 30, || {
        let mut value = deep.clone();
        black_box(
            execute_commands(
                &mut value,
                &[Command::LayerUpdate {
                    id: "deep-leaf".to_owned(),
                    set: LayerPatch {
                        x: Some(1.0),
                        ..Default::default()
                    },
                }],
            )
            .unwrap(),
        );
    }));

    for (workload, size, samples) in [
        ("reference-sprite-128", 128, 25),
        ("reference-2k", 2048, 5),
        ("reference-4k", 4096, 2),
    ] {
        let document = fills(size, size, 3);
        measurements.push(measure("phase1", workload, samples, || {
            black_box(render_document(&document, &RenderOptions::default()).unwrap());
        }));
    }

    for (workload, size, layers, samples) in [
        ("sprite-retained", 128, 40, 30),
        ("general-2k-retained", 2048, 6, 8),
        ("general-4k-retained", 4096, 3, 3),
        ("deep-220-retained", 512, 220, 10),
    ] {
        let document = if workload.starts_with("sprite") {
            image_batch_document(size, layers)
        } else if workload.starts_with("deep") {
            nested_document(layers)
        } else {
            fills(size, size, layers)
        };
        let mut renderer = RetainedRenderer::default();
        renderer.render(&document, &RenderOptions::default()).unwrap();
        let mut result = measure("phase2", workload, samples, || {
            black_box(renderer.render(&document, &RenderOptions::default()).unwrap());
        });
        let cache = renderer.metrics();
        result.cache_hits = Some(cache.cache_hits);
        result.cache_misses = Some(cache.cache_misses);
        result.cache_evictions = Some(cache.cache_evictions);
        result.cache_bytes = Some(cache.cache_bytes);
        measurements.push(result);
    }

    let mut sprite_changes = image_batch_document(128, 40);
    let mut sprite_renderer = RetainedRenderer::default();
    sprite_renderer
        .render(&sprite_changes, &RenderOptions::default())
        .unwrap();
    let mut visible = true;
    measurements.push(measure("phase2", "sprite-visibility-change", 30, || {
        visible = !visible;
        let result = execute_commands(
            &mut sprite_changes,
            &[Command::LayerUpdate {
                id: "image-0".to_owned(),
                set: LayerPatch {
                    visible: Some(visible),
                    ..Default::default()
                },
            }],
        )
        .unwrap();
        sprite_renderer.invalidate(&sprite_changes, &result);
        black_box(
            sprite_renderer
                .render(&sprite_changes, &RenderOptions::default())
                .unwrap(),
        );
    }));

    let mut deep_changes = nested_document(220);
    let mut deep_renderer = RetainedRenderer::default();
    deep_renderer.render(&deep_changes, &RenderOptions::default()).unwrap();
    let mut opacity = 1.0;
    measurements.push(measure("phase2", "deep-top-opacity-change", 12, || {
        opacity = if opacity == 1.0 { 0.8 } else { 1.0 };
        let result = execute_commands(
            &mut deep_changes,
            &[Command::LayerUpdate {
                id: "group-119".to_owned(),
                set: LayerPatch {
                    opacity: Some(opacity),
                    ..Default::default()
                },
            }],
        )
        .unwrap();
        deep_renderer.invalidate(&deep_changes, &result);
        black_box(deep_renderer.render(&deep_changes, &RenderOptions::default()).unwrap());
    }));
    measurements.push(measure("phase2", "deep-bottom-source-change", 3, || {
        let blue = deep_changes.manifest.revision as u8;
        let result = execute_commands(
            &mut deep_changes,
            &[Command::LayerUpdate {
                id: "deep-leaf".to_owned(),
                set: LayerPatch {
                    color: Some(Rgba(40, 180, blue, 255)),
                    ..Default::default()
                },
            }],
        )
        .unwrap();
        deep_renderer.invalidate(&deep_changes, &result);
        black_box(deep_renderer.render(&deep_changes, &RenderOptions::default()).unwrap());
    }));

    let batch = image_batch_document(256, 20);
    let mut cold_samples = Vec::new();
    let mut warm_samples = Vec::new();
    let mut retained = RetainedRenderer::default();
    for index in 0..32 {
        let options = RenderOptions {
            sampling: Sampling::Smooth,
            scale: [0.25, 0.5, 1.0][index % 3],
            ..Default::default()
        };
        cold_samples.push(timed(|| {
            let mut cold = RetainedRenderer::default();
            black_box(cold.render_encoded(&batch, &options, OutputFormat::Png).unwrap());
        }));
        warm_samples.push(timed(|| {
            black_box(retained.render_encoded(&batch, &options, OutputFormat::Png).unwrap());
        }));
    }
    measurements.push(from_samples("phase2", "batch-32-cold", cold_samples, None));
    let cache = retained.metrics();
    measurements.push(from_samples("phase2", "batch-32-warm", warm_samples, Some(cache)));

    let report = Report {
        generated_at_unix_seconds: unix_timestamp(),
        profile: if cfg!(debug_assertions) { "debug" } else { "release" },
        host: Host {
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            logical_cpus: std::thread::available_parallelism().map(usize::from).unwrap_or(1),
            cpu: cpu_name(),
        },
        measurements,
        peak_resident_bytes: peak_resident_bytes(),
    };
    let json = serde_json::to_string_pretty(&report).unwrap();
    let mut arguments = std::env::args().skip(1);
    if arguments.next().as_deref() == Some("--output") {
        let path = arguments.next().expect("--output requires a path");
        std::fs::write(path, format!("{json}\n")).unwrap();
    } else {
        println!("{json}");
    }
}

fn fills(width: u32, height: u32, count: usize) -> Document {
    let mut document = Document::new(width, height, 72.0);
    for index in 0..count {
        let inset = (index % 12) as f32;
        document.manifest.layers.push(Layer {
            common: LayerCommon {
                id: format!("layer-{index}"),
                name: format!("Layer {index}"),
                visible: true,
                opacity: 0.35 + (index % 4) as f32 * 0.15,
                blend_mode: if index % 5 == 0 {
                    BlendMode::Multiply
                } else {
                    BlendMode::Normal
                },
                transform: Transform {
                    x: inset,
                    y: inset,
                    scale_x: 1.0,
                    scale_y: 1.0,
                },
            },
            kind: LayerKind::Fill {
                width: width.saturating_sub(inset as u32 * 2).max(1),
                height: height.saturating_sub(inset as u32 * 2).max(1),
                color: Rgba((index * 47) as u8, (index * 83) as u8, (index * 131) as u8, 220),
            },
        });
    }
    document
}

fn image_batch_document(size: u32, count: usize) -> Document {
    let mut pixels = image::RgbaImage::new(size, size);
    for (x, y, pixel) in pixels.enumerate_pixels_mut() {
        *pixel = image::Rgba([(x ^ y) as u8, x.wrapping_mul(3) as u8, y.wrapping_mul(5) as u8, 220]);
    }
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(pixels)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    let mut document = Document::new(size, size, 72.0);
    execute_commands(
        &mut document,
        &[Command::AssetAdd {
            id: "shared-image".to_owned(),
            media_type: "image/png".to_owned(),
            bytes_base64: base64::engine::general_purpose::STANDARD.encode(png.into_inner()),
            original_name: Some("shared.png".to_owned()),
            author: None,
        }],
    )
    .unwrap();
    for index in 0..count {
        document.manifest.layers.push(Layer {
            common: LayerCommon {
                id: format!("image-{index}"),
                name: format!("Image {index}"),
                visible: true,
                opacity: 0.15,
                blend_mode: if index % 4 == 0 {
                    BlendMode::Multiply
                } else {
                    BlendMode::Normal
                },
                transform: Transform::default(),
            },
            kind: LayerKind::Image {
                asset_id: "shared-image".to_owned(),
            },
        });
    }
    document
}

fn nested_document(depth: usize) -> Document {
    let leaf = Layer {
        common: common("deep-leaf"),
        kind: LayerKind::Fill {
            width: 64,
            height: 64,
            color: Rgba(40, 180, 220, 255),
        },
    };
    let mut current = leaf;
    for index in 0..depth.min(120) {
        let sibling = Layer {
            common: common(&format!("deep-sibling-{index}")),
            kind: LayerKind::Fill {
                width: 8,
                height: 8,
                color: Rgba(index as u8, (index * 3) as u8, (index * 7) as u8, 80),
            },
        };
        current = Layer {
            common: common(&format!("group-{index}")),
            kind: LayerKind::Group {
                children: vec![current, sibling],
            },
        };
    }
    let mut document = Document::new(512, 512, 72.0);
    document.manifest.layers.push(current);
    document
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

fn measure(category: &'static str, workload: &'static str, samples: usize, mut work: impl FnMut()) -> Measurement {
    let mut times = Vec::with_capacity(samples);
    for _ in 0..samples {
        times.push(timed(&mut work));
    }
    from_samples(category, workload, times, None)
}

fn timed(mut work: impl FnMut()) -> Duration {
    let started = Instant::now();
    work();
    started.elapsed()
}

fn from_samples(
    category: &'static str,
    workload: &'static str,
    mut times: Vec<Duration>,
    cache: Option<lg_core::RetainedRenderMetrics>,
) -> Measurement {
    times.sort();
    let samples = times.len();
    let median_ms = millis(times[samples / 2]);
    let p95_ms = millis(
        times[((samples as f64 * 0.95).ceil() as usize)
            .saturating_sub(1)
            .min(samples - 1)],
    );
    Measurement {
        category,
        workload,
        samples,
        median_ms,
        p95_ms,
        throughput_per_second: if median_ms == 0.0 {
            f64::INFINITY
        } else {
            1000.0 / median_ms
        },
        cache_hits: cache.as_ref().map(|value| value.cache_hits),
        cache_misses: cache.as_ref().map(|value| value.cache_misses),
        cache_evictions: cache.as_ref().map(|value| value.cache_evictions),
        cache_bytes: cache.map(|value| value.cache_bytes),
    }
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}
fn unix_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

#[cfg(target_os = "linux")]
fn peak_resident_bytes() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmHWM:")?
                .split_whitespace()
                .next()?
                .parse::<u64>()
                .ok()
                .map(|kb| kb * 1024)
        })
}
#[cfg(not(target_os = "linux"))]
fn peak_resident_bytes() -> Option<u64> {
    None
}

#[cfg(target_os = "linux")]
fn cpu_name() -> Option<String> {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()?
        .lines()
        .find_map(|line| line.strip_prefix("model name\t: ").map(str::to_owned))
}
#[cfg(not(target_os = "linux"))]
fn cpu_name() -> Option<String> {
    None
}
