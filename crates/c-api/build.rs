use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_path = PathBuf::from(&crate_dir)
        .join("include")
        .join(format!("{}.h", package_name.replace('-', "_")));

    // Only generate the header if cbindgen is available
    // This allows the crate to build even without cbindgen
    if let Ok(cbindgen) = env::var("CARGO_FEATURE_CBINDGEN") {
        cbindgen::Builder::new()
            .with_crate(crate_dir)
            .with_language(cbindgen::Language::C)
            .generate()
            .expect("Unable to generate bindings")
            .write_to_file(output_path);
    }
}
