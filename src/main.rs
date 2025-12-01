use anyhow::{Context, Result, bail};
use clap::Parser;
use image::GenericImageView;
use rustface::{FaceInfo, ImageData};
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

// 1. Embed the model bytes into the binary at compile time.
const MODEL_BYTES: &[u8] = include_bytes!("../models/seeta_fd_frontal_v1.0.bin");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input path (can be a single image file or a directory)
    #[arg(short, long)]
    input: PathBuf,

    /// Output path (optional).
    /// If input is a file: this is the destination file path.
    /// If input is a directory: this is the destination directory.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 2. Write the embedded model to a temporary file
    let mut model_temp_file = tempfile::Builder::new()
        .suffix(".bin")
        .tempfile()
        .context("Failed to create temp file for model")?;

    model_temp_file
        .write_all(MODEL_BYTES)
        .context("Failed to write model bytes")?;

    // 3. Get the path of the temp file
    let model_path = model_temp_file.path();

    // 4. Initialize Detector ONCE

    let mut detector = rustface::create_detector(model_path.to_str().unwrap())
        .context("Failed to create face detector")?;

    detector.set_min_face_size(20);
    detector.set_score_thresh(2.0);
    detector.set_pyramid_scale_factor(0.8);
    detector.set_slide_window_step(4, 4);


    if args.input.is_dir() {
        process_directory(&args, &mut *detector)?;
    } else {
        // Process single file

        let output_path = match &args.output {
            Some(p) => p.clone(),
            None => generate_default_output_path(&args.input)?,
        };

        match process_image(&args.input, output_path, &mut *detector) {
            Ok(_) => println!("Successfully processed: {:?}", args.input),
            Err(e) => eprintln!("Error processing {:?}: {}", args.input, e),
        }
    }

    // The temp file is automatically deleted when 'model_temp_file' goes out of scope here.
    Ok(())
}


fn process_directory(args: &Args, detector: &mut dyn rustface::Detector) -> Result<()> {
    let entries = fs::read_dir(&args.input).context("Failed to read input directory")?;

    // If output dir is specified, create it if it doesn't exist
    if let Some(out_dir) = &args.output {
        fs::create_dir_all(out_dir).context("Failed to create output directory")?;
    }

    for entry in entries {
        let entry = entry?;
        let path = entry.path();


        if path.is_file() && is_image_extension(&path) {
            // Calculate output path
            let output_path = if let Some(out_dir) = &args.output {
                // If output dir specified: out_dir / filename_cropped.ext
                let file_name = generate_cropped_filename(&path)?;
                out_dir.join(file_name)
            } else {
                // If no output dir: input_dir / filename_cropped.ext
                generate_default_output_path(&path)?
            };


            match process_image(&path, output_path, detector) {
                Ok(_) => println!("Processed: {:?}", path.file_name().unwrap()),
                Err(e) => eprintln!("Skipping {:?}: {}", path.file_name().unwrap(), e),
            }
        }
    }
    Ok(())
}


fn is_image_extension(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|ext| {
            let e = ext.to_lowercase();
            matches!(
                e.as_str(),
                "jpg" | "jpeg" | "png" | "bmp" | "tif" | "tiff" | "webp"
            )
        })
        .unwrap_or(false)
}


fn generate_default_output_path(input_path: &Path) -> Result<PathBuf> {
    let stem = input_path
        .file_stem()
        .context("Input file has no file name")?;

    let mut new_filename = stem.to_os_string();
    new_filename.push("_cropped");

    if let Some(ext) = input_path.extension() {
        new_filename.push(".");
        new_filename.push(ext);
    }

    Ok(input_path.with_file_name(new_filename))
}


fn generate_cropped_filename(input_path: &Path) -> Result<PathBuf> {
    let stem = input_path
        .file_stem()
        .context("Input file has no file name")?;

    let mut new_filename = stem.to_os_string();
    new_filename.push("_cropped");

    if let Some(ext) = input_path.extension() {
        new_filename.push(".");
        new_filename.push(ext);
    }

    Ok(PathBuf::from(new_filename))
}


fn process_image(
    input_path: &Path,
    output_path: PathBuf,
    detector: &mut dyn rustface::Detector,
) -> Result<()> {
    let mut img = image::open(input_path).context("Failed to open image")?;
    let (width, height) = img.dimensions();

    let gray = img.to_luma8();
    let image_data = ImageData::new(&gray, width, height);

    let faces: Vec<FaceInfo> = detector.detect(&image_data);

    // Validation
    if faces.is_empty() {
        bail!("Validation Failed: No faces detected.");
    } else if faces.len() > 1 {
        bail!(
            "Validation Failed: Multiple faces detected (Found {}).",
            faces.len()
        );
    }

    let face = &faces[0];
    let bbox = face.bbox();

    // Calculate Geometry
    let crop_size = width.min(height);

    let face_center_x = bbox.x() as u32 + (bbox.width() as u32 / 2);
    let face_center_y = bbox.y() as u32 + (bbox.height() as u32 / 2);

    let mut origin_x = face_center_x.saturating_sub(crop_size / 2);
    let mut origin_y = face_center_y.saturating_sub(crop_size / 2);

    if origin_x + crop_size > width {
        origin_x = width - crop_size;
    }
    if origin_y + crop_size > height {
        origin_y = height - crop_size;
    }

    // Crop and Save
    let cropped_img = img.crop(origin_x, origin_y, crop_size, crop_size);
    cropped_img
        .save(&output_path)
        .context("Failed to save output")?;

    Ok(())
}
