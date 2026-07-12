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

    lg().args(["layer", "rm", doc.to_str().unwrap(), "top"])
        .assert()
        .success();
    lg().args(["validate", doc.to_str().unwrap()]).assert().success();
}
