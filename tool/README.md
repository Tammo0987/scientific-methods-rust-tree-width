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

## FlowCutter setup

Preferred path: place the `flow-cutter-pace17` source tree next to this repo
at `./flow-cutter-pace17/`. The analyzer will compile and call C++ functions
directly via FFI (no temp files, no subprocess, no output parsing).

```bash
git clone https://github.com/kit-algo/flow-cutter-pace17.git
cd flow-cutter-pace17
./build.sh
cd ..
```

Then run:

```bash
cd /path/to/this/tool
./analyze.sh std --solver flow-cutter
```

Fallback path: if the source tree is not present, the analyzer uses the
`flow_cutter_pace17` binary via subprocess. In that case set:

```bash
export FLOW_CUTTER_BIN=/absolute/path/to/flow_cutter_pace17
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
| `FLOW_CUTTER_BIN` | `flow_cutter_pace17` | FlowCutter executable name/path (used by `--solver flow-cutter`) |
| `FLOW_CUTTER_TIMEOUT_SECS` | `30` | Per-function timeout for FlowCutter |
| `RAYON_NUM_THREADS` | CPUs | Parallel solver threads in `analyzer` |
