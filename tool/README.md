# rust-treewidth

Empirical treewidth analysis of Rust control-flow graphs extracted from MIR.

## How it works

```
rustc (MIR)  ->  mir-extractor  ->  JSONL  ->  analyzer  ->  CSV + summary.json
```

For exact toolchain, dependency, and environment metadata which you can use to reproduce the results in the paper,
see [`REPRODUCIBILITY.md`](./REPRODUCIBILITY.md).

| Tool | Description |
|---|---|
| `mir-extractor` | Custom `rustc` driver, extracts CFG of every function after MIR optimisation |
| `analyzer` | Reads JSONL, computes treewidth in parallel, writes `results.csv` and `summary.json` |

## Usage

```bash
./analyze.sh <crate-name> [analyzer-flags...]
```

Outputs go to `results/<crate-name>/`. Analyzer flags such as `--verify-oracle` control verification behavior.

```bash
./analyze.sh test-crate
./analyze.sh std
./analyze.sh test-crate --verify-oracle
```

The special crate name `std` extracts `core + alloc + std` via `-Z build-std`.
By default this uses your host target triple.

## macOS setup

This project requires `nightly` + rustc internals for `mir-extractor`.

```bash
export PATH="$HOME/.cargo/bin:$PATH"
rustup toolchain install nightly
rustup component add --toolchain nightly rustc-dev rust-src llvm-tools-preview
```

If you need a non-host target for `std` extraction, set:

```bash
ANALYZE_TARGET=<target-triple> ./analyze.sh std
```

## FlowCutter oracle setup

`flow-cutter-pace17` is included as a git submodule at `./flow-cutter-pace17/`.
When cloning this repository, initialise it with:

```bash
git clone --recurse-submodules <repo-url>
```

Or, if you already have a clone without the submodule:

```bash
git submodule update --init
```

The analyzer will automatically compile the C++ sources and link them via FFI
as a verification oracle. Then run:

```bash
./analyze.sh std --verify-oracle
```

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
