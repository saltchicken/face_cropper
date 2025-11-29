use anyhow::{Context, Result, bail};
use clap::Parser;
use image::GenericImageView;
use rustface::{FaceInfo, ImageData};
use std::io::Write; // Needed to write the model to temp
use std::path::PathBuf;

// 1. Embed the model bytes into the binary at compile time.
// Note: ".." looks in the parent of src/, which is the project root.
const MODEL_BYTES: &[u8] = include_bytes!("../models/seeta_fd_frontal_v1.0.bin");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input image path
    #[arg(short, long)]
    input: PathBuf,


    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 2. Write the embedded model to a temporary file
    // We use a NamedTempFile so we can get a file path to pass to rustface
    let mut model_temp_file = tempfile::Builder::new()
        .suffix(".bin")
        .tempfile()
        .context("Failed to create temp file for model")?;

    model_temp_file
        .write_all(MODEL_BYTES)
        .context("Failed to write model bytes")?;

    // 3. Get the path of the temp file
    let model_path = model_temp_file.path();

    let mut img = image::open(&args.input).context("Failed to open image")?;
    let (width, height) = img.dimensions();

    // 4. Initialize Detector using the temp file path
    let mut detector = rustface::create_detector(model_path.to_str().unwrap())
        .context("Failed to create face detector")?;

    detector.set_min_face_size(20);
    detector.set_score_thresh(2.0);
    detector.set_pyramid_scale_factor(0.8);
    detector.set_slide_window_step(4, 4);

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


    let output_path = match args.output {
        Some(path) => path,
        None => {
            let stem = args
                .input
                .file_stem()
                .context("Input file has no file name")?;
            let mut new_filename = stem.to_os_string();
            new_filename.push("_cropped");

            if let Some(ext) = args.input.extension() {
                new_filename.push(".");
                new_filename.push(ext);
            }

            args.input.with_file_name(new_filename)
        }
    };

    // Crop and Save
    println!("Processing {:?} -> Face at {:?}", args.input, bbox);
    let cropped_img = img.crop(origin_x, origin_y, crop_size, crop_size);
    cropped_img
        .save(&output_path)
        .context("Failed to save output")?;

    println!("Saved to {:?}", output_path);

    // The temp file is automatically deleted when 'model_temp_file' goes out of scope here.
    Ok(())
}