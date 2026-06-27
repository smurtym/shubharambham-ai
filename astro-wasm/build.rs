fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();

    if target.contains("emscripten") {
        // ---------------------------------------------------------------------------
        // Emscripten branch — link against pre-built libswe.a from build.sh Phase 2
        // ---------------------------------------------------------------------------
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

        // Direct emcc to write JS glue + WASM + data bundle to public/.
        // Vite copies public/ contents verbatim into dist/.
        let public_js = repo_root.join("public").join("astro.js");
        println!("cargo:rustc-link-arg=-o");
        println!("cargo:rustc-link-arg={}", public_js.display());
    } else {
        // ---------------------------------------------------------------------------
        // Native branch — compile swisseph C sources directly via the cc crate.
        // This is used by `cargo test` (no Emscripten required).
        // ---------------------------------------------------------------------------
        cc::Build::new()
            .include("../vendor/swisseph")
            .define("NOT_WINDOWS", None)
            .opt_level(2)
            .warnings(false)
            .file("../vendor/swisseph/swedate.c")
            .file("../vendor/swisseph/swehouse.c")
            .file("../vendor/swisseph/swejpl.c")
            .file("../vendor/swisseph/swemmoon.c")
            .file("../vendor/swisseph/swemplan.c")
            .file("../vendor/swisseph/sweph.c")
            .file("../vendor/swisseph/swephlib.c")
            .compile("swe");
        // .compile() automatically emits:
        //   cargo:rustc-link-lib=static=swe
        //   cargo:rustc-link-search=native=<out_dir>
    }
}

