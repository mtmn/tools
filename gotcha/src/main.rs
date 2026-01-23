use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use rusty_tesseract::Image;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <image_path> [output_path]", args[0]);
        eprintln!("Example: {} input.png output.txt", args[0]);
        std::process::exit(1);
    }
    
    let image_path = &args[1];
    let output_path = if args.len() >= 3 {
        args[2].clone()
    } else {
        let input = Path::new(image_path);
        let stem = input.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        format!("{}.txt", stem)
    };
    
    println!("Reading text from: {}", image_path);
    
    let img = match Image::from_path(image_path) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Error loading image: {}", e);
            std::process::exit(1);
        }
    };
    
    match rusty_tesseract::image_to_string(&img, &Default::default()) {
        Ok(text) => {
            println!("Text extracted successfully!");
            println!("Writing to: {}", output_path);
            
            match write_to_file(&output_path, &text) {
                Ok(_) => {
                    println!("✓ Text saved to {}", output_path);
                }
                Err(e) => {
                    eprintln!("Error writing to file: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error performing OCR: {}", e);
            std::process::exit(1);
        }
    }
}

fn write_to_file(path: &str, content: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
