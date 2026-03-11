use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rustc-check-cfg=cfg(has_flowcutter_ffi)");
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let flow_root = manifest_dir.join("../../flow-cutter-pace17");
    let flow_src = flow_root.join("src");
    let bridge = manifest_dir.join("cpp/flowcutter_bridge.cpp");

    println!("cargo:rerun-if-changed={}", bridge.display());

    if !(flow_root.exists() && flow_src.exists()) {
        println!(
            "cargo:warning=FlowCutter sources not found at '{}'; using subprocess fallback.",
            flow_root.display()
        );
        return;
    }

    let mut sources = Vec::new();
    let entries = match fs::read_dir(&flow_src) {
        Ok(v) => v,
        Err(_) => {
            println!(
                "cargo:warning=Could not read '{}'; using subprocess fallback.",
                flow_src.display()
            );
            return;
        }
    };

    for entry in entries.flatten() {
        let p = entry.path();
        if is_cpp(&p) {
            println!("cargo:rerun-if-changed={}", p.display());
            sources.push(p);
        }
    }

    if sources.is_empty() {
        println!(
            "cargo:warning=No FlowCutter C++ sources found in '{}'; using subprocess fallback.",
            flow_src.display()
        );
        return;
    }

    println!("cargo:rustc-cfg=has_flowcutter_ffi");

    let mut build = cc::Build::new();
    build.cpp(true);
    build.warnings(false);
    build.include(&flow_src);
    build.define("NDEBUG", None);
    // pace.cpp defines a CLI main; rename it so we can link as a library.
    build.define("main", Some("flowcutter_pace17_ignored_main"));
    build.flag_if_supported("-std=c++11");
    build.flag_if_supported("-O3");

    for src in sources {
        build.file(src);
    }
    build.file(bridge);
    build.compile("flowcutter_ffi");
}

fn is_cpp(p: &Path) -> bool {
    p.extension()
        .and_then(|x| x.to_str())
        .map(|x| x.eq_ignore_ascii_case("cpp"))
        .unwrap_or(false)
}
