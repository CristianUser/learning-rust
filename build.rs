fn main() {
    #[cfg(target_os = "windows")]
    {
        use std::env;
        use std::fs;
        use std::path::PathBuf;

        // Get the output directory from the environment variables
        let out_dir = env::var("OUT_DIR").unwrap();
        let target_dir = PathBuf::from(out_dir).join("../../../");

        // Define the path to the tool
        let tool_path = PathBuf::from("packaging/windows/PDFtoPrinter.exe");

        // Copy the tool to the target directory if it exists
        if tool_path.exists() {
            fs::copy(&tool_path, target_dir.join("PDFtoPrinter.exe"))
                .expect("Failed to copy PDFtoPrinter.exe");
            println!("cargo:rerun-if-changed={}", tool_path.display());
        }
    }
}
