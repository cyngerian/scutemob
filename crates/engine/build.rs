use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let defs_dir = Path::new(&manifest_dir).join("src/cards/defs");

    // Rerun if any file in defs/ changes or is added/removed.
    println!("cargo::rerun-if-changed=src/cards/defs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("card_defs_generated.rs");

    let mut modules: Vec<String> = Vec::new();

    if defs_dir.exists() {
        for entry in fs::read_dir(&defs_dir).expect("failed to read defs/ directory") {
            let entry = entry.expect("failed to read dir entry");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
            if stem == "mod" {
                continue;
            }
            modules.push(stem);
        }
    }

    modules.sort();

    let mut out = fs::File::create(&out_path).expect("failed to create generated file");

    // Module declarations with absolute #[path] so include! resolves correctly
    for m in &modules {
        let abs_path = defs_dir.join(format!("{m}.rs"));
        let abs_str = abs_path.display();
        writeln!(out, "#[path = \"{abs_str}\"]").unwrap();
        writeln!(out, "pub mod {m};").unwrap();
    }

    writeln!(out).unwrap();

    // Collector function
    writeln!(
        out,
        "pub fn all_cards() -> Vec<crate::cards::card_definition::CardDefinition> {{"
    )
    .unwrap();
    writeln!(out, "    vec![").unwrap();
    for m in &modules {
        writeln!(out, "        {m}::card(),").unwrap();
    }
    writeln!(out, "    ]").unwrap();
    writeln!(out, "}}").unwrap();
}
