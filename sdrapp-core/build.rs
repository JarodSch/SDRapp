fn main() {
    // Nur neu bauen wenn sich relevante Quellen geändert haben
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_dir = format!("{}/../SDRapp/Application", crate_dir);
    std::fs::create_dir_all(&output_dir).ok();

    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_language(cbindgen::Language::C)
        .with_include_guard("SDRAPP_CORE_H")
        .generate()
        .expect("cbindgen fehlgeschlagen")
        .write_to_file(format!("{}/sdrapp_core.h", output_dir));

    // SoapySDR verlinken — Homebrew-Prefix aus Umgebung lesen (portabel)
    let homebrew_prefix = std::env::var("HOMEBREW_PREFIX")
        .unwrap_or_else(|_| "/opt/homebrew".to_string());
    println!("cargo:rustc-link-search=native={}/lib", homebrew_prefix);
    println!("cargo:rustc-link-lib=SoapySDR");
}
