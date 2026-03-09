# rust-treewidth

Empirical treewidth analysis of Rust control-flow graphs extracted from MIR.

## How it works

```
rustc (MIR)  ->  mir-extractor  ->  JSONL  ->  analyzer  ->  CSV + summary.json
```

| Tool | Description |
|---|---|
| `mir-extractor` | Custom `rustc` driver, extracts CFG of every function after MIR optimisation |
| `analyzer` | Reads JSONL, computes treewidth in parallel, writes `results.csv` and `summary.json` |

## Usage

```bash
./analyze.sh <crate-name> [analyzer-flags...]
```

Outputs go to `results/<crate-name>/<solver>/`. The solver subfolder reflects `--solver` (default: `auto`).

```bash
./analyze.sh test-crate
./analyze.sh std
./analyze.sh test-crate --solver native
```

The special crate name `std` extracts `core + alloc + std` via `-Z build-std`.

## Project layout

```
crates/
  mir-extractor/   custom rustc driver (MIR -> JSONL)
  analyzer/        JSONL -> treewidth -> CSV + summary.json
test-crate/        validation crate (11 control-flow examples)
analyze.sh         full pipeline script
results/           output
```

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `MIR_OUTPUT` | stdout | Path to JSONL output file for `mir-extractor` |
| `MIR_CRATES` | (all) | Comma-separated crate allowlist for `mir-extractor` |
| `RAYON_NUM_THREADS` | CPUs | Parallel solver threads in `analyzer` |
