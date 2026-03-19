fn main() {
    generate_cities();

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

// ---------------------------------------------------------------------------
// CSV → Rust code-gen: reads data/cities.csv, writes OUT_DIR/cities_generated.rs
// Uses only std — no new build-dependencies.
// ---------------------------------------------------------------------------

fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn generate_cities() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let csv_path = manifest_dir
        .parent()
        .expect("CARGO_MANIFEST_DIR has no parent")
        .join("data")
        .join("cities.csv");

    println!("cargo:rerun-if-changed={}", csv_path.display());

    let content = std::fs::read_to_string(&csv_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", csv_path.display()));

    struct Row {
        city_id: u32,
        canonical: String,
        timezone: String,
        lang: String,
        city_name: String,
        region1: String,
        region2: String,
        region1_order: u16,
        region2_order: u16,
    }

    let mut rows: Vec<Row> = Vec::new();
    for (lineno, raw) in content.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("city_id") {
            continue;
        }
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() != 9 {
            panic!(
                "cities.csv line {}: expected 9 columns, got {}\n  > {line}",
                lineno + 1,
                cols.len()
            );
        }
        let parse_u16 = |s: &str, field: &str| -> u16 {
            s.trim().parse::<u16>().unwrap_or_else(|_| {
                panic!(
                    "cities.csv line {}: '{}' is not a valid u16 for field '{field}'",
                    lineno + 1,
                    s.trim()
                )
            })
        };
        let parse_u32 = |s: &str, field: &str| -> u32 {
            s.trim().parse::<u32>().unwrap_or_else(|_| {
                panic!(
                    "cities.csv line {}: '{}' is not a valid u32 for field '{field}'",
                    lineno + 1,
                    s.trim()
                )
            })
        };
        rows.push(Row {
            city_id:       parse_u32(cols[0], "city_id"),
            canonical:     cols[1].trim().to_string(),
            timezone:      cols[2].trim().to_string(),
            lang:          cols[3].trim().to_string(),
            city_name:     cols[4].trim().to_string(),
            region1:       cols[5].trim().to_string(),
            region2:       cols[6].trim().to_string(),
            region1_order: parse_u16(cols[7], "region1_order"),
            region2_order: parse_u16(cols[8], "region2_order"),
        });
    }

    // Group translations by city_id, preserving first-occurrence insertion order.
    let mut city_order: Vec<u32> = Vec::new();
    let mut city_map: std::collections::HashMap<u32, (String, String, Vec<Row>)> =
        std::collections::HashMap::new();
    for row in rows {
        let id = row.city_id;
        if !city_map.contains_key(&id) {
            city_order.push(id);
            city_map.insert(id, (row.canonical.clone(), row.timezone.clone(), Vec::new()));
        }
        city_map.get_mut(&id).unwrap().2.push(row);
    }

    // Emit Rust source.
    let mut src = String::from("pub static CITIES: &[CityRecord] = &[\n");
    for id in &city_order {
        let (canonical, timezone, translations) = city_map.get(id).unwrap();
        src.push_str(&format!(
            "    CityRecord {{\n        city_id: {id}u32,\n        canonical_name: \"{}\",\n        timezone: \"{}\",\n        translations: &[\n",
            escape_str(canonical),
            escape_str(timezone)
        ));
        for tr in translations {
            src.push_str(&format!(
                "            (\"{}\", TranslationEntry {{ city_name: \"{}\", region1: \"{}\", region2: \"{}\", region1_order: {}u16, region2_order: {}u16 }}),\n",
                escape_str(&tr.lang),
                escape_str(&tr.city_name),
                escape_str(&tr.region1),
                escape_str(&tr.region2),
                tr.region1_order,
                tr.region2_order
            ));
        }
        src.push_str("        ],\n    },\n");
    }
    src.push_str("];\n");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = std::path::Path::new(&out_dir).join("cities_generated.rs");
    std::fs::write(&dest, &src)
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", dest.display()));
}

