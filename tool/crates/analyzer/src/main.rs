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
use treewidth::{compute, compute_oracle, ensure_oracle_available};

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

    /// Cross-check native results against the FlowCutter FFI oracle.
    #[arg(long, default_value_t = false)]
    verify_oracle: bool,
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
    treewidth: u32,
    oracle_treewidth: Option<u32>,
    oracle_error: Option<String>,
    is_unsafe: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    fs::create_dir_all(&cli.outdir)?;

    let records = read_records(&cli.input, cli.limit)?;
    let total = records.len();
    if cli.verify_oracle {
        ensure_oracle_available()?;
        eprintln!("Oracle verification enabled: cross-checking native results against FlowCutter FFI.");
    }

    eprintln!("Read {total} functions, computing treewidth in parallel…");

    let completed = AtomicUsize::new(0);
    let rows: Vec<Row> = records
        .into_par_iter()
        .map(|rec| {
            let graph = Graph::from_edges(rec.blocks, &rec.edges);
            let tw = compute(&graph);
            let (oracle_treewidth, oracle_error) = if cli.verify_oracle {
                match compute_oracle(&graph) {
                    Ok(value) => (Some(value), None),
                    Err(e) => (None, Some(e.to_string())),
                }
            } else {
                (None, None)
            };

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
                oracle_treewidth,
                oracle_error,
                is_unsafe: rec.is_unsafe,
            }
        })
        .collect();

    if cli.verify_oracle {
        let oracle_errors = collect_oracle_errors(&rows);
        if !oracle_errors.is_empty() {
            for line in oracle_errors.iter().take(20) {
                eprintln!("ORACLE ERROR: {line}");
            }
            anyhow::bail!(
                "oracle verification failed because {} function(s) could not be checked",
                oracle_errors.len()
            );
        }

        let mismatches = collect_oracle_mismatches(&rows);
        if !mismatches.is_empty() {
            for line in mismatches.iter().take(20) {
                eprintln!("MISMATCH: {line}");
            }
            anyhow::bail!(
                "oracle verification failed for {} function(s)",
                mismatches.len()
            );
        }

        eprintln!(
            "Oracle verification passed: checked {} function(s), 0 mismatches.",
            rows.len()
        );
    }

    write_csv(&cli.outdir.join("results.csv"), &rows)?;
    print_top(&rows, 20);

    let summary = build_summary(&rows);
    let json = serde_json::to_string_pretty(&summary)?;
    fs::write(&cli.outdir.join("summary.json"), &json)?;
    eprintln!("JSON → {}", cli.outdir.join("summary.json").display());
    summary.print();

    Ok(())
}

fn collect_oracle_mismatches(rows: &[Row]) -> Vec<String> {
    rows.iter()
        .filter_map(|row| match row.oracle_treewidth {
            Some(oracle) if row.treewidth != oracle => Some(format!(
                "{}: native={} oracle={}",
                row.name, row.treewidth, oracle
            )),
            _ => None,
        })
        .collect()
}

fn collect_oracle_errors(rows: &[Row]) -> Vec<String> {
    rows.iter()
        .filter_map(|row| {
            row.oracle_error
                .as_ref()
                .map(|err| format!("{}: {err}", row.name))
        })
        .collect()
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
        let name = row.name.replace('"', "\"\"");
        writeln!(
            w,
            "\"{name}\",{},{},{},{},{},{}",
            row.krate, row.blocks, row.edges, row.stmt_count, row.treewidth, row.is_unsafe as u8
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
    let mut solved: Vec<&Row> = rows.iter().collect();
    solved.sort_by_key(|r| std::cmp::Reverse(r.treewidth));
    println!("\nTop {n} highest treewidth:");
    println!("{:<5}  {:<7}  name", "tw", "blocks");
    for row in solved.iter().take(n) {
        println!("{:<5}  {:<7}  {}", row.treewidth, row.blocks, row.name);
    }
}
