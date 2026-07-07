fn main() {
    workorder_lib::export_typescript_bindings("../src/bindings.ts")
        .expect("Failed to export typescript bindings");
    println!("Exported bindings to ../src/bindings.ts");
}
