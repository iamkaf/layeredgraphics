use anyhow::{Context, Result, anyhow, bail};
use base64::Engine;
use clap::{Args, Parser, Subcommand, ValueEnum};
use lg_core::{
    BlendMode, Command, Document, Layer, LayerCommon, LayerKind, LayerPatch, RenderOptions, Rgba, Transform,
    execute_commands, load_kgfx, render_document_png, save_kgfx,
};
use serde_json::{Value, json};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "lg", version, about = "Headless layered graphics authoring")]
struct Cli {
    #[command(subcommand)]
    command: TopCommand,
}

#[derive(Subcommand)]
enum TopCommand {
    New(NewArgs),
    Exec(ExecArgs),
    Layer {
        #[command(subcommand)]
        command: LayerCommand,
    },
    Asset {
        #[command(subcommand)]
        command: AssetCommand,
    },
    Render(RenderArgs),
    Inspect(InspectArgs),
    Validate(ValidateArgs),
}

#[derive(Args)]
struct NewArgs {
    file: PathBuf,
    #[arg(long)]
    id: Option<String>,
    #[arg(long, default_value_t = 1200)]
    width: u32,
    #[arg(long, default_value_t = 630)]
    height: u32,
    #[arg(long, default_value_t = 72.0)]
    dpi: f32,
}

#[derive(Args)]
struct ExecArgs {
    file: PathBuf,
    #[arg(default_value = "-")]
    ops: String,
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand)]
enum LayerCommand {
    Add(LayerAddArgs),
    Update(LayerUpdateArgs),
    Rm(LayerRemoveArgs),
    Ls(LayerListArgs),
    Move(LayerMoveArgs),
}

#[derive(Clone, Copy, ValueEnum)]
enum LayerType {
    Image,
    Fill,
    Text,
    Group,
}

#[derive(Clone, Copy, ValueEnum)]
enum BlendArg {
    Normal,
    Multiply,
}

impl From<BlendArg> for BlendMode {
    fn from(value: BlendArg) -> Self {
        match value {
            BlendArg::Normal => BlendMode::Normal,
            BlendArg::Multiply => BlendMode::Multiply,
        }
    }
}

