use assert_cmd::Command;
use std::fs;

fn lg() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("lg")
}

#[test]
fn complete_milestone_cli_surface_operates_on_real_files() {
    let directory = tempfile::tempdir().unwrap();
    let doc = directory.path().join("fixture.kgfx");
    let asset = directory.path().join("asset.bin");
    let ops = directory.path().join("ops.json");
    let output = directory.path().join("render.png");
    let jpeg = directory.path().join("render.jpg");
    let webp = directory.path().join("render.webp");
    let linked_copy = directory.path().join("linked-copy.png");
    let target = directory.path().join("target.kgfx");
    let transformed = directory.path().join("transformed.kgfx");
    let diff = directory.path().join("diff.json");
    let watched = directory.path().join("watched.png");
    fs::write(&asset, b"asset payload").unwrap();
    fs::write(&ops, r#"{"op":"documentUpdate","dpi":144}"#).unwrap();

    lg().args(["new", doc.to_str().unwrap(), "--width", "64", "--height", "32"])
        .assert()
        .success();
    lg().args([
        "asset",
        "add",
        doc.to_str().unwrap(),
        "--id",
        "unused",
        asset.to_str().unwrap(),
    ])
    .assert()
    .success();
    lg().args(["asset", "ls", doc.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("unused"));
    lg().args(["asset", "rm", doc.to_str().unwrap(), "unused"])
        .assert()
        .success();

    lg().args([
        "layer",
        "add",
        doc.to_str().unwrap(),
        "--type",
        "fill",
        "--id",
        "bottom",
        "--width",
        "64",
        "--height",
        "32",
        "--color",
        "#223344ff",
    ])
    .assert()
    .success();
    lg().args([
        "layer",
        "add",
        doc.to_str().unwrap(),
        "--type",
        "group",
        "--id",
        "group",
    ])
    .assert()
    .success();
    lg().args([
        "layer",
        "add",
        doc.to_str().unwrap(),
        "--type",
        "fill",
        "--id",
        "top",
        "--parent",
        "group",
        "--width",
        "16",
        "--height",
        "16",
        "--color",
        "#ff00ffff",
        "--x",
        "8",
        "--y",
        "8",
    ])
    .assert()
    .success();
    lg().args([
        "layer",
        "update",
        doc.to_str().unwrap(),
        "top",
        "--set",
        "opacity=0.7",
        "--set",
        "blend=multiply",
    ])
    .assert()
    .success();
    lg().args(["layer", "move", doc.to_str().unwrap(), "group", "--above", "bottom"])
        .assert()
        .success();
    lg().args(["layer", "ls", doc.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("multiply"));

    lg().args(["exec", doc.to_str().unwrap(), ops.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("revision"));
    lg().args(["inspect", doc.to_str().unwrap(), "--path", "canvas.dpi"])
        .assert()
        .success()
        .stdout("144.0\n");
    lg().args(["inspect", doc.to_str().unwrap(), "--path", "layers.0.opacity"])
        .assert()
        .success()
        .stdout("1.0\n");
    lg().args(["validate", doc.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"valid\": true"));
    lg().args(["validate", ops.to_str().unwrap()])
        .assert()
        .success()
        .stdout("valid\n");
    lg().args(["render", doc.to_str().unwrap(), "-o", output.to_str().unwrap()])
        .assert()
        .success();
    let image = image::open(&output).unwrap();
    assert_eq!((image.width(), image.height()), (64, 32));
    lg().args([
        "render",
        doc.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--layer",
        "bottom",
        "--scale",
        "2",
    ])
    .assert()
    .success();
    let image = image::open(&output).unwrap();
    assert_eq!((image.width(), image.height()), (128, 64));

    lg().args([
        "render",
        doc.to_str().unwrap(),
        "-o",
        jpeg.to_str().unwrap(),
        "--format",
        "jpeg",
        "--sampling",
        "smooth",
    ])
    .assert()
    .success();
    lg().args([
        "render",
        doc.to_str().unwrap(),
        "-o",
        webp.to_str().unwrap(),
        "--format",
        "webp",
    ])
    .assert()
    .success();
    assert_eq!(image::open(&jpeg).unwrap().width(), 64);
    assert_eq!(image::open(&webp).unwrap().width(), 64);

    fs::copy(&output, &linked_copy).unwrap();
    lg().args([
        "asset",
        "add",
        doc.to_str().unwrap(),
        "--id",
        "linked-image",
        output.to_str().unwrap(),
        "--linked",
    ])
    .assert()
    .success();
    lg().args([
        "asset",
        "relink",
        doc.to_str().unwrap(),
        "linked-image",
        linked_copy.to_str().unwrap(),
    ])
    .assert()
    .success();
    lg().args([
        "layer",
        "add",
        doc.to_str().unwrap(),
        "--type",
        "image",
        "--id",
        "linked-layer",
        "--asset-id",
        "linked-image",
    ])
    .assert()
    .success();
    lg().args(["render", doc.to_str().unwrap(), "-o", output.to_str().unwrap()])
        .assert()
        .success();

    lg().args([
        "extension",
        "set",
        doc.to_str().unwrap(),
        "com.example.test",
        r#"{"enabled":true}"#,
    ])
    .assert()
    .success();
    lg().args(["extension", "ls", doc.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("com.example.test"));
    lg().args(["extension", "rm", doc.to_str().unwrap(), "com.example.test"])
        .assert()
        .success();

    fs::copy(&doc, &target).unwrap();
    lg().args(["layer", "update", target.to_str().unwrap(), "bottom", "--set", "x=5"])
        .assert()
        .success();
    fs::copy(&doc, &transformed).unwrap();
    lg().args([
        "diff",
        doc.to_str().unwrap(),
        target.to_str().unwrap(),
        "-o",
        diff.to_str().unwrap(),
    ])
    .assert()
    .success();
    lg().args(["exec", transformed.to_str().unwrap(), diff.to_str().unwrap()])
        .assert()
        .success();
    lg().args([
        "inspect",
        transformed.to_str().unwrap(),
        "--path",
        "layers.0.transform.x",
    ])
    .assert()
    .success()
    .stdout("5.0\n");

    lg().args([
        "watch",
        doc.to_str().unwrap(),
        "--ops",
        ops.to_str().unwrap(),
        "--render",
        watched.to_str().unwrap(),
        "--once",
    ])
    .assert()
    .success();
    assert_eq!(image::open(&watched).unwrap().width(), 64);

    lg().args(["layer", "rm", doc.to_str().unwrap(), "top"])
        .assert()
        .success();
    lg().args(["validate", doc.to_str().unwrap()]).assert().success();
}
