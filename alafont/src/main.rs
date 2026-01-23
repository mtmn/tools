use std::fs;
use std::path::PathBuf;
use clap::Parser;
use toml_edit::{DocumentMut, value};

#[derive(Parser, Debug)]
#[command(name = "alacritty_font")]
struct Args {
    #[arg(short, long)]
    font: String,

    #[arg(short, long)]
    size: f64,

    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config_path = if let Some(path) = args.config {
        path
    } else {
        let home = std::env::var("HOME")?;
        PathBuf::from(home).join(".config/alacritty/alacritty.toml")
    };

    let content = fs::read_to_string(&config_path)?;
    let mut doc = content.parse::<DocumentMut>()?;

    doc["font"]["size"] = value(args.size);

    doc["font"]["normal"]["family"] = value(&args.font);
    doc["font"]["bold"]["family"] = value(&args.font);
    doc["font"]["italic"]["family"] = value(&args.font);

    fs::write(&config_path, doc.to_string())?;

    println!("Updated font to '{}' with size {}", args.font, args.size);

    Ok(())
}