#[derive(Args)]
struct LayerAddArgs {
    file: PathBuf,
    #[arg(long = "type")]
    kind: LayerType,
    #[arg(long)]
    id: Option<String>,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    parent: Option<String>,
    #[arg(long)]
    index: Option<usize>,
    #[arg(long)]
    asset_id: Option<String>,
    #[arg(long)]
    font_asset_id: Option<String>,
    #[arg(long)]
    text: Option<String>,
    #[arg(long, default_value_t = 48.0)]
    font_size: f32,
    #[arg(long, default_value = "#ffffffff")]
    color: String,
    #[arg(long)]
    width: Option<u32>,
    #[arg(long)]
    height: Option<u32>,
    #[arg(long, default_value_t = 0.0)]
    x: f32,
    #[arg(long, default_value_t = 0.0)]
    y: f32,
    #[arg(long, default_value_t = 1.0)]
    scale_x: f32,
    #[arg(long, default_value_t = 1.0)]
    scale_y: f32,
    #[arg(long, default_value_t = 1.0)]
    opacity: f32,
    #[arg(long, value_enum, default_value_t = BlendArg::Normal)]
    blend: BlendArg,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct LayerUpdateArgs {
    file: PathBuf,
    id: String,
    #[arg(long = "set", required = true)]
    values: Vec<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct LayerRemoveArgs {
    file: PathBuf,
    id: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct LayerListArgs {
    file: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct LayerMoveArgs {
    file: PathBuf,
    id: String,
    #[arg(long)]
    parent: Option<String>,
    #[arg(long, conflicts_with_all = ["above", "below"])]
    to: Option<usize>,
    #[arg(long, conflicts_with = "below")]
    above: Option<String>,
    #[arg(long)]
    below: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand)]
enum AssetCommand {
    Add(AssetAddArgs),
    Ls(AssetListArgs),
    Rm(AssetRemoveArgs),
}

#[derive(Args)]
struct AssetAddArgs {
    file: PathBuf,
    source: PathBuf,
    #[arg(long)]
    id: String,
    #[arg(long)]
    media_type: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct AssetListArgs {
    file: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct AssetRemoveArgs {
    file: PathBuf,
    id: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct RenderArgs {
    file: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
    #[arg(long, default_value = "png")]
    format: String,
    #[arg(long)]
    layer: Option<String>,
    #[arg(long, default_value_t = 1.0)]
    scale: f32,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct InspectArgs {
    file: PathBuf,
    #[arg(long)]
    path: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ValidateArgs {
    file: String,
    #[arg(long)]
    json: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    match Cli::parse().command {
        TopCommand::New(args) => create_document(args),
        TopCommand::Exec(args) => exec(args),
        TopCommand::Layer { command } => match command {
            LayerCommand::Add(args) => layer_add(args),
            LayerCommand::Update(args) => mutate(
                args.file,
                vec![Command::LayerUpdate {
                    id: args.id,
                    set: parse_patch(&args.values)?,
                }],
                args.json,
            ),
            LayerCommand::Rm(args) => mutate(args.file, vec![Command::LayerRemove { id: args.id }], args.json),
            LayerCommand::Ls(args) => layer_list(args),
            LayerCommand::Move(args) => mutate(
                args.file,
                vec![Command::LayerMove {
                    id: args.id,
                    parent_id: args.parent,
                    to: args.to,
                    above: args.above,
                    below: args.below,
                }],
                args.json,
            ),
        },
        TopCommand::Asset { command } => match command {
            AssetCommand::Add(args) => asset_add(args),
            AssetCommand::Ls(args) => asset_list(args),
            AssetCommand::Rm(args) => mutate(args.file, vec![Command::AssetRemove { id: args.id }], args.json),
        },
        TopCommand::Render(args) => render(args),
        TopCommand::Inspect(args) => inspect(args),
        TopCommand::Validate(args) => validate(args),
    }
}

fn create_document(args: NewArgs) -> Result<()> {
    if args.file.exists() {
        bail!("{} already exists", args.file.display());
    }
    let mut doc = Document::new(args.width, args.height, args.dpi);
    if let Some(id) = args.id {
        doc.manifest.id = id;
    }
    save_kgfx(&args.file, &doc)?;
    println!(
        "created {} ({}x{} at {} dpi)",
        args.file.display(),
        args.width,
        args.height,
        args.dpi
    );
    Ok(())
}

fn exec(args: ExecArgs) -> Result<()> {
    let text = read_text_input(&args.ops)?;
    let value: Value = serde_json::from_str(&text).context("operations are not valid JSON")?;
    let commands: Vec<Command> = if value.is_array() {
        serde_json::from_value(value).context("invalid operation array")?
    } else {
        vec![serde_json::from_value(value).context("invalid operation")?]
    };
    mutate(args.file, commands, args.json)
}

fn layer_add(args: LayerAddArgs) -> Result<()> {
    let id = args.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let name = args.name.unwrap_or_else(|| {
        match args.kind {
            LayerType::Image => "Image",
            LayerType::Fill => "Fill",
            LayerType::Text => "Text",
            LayerType::Group => "Group",
        }
        .to_owned()
    });
    let kind = match args.kind {
        LayerType::Image => LayerKind::Image {
            asset_id: args
                .asset_id
                .ok_or_else(|| anyhow!("--asset-id is required for image layers"))?,
        },
        LayerType::Fill => LayerKind::Fill {
            width: args
                .width
                .ok_or_else(|| anyhow!("--width is required for fill layers"))?,
            height: args
                .height
                .ok_or_else(|| anyhow!("--height is required for fill layers"))?,
            color: parse_color(&args.color)?,
        },
        LayerType::Text => LayerKind::Text {
            text: args.text.ok_or_else(|| anyhow!("--text is required for text layers"))?,
            font_asset_id: args
                .font_asset_id
                .ok_or_else(|| anyhow!("--font-asset-id is required for text layers"))?,
            font_size: args.font_size,
            color: parse_color(&args.color)?,
        },
        LayerType::Group => LayerKind::Group { children: Vec::new() },
    };
    let layer = Layer {
        common: LayerCommon {
            id,
            name,
            visible: true,
            opacity: args.opacity,
            blend_mode: args.blend.into(),
            transform: Transform {
                x: args.x,
                y: args.y,
                scale_x: args.scale_x,
                scale_y: args.scale_y,
            },
        },
        kind,
    };
    mutate(
        args.file,
        vec![Command::LayerAdd {
            layer,
            parent_id: args.parent,
            index: args.index,
        }],
        args.json,
    )
}

fn asset_add(args: AssetAddArgs) -> Result<()> {
    let bytes = fs::read(&args.source).with_context(|| format!("cannot read {}", args.source.display()))?;
    let media_type = args
        .media_type
        .unwrap_or_else(|| media_type_for(&args.source).to_owned());
    let command = Command::AssetAdd {
        id: args.id,
        media_type,
        bytes_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
        original_name: args
            .source
            .file_name()
            .map(|value| value.to_string_lossy().into_owned()),
    };
    mutate(args.file, vec![command], args.json)
}

fn mutate(file: PathBuf, commands: Vec<Command>, json_output: bool) -> Result<()> {
    let mut doc = load_kgfx(&file)?;
    let result = execute_commands(&mut doc, &commands)?;
    save_kgfx(&file, &doc)?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("applied {} operation(s); revision {}", result.applied, result.revision);
    }
    Ok(())
}

fn layer_list(args: LayerListArgs) -> Result<()> {
    let doc = load_kgfx(&args.file)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&doc.manifest.layers)?);
    } else {
        print_layers(&doc.manifest.layers, 0);
    }
    Ok(())
}

fn print_layers(layers: &[Layer], depth: usize) {
    for (index, layer) in layers.iter().enumerate().rev() {
        let kind = match &layer.kind {
            LayerKind::Image { .. } => "image",
            LayerKind::Fill { .. } => "fill",
            LayerKind::Text { .. } => "text",
            LayerKind::Group { .. } => "group",
        };
        println!(
            "{}{}  {}  {}  [{}]",
            "  ".repeat(depth),
            index,
            layer.common.id,
            layer.common.name,
            kind
        );
        if let LayerKind::Group { children } = &layer.kind {
            print_layers(children, depth + 1);
        }
    }
}

fn asset_list(args: AssetListArgs) -> Result<()> {
    let doc = load_kgfx(&args.file)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&doc.manifest.assets)?);
    } else {
        for asset in doc.manifest.assets.values() {
            println!(
                "{}  {}  {} bytes  {}",
                asset.id,
                asset.media_type,
                asset.byte_length,
                asset.original_name.as_deref().unwrap_or("-")
            );
        }
    }
    Ok(())
}

fn render(args: RenderArgs) -> Result<()> {
    if !args.format.eq_ignore_ascii_case("png") {
        bail!("milestone 1 supports only --format png");
    }
    let doc = load_kgfx(&args.file)?;
    let png = render_document_png(
        &doc,
        &RenderOptions {
            layer_id: args.layer,
            scale: args.scale,
        },
    )?;
    safe_write(&args.output, &png)?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &json!({ "output": args.output, "bytes": png.len(), "revision": doc.manifest.revision })
            )?
        );
    } else {
        println!("rendered {} ({} bytes)", args.output.display(), png.len());
    }
    Ok(())
}

