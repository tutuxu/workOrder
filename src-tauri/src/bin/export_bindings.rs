fn main() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts");
    let path = path.to_str().expect("invalid bindings path");
    workorder_lib::export_typescript_bindings(path)
        .expect("Failed to export typescript bindings");
    println!("Exported bindings to {path}");
}
