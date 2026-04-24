fn main() {
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

    println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
    println!("cargo:rustc-link-lib=SoapySDR");
}
