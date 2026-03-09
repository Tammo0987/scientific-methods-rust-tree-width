#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;

use rustc_driver::{Callbacks, Compilation};
use rustc_hir::def::DefKind;
use rustc_hir::def_id::LOCAL_CRATE;
use rustc_interface::interface::Compiler;
use rustc_middle::mir::Body;
use rustc_middle::ty::TyCtxt;

use serde::Serialize;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::Mutex;

#[derive(Serialize)]
struct FunctionRecord {
    name: String,
    #[serde(rename = "crate")]
    krate: String,
    blocks: usize,
    edges: Vec<(usize, usize)>,
    stmt_count: usize,
    is_unsafe: bool,
}

struct Cfg {
    blocks: usize,
    edges: Vec<(usize, usize)>,
    stmt_count: usize,
}

struct MirCallbacks {
    writer: Mutex<Box<dyn Write + Send>>,
    crates: Vec<String>,
}

impl MirCallbacks {
    fn new() -> Self {
        let writer: Box<dyn Write + Send> = match std::env::var("MIR_OUTPUT") {
            Ok(path) => {
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .unwrap_or_else(|e| panic!("cannot open MIR_OUTPUT '{path}': {e}"));
                Box::new(BufWriter::new(file))
            }
            Err(_) => Box::new(BufWriter::new(std::io::stdout())),
        };
        let crates = match std::env::var("MIR_CRATES") {
            Ok(s) => s.split(',').map(str::trim).map(str::to_owned).collect(),
            Err(_) => Vec::new(),
        };
        Self { writer: Mutex::new(writer), crates }
    }
}

impl Callbacks for MirCallbacks {
    fn after_analysis<'tcx>(&mut self, _compiler: &Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        extract_all(tcx, &self.writer, &self.crates);
        Compilation::Continue
    }
}

fn extract_cfg(body: &Body<'_>) -> Cfg {
    let n_raw = body.basic_blocks.len();
    let is_cleanup: Vec<bool> = body.basic_blocks.iter().map(|bb| bb.is_cleanup).collect();

    let mut remap = vec![None::<usize>; n_raw];
    let mut next = 0usize;
    for (i, &cleanup) in is_cleanup.iter().enumerate() {
        if !cleanup {
            remap[i] = Some(next);
            next += 1;
        }
    }
    let blocks = next;

    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut stmt_count = 0usize;

    for (bb_idx, bb_data) in body.basic_blocks.iter_enumerated() {
        let src_raw = bb_idx.index();
        if is_cleanup[src_raw] {
            continue;
        }
        let src = remap[src_raw].unwrap();
        stmt_count += bb_data.statements.len();

        for succ in bb_data.terminator().successors() {
            let tgt_raw = succ.index();
            if is_cleanup[tgt_raw] {
                continue;
            }
            if let Some(tgt) = remap[tgt_raw] {
                let edge = (src, tgt);
                if !edges.contains(&edge) {
                    edges.push(edge);
                }
            }
        }
    }

    Cfg { blocks, edges, stmt_count }
}

fn extract_all(tcx: TyCtxt<'_>, writer: &Mutex<Box<dyn Write + Send>>, crates: &[String]) {
    let krate_name = tcx.crate_name(LOCAL_CRATE).to_string();
    if !crates.is_empty() && !crates.contains(&krate_name) {
        return;
    }

    for &local_def_id in tcx.mir_keys(()).iter() {
        let def_id = local_def_id.to_def_id();

        match tcx.def_kind(local_def_id) {
            DefKind::Fn | DefKind::AssocFn => {}
            _ => continue,
        }

        let body = tcx.optimized_mir(def_id);
        let cfg = extract_cfg(body);
        let is_unsafe = tcx.fn_sig(def_id).skip_binder().safety().is_unsafe();

        let record = FunctionRecord {
            name: tcx.def_path_str(def_id),
            krate: krate_name.clone(),
            blocks: cfg.blocks,
            edges: cfg.edges,
            stmt_count: cfg.stmt_count,
            is_unsafe,
        };

        let line = serde_json::to_string(&record).expect("serialisation failed");
        let mut w = writer.lock().unwrap();
        writeln!(w, "{line}").expect("write failed");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut callbacks = MirCallbacks::new();
    rustc_driver::run_compiler(&args, &mut callbacks);
}