fn inspect(args: InspectArgs) -> Result<()> {
    let doc = load_kgfx(&args.file)?;
    let diagnostics = doc.validate();
    let mut value = serde_json::to_value(&doc.manifest)?;
    if let Value::Object(object) = &mut value {
        object.insert(
            "summary".to_owned(),
            json!({
                "layerCount": count_layers(&doc.manifest.layers),
                "assetCount": doc.manifest.assets.len(),
                "valid": diagnostics.is_empty()
            }),
        );
        object.insert("diagnostics".to_owned(), serde_json::to_value(diagnostics)?);
    }
    if let Some(path) = args.path {
        value = select_path(&value, &path)
            .ok_or_else(|| anyhow!("inspection path '{path}' does not exist"))?
            .clone();
    }
    if args.json || !value.is_string() {
        println!("{}", serde_json::to_string_pretty(&value)?);
    } else if let Some(value) = value.as_str() {
        println!("{value}");
    }
    Ok(())
}

fn validate(args: ValidateArgs) -> Result<()> {
    let is_kgfx =
        args.file != "-" && Path::new(&args.file).extension().and_then(|value| value.to_str()) == Some("kgfx");
    let diagnostics = if is_kgfx {
        let doc = load_kgfx(&args.file)?;
        doc.validate()
    } else {
        let text = read_text_input(&args.file)?;
        let value: Value = serde_json::from_str(&text).context("input is not valid JSON")?;
        if value.is_array() {
            let _: Vec<Command> = serde_json::from_value(value).context("invalid operation array")?;
        } else {
            let _: Command = serde_json::from_value(value).context("invalid operation")?;
        }
        Vec::new()
    };
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({ "valid": diagnostics.is_empty(), "diagnostics": diagnostics }))?
        );
    } else if diagnostics.is_empty() {
        println!("valid");
    } else {
        for diagnostic in &diagnostics {
            eprintln!(
                "{:?} {} at {}: {}",
                diagnostic.severity, diagnostic.code, diagnostic.path, diagnostic.message
            );
        }
        bail!("validation failed with {} diagnostic(s)", diagnostics.len());
    }
    Ok(())
}

fn parse_patch(values: &[String]) -> Result<LayerPatch> {
    let mut patch = LayerPatch::default();
    for value in values {
        let (key, value) = value
            .split_once('=')
            .ok_or_else(|| anyhow!("--set values must use key=value"))?;
        match key {
            "name" => patch.name = Some(value.to_owned()),
            "visible" => patch.visible = Some(value.parse().context("visible must be true or false")?),
            "opacity" => patch.opacity = Some(value.parse().context("opacity must be a number")?),
            "blend" | "blendMode" => {
                patch.blend_mode = Some(match value {
                    "normal" => BlendMode::Normal,
                    "multiply" => BlendMode::Multiply,
                    _ => bail!("blend must be normal or multiply"),
                })
            }
            "x" => patch.x = Some(value.parse().context("x must be a number")?),
            "y" => patch.y = Some(value.parse().context("y must be a number")?),
            "scaleX" | "scale_x" => patch.scale_x = Some(value.parse().context("scaleX must be a number")?),
            "scaleY" | "scale_y" => patch.scale_y = Some(value.parse().context("scaleY must be a number")?),
            "assetId" | "asset_id" => patch.asset_id = Some(value.to_owned()),
            "text" => patch.text = Some(value.to_owned()),
            "fontAssetId" | "font_asset_id" => patch.font_asset_id = Some(value.to_owned()),
            "fontSize" | "font_size" => patch.font_size = Some(value.parse().context("fontSize must be a number")?),
            "color" => patch.color = Some(parse_color(value)?),
            "width" => patch.width = Some(value.parse().context("width must be an integer")?),
            "height" => patch.height = Some(value.parse().context("height must be an integer")?),
            _ => bail!("unknown layer property '{key}'"),
        }
    }
    Ok(patch)
}

fn parse_color(value: &str) -> Result<Rgba> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    let expanded = match hex.len() {
        6 => format!("{hex}ff"),
        8 => hex.to_owned(),
        _ => bail!("color must be #RRGGBB or #RRGGBBAA"),
    };
    let number = u32::from_str_radix(&expanded, 16).context("color contains invalid hexadecimal digits")?;
    Ok(Rgba(
        (number >> 24) as u8,
        (number >> 16) as u8,
        (number >> 8) as u8,
        number as u8,
    ))
}

fn media_type_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        _ => "application/octet-stream",
    }
}

fn read_text_input(value: &str) -> Result<String> {
    if value == "-" {
        let mut text = String::new();
        io::stdin().read_to_string(&mut text)?;
        Ok(text)
    } else {
        fs::read_to_string(value).with_context(|| format!("cannot read {value}"))
    }
}

fn count_layers(layers: &[Layer]) -> usize {
    layers
        .iter()
        .map(|layer| {
            1 + match &layer.kind {
                LayerKind::Group { children } => count_layers(children),
                _ => 0,
            }
        })
        .sum()
}

fn select_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for segment in path.split('.') {
        current = match current {
            Value::Object(map) => map.get(segment)?,
            Value::Array(values) => values.get(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(current)
}

fn safe_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let temp = path.with_extension(format!(
        "{}.tmp",
        path.extension().and_then(|value| value.to_str()).unwrap_or("output")
    ));
    fs::write(&temp, bytes)?;
    fs::rename(&temp, path).inspect_err(|_| {
        let _ = fs::remove_file(&temp);
    })?;
    Ok(())
}
