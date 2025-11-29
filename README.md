# Face Crop CLI

![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg?logo=rust)
![Status](https://img.shields.io/badge/status-active-success.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

**Face Crop CLI** is a high-performance command-line utility written in Rust that automatically detects faces in images and generates square crops centered on the subject. 

It is designed for portability and ease of use, embedding the required face detection models directly into the binary so you don't need to manage external dependency files at runtime.

## 🚀 Features

- **Embedded Models**: The SeetaFace detection model is baked into the executable using `include_bytes!`, making the binary completely self-contained.
- **Smart Validation**: 
  - Automatically rejects images with no faces.
  - Rejects images with multiple faces to ensure the correct subject is cropped.
- **Intelligent Geometry**: Calculates a square crop based on the shortest dimension of the image, centering it perfectly on the detected face bounding box.
- **Safety Checks**: Handles boundary conditions to ensure crops never exceed image dimensions (e.g., if a face is too close to an edge).
- **Flexible Output**: Automatically handles output naming (appending `_cropped`) if no specific output path is provided.

## 🛠 Tech Stack

- **Language**: Rust
- **CLI Framework**: [clap](https://crates.io/crates/clap) (Parser derive mode)
- **Computer Vision**: [rustface](https://crates.io/crates/rustface) (SeetaFace implementation)
- **Image Processing**: [image](https://crates.io/crates/image)
- **Error Handling**: [anyhow](https://crates.io/crates/anyhow)

## 📋 Prerequisites

To build this project from source, you need the Rust toolchain installed.

- **Rust**: v1.70.0 or higher
- **Cargo**: Included with Rust

## 📦 Installation

1. **Clone the repository**
   ```bash
   git clone [https://github.com/yourusername/face-crop-cli.git](https://github.com/yourusername/face-crop-cli.git)
   cd face-crop-cli
   ```

2. **Download the Model**
   Ensure the SeetaFace model binary is present in the `models/` directory relative to the project root.
   *Required Path:* `models/seeta_fd_frontal_v1.0.bin`

3. **Build Release Binary**
   ```bash
   cargo build --release
   ```
   The executable will be available at `./target/release/face-crop-cli`.

## 💻 Usage

### Basic Usage
Run the tool by pointing it to an input image. By default, it creates a new file with `_cropped` appended to the filename.

```bash
# Using cargo
cargo run --release -- --input photo.jpg

# Using the binary directly
./target/release/face-crop-cli --input ./assets/profile.jpg
```

**Result:** Generates `./assets/profile_cropped.jpg`

### Custom Output Path
You can specify an exact output location using the `--output` (or `-o`) flag.

```bash
cargo run --release -- -i raw_photo.png -o final_avatar.png
```

## ⚙️ Configuration

No configuration files or environment variables are required. 
* **Model Loading**: The application extracts the embedded model to a temporary file at runtime using `tempfile` to interface with the C++ based logic in `rustface`, and cleans it up automatically upon completion.

## 🤝 Contributing

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.

