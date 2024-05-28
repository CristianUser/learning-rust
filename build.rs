use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Get the output directory from the environment variables
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = PathBuf::from(out_dir).join("../../../");

    // Define the path to the tool
    let tool_path = PathBuf::from("tools/PDFtoPrinter.exe");

    // Copy the tool to the target directory
    fs::copy(&tool_path, target_dir.join("PDFtoPrinter.exe"))
        .expect("Failed to copy PDFtoPrinter.exe");

    // Print instructions for Cargo to rerun the build script if the tool changes
    println!("cargo:rerun-if-changed={}", tool_path.display());
}
