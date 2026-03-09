mod graph;
mod stats;
mod treewidth;

use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use clap::Parser;
use graph::Graph;
use rayon::prelude::*;
use serde::Deserialize;
use stats::Accumulator;
use treewidth::Solver;

#[derive(Parser)]
#[command(name = "analyzer", about = "Compute treewidth of MIR CFGs")]
struct Cli {
    /// JSONL file produced by mir-extractor.
    input: PathBuf,

    /// Directory for output files (created if absent).
    #[arg(short, long, default_value = ".")]
    outdir: PathBuf,

    /// Only process the first N functions (0 = all).
    #[arg(long, default_value_t = 0)]
    limit: usize,

    /// Print progress every N functions (0 = silent).
    #[arg(long, default_value_t = 500)]
    progress: usize,

    /// Treewidth solver to use.
    #[arg(long, default_value = "auto", value_enum)]
    solver: SolverArg,
}

#[derive(clap::ValueEnum, Clone)]
enum SolverArg {
    /// Native Rust for small graphs (≤500 nodes), FlowCutter subprocess for large.
    Auto,
    /// Pure Rust min-degree + min-fill elimination. Fast, no subprocess.
    Native,
    /// FlowCutter subprocess. Supports arbitrarily large graphs.
    FlowCutter,
}

#[derive(Deserialize)]
struct FunctionRecord {
    name: String,
    #[serde(rename = "crate")]
    krate: String,
    blocks: usize,
    edges: Vec<(usize, usize)>,
    stmt_count: usize,
    is_unsafe: bool,
}

struct Row {
    name: String,
    krate: String,
    blocks: usize,
    edges: usize,
    stmt_count: usize,
    treewidth: Option<u32>,
    is_unsafe: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    fs::create_dir_all(&cli.outdir)?;

    let records = read_records(&cli.input, cli.limit)?;
    let total = records.len();
    let solver = match cli.solver {
        SolverArg::Auto => Solver::Auto,
        SolverArg::Native => Solver::Native,
        SolverArg::FlowCutter => Solver::FlowCutter,
    };

    eprintln!("Read {total} functions, computing treewidth in parallel…");

    let completed = AtomicUsize::new(0);
    let rows: Vec<Row> = records
        .into_par_iter()
        .map(|rec| {
            let tw = solver
                .compute(&Graph::from_edges(rec.blocks, &rec.edges))
                .map_err(|e| eprintln!("WARN: '{}': {e}", rec.name))
                .ok();

            if cli.progress > 0 {
                let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
                if done % cli.progress == 0 || done == total {
                    eprintln!("[{done}/{total}]");
                }
            }

            Row {
                name: rec.name,
                krate: rec.krate,
                blocks: rec.blocks,
                edges: rec.edges.len(),
                stmt_count: rec.stmt_count,
                treewidth: tw,
                is_unsafe: rec.is_unsafe,
            }
        })
        .collect();

    write_csv(&cli.outdir.join("results.csv"), &rows)?;
    print_top(&rows, 20);

    let summary = build_summary(&rows);
    let json = serde_json::to_string_pretty(&summary)?;
    fs::write(&cli.outdir.join("summary.json"), &json)?;
    eprintln!("JSON → {}", cli.outdir.join("summary.json").display());
    summary.print();

    Ok(())
}

fn read_records(path: &PathBuf, limit: usize) -> anyhow::Result<Vec<FunctionRecord>> {
    let reader = BufReader::new(
        File::open(path).unwrap_or_else(|e| panic!("cannot open '{}': {e}", path.display())),
    );
    let records = reader
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let line = line.expect("I/O error");
            if line.trim().is_empty() {
                return None;
            }
            Some(
                serde_json::from_str(&line)
                    .unwrap_or_else(|e| panic!("parse error on line {}: {e}", i + 1)),
            )
        })
        .take(if limit > 0 { limit } else { usize::MAX })
        .collect();
    Ok(records)
}

fn write_csv(path: &PathBuf, rows: &[Row]) -> anyhow::Result<()> {
    let mut w = BufWriter::new(
        File::create(path).unwrap_or_else(|e| panic!("cannot create '{}': {e}", path.display())),
    );
    writeln!(w, "name,crate,blocks,edges,stmt_count,treewidth,is_unsafe")?;
    for row in rows {
        let tw = row
            .treewidth
            .map(|v| v.to_string())
            .unwrap_or_else(|| "timeout".into());
        let name = row.name.replace('"', "\"\"");
        writeln!(
            w,
            "\"{name}\",{},{},{},{},{tw},{}",
            row.krate, row.blocks, row.edges, row.stmt_count, row.is_unsafe as u8
        )?;
    }
    w.flush()?;
    eprintln!("CSV  → {}", path.display());
    Ok(())
}

fn build_summary(rows: &[Row]) -> stats::Summary {
    let mut acc = Accumulator::default();
    for row in rows {
        acc.add(row.treewidth, row.is_unsafe);
    }
    acc.into_summary()
}

fn print_top(rows: &[Row], n: usize) {
    let mut solved: Vec<&Row> = rows.iter().filter(|r| r.treewidth.is_some()).collect();
    solved.sort_by_key(|r| std::cmp::Reverse(r.treewidth.unwrap()));
    println!("\nTop {n} highest treewidth:");
    println!("{:<5}  {:<7}  name", "tw", "blocks");
    for row in solved.iter().take(n) {
        println!(
            "{:<5}  {:<7}  {}",
            row.treewidth.unwrap(),
            row.blocks,
            row.name
        );
    }
}
