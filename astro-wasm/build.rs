fn main() {
    // Point cargo to the directory containing libswe.a.
    // LIBSWE_DIR is exported by build.sh before invoking cargo.
    let libswe_dir = std::env::var("LIBSWE_DIR")
        .unwrap_or_else(|_| "../lib".to_string());
    println!("cargo:rustc-link-search=native={libswe_dir}");
    println!("cargo:rustc-link-lib=static=swe");
    println!("cargo:rerun-if-env-changed=LIBSWE_DIR");

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .expect("CARGO_MANIFEST_DIR has no parent");

    // --preload-file must use an absolute host path so emcc can find the
    // ephe/ directory regardless of which directory emcc is invoked from.
    let ephe_host = repo_root.join("ephe");
    let preload_arg = format!("{}@/ephe/", ephe_host.display());
    println!("cargo:rustc-link-arg=--preload-file");
    println!("cargo:rustc-link-arg={preload_arg}");

    // Direct emcc to write JS glue + WASM + data bundle straight into dist/.
    // Without an explicit -o pointing to a .js target, emcc only writes the
    // bare .wasm binary. build.sh ensures dist/ exists before cargo runs.
    let dist_js = repo_root.join("dist").join("astro.js");
    println!("cargo:rustc-link-arg=-o");
    println!("cargo:rustc-link-arg={}", dist_js.display());
}

