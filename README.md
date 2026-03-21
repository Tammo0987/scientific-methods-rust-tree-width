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

# Reproducibility

This project depends on a nightly Rust toolchain because `mir-extractor`
links against `rustc_private` crates and the `std` analysis path uses
`-Z build-std`.

The exact toolchain and environment used for the experiments in the paper are:
- Rust toolchain: `nightly-2026-03-10`
- `rustc`: `1.96.0-nightly (0c68443b0 2026-03-10)`
- `cargo`: `1.96.0-nightly (90ed291a5 2026-03-05)`
- Rust components required by this repo: `rustc-dev`, `rust-src`, `llvm-tools`, `rustfmt`, `clippy`, `rust-analyzer`
- Host/target triple used in this environment: `aarch64-apple-darwin`
- Operating system: `macOS 26.3.1 (build 25D2128)`
- Kernel: `Darwin 25.3.0`
- Native C++ compiler used to build the FlowCutter FFI bridge: `Apple clang 17.0.0 (clang-1700.6.4.2)`
- FlowCutter oracle source revision: `7f94541b0119284ea9322d528cef420e041539b6`

The Rust dependency closure is pinned in `Cargo.lock`. The most relevant direct crate versions are:
- `serde 1.0.228`
- `serde_json 1.0.149`
- `clap 4.5.60`
- `rayon 1.11.0`
- `cc 1.2.56`

# Treewidth Methodology Notes

## On Treewidth 0

Treewidth 0 is mathematically valid and not a bug. A graph has treewidth 0 if and only if it
is edgeless. The optimal tree decomposition places each node in its own bag of size 1, giving
width = 1 − 1 = 0.

In our context, this corresponds to MIR functions with a single basic block and no control-flow
edges — trivially straight-line functions with no branches or loops. Examples from `std`:

```
f128::is_nan      blocks=1, edges=0, stmt_count=1, tw=0
f128::to_bits     blocks=1, edges=0, stmt_count=1, tw=0
```

About 11% of std functions (3352 / 29704) fall into this category. This is plausible for simple
numeric wrappers, getters, and delegating functions in a standard library.

Treewidth reference:

| Graph | tw |
|---|---|
| Edgeless (isolated nodes) | 0 |
| Path / tree | 1 |
| Cycle C4, simple grid | 2 |
| K4 (complete on 4 nodes) | 3 |

---

## Comparison with Gustedt, Mæhle, Telle (2001)

**"The Treewidth of Java Programs"**, INRIA RR-4318.
They analyze the Java standard library and report a minimum treewidth of 2, average ~2.7.

### Why their minimum is 2

Their approach differs from ours in two fundamental ways:

**1. Different graph granularity**
They work at the source token/statement level, not the basic block level. Their CFG nodes are
individual keywords and statements (e.g. `while`, `endwhile`, `if`, `break`). Every method has
many nodes even if trivially simple.

Our tool collapses all straight-line code into a single MIR basic block, giving potentially fewer
nodes and lower treewidth.

**2. Different algorithm — Thorup's structural decomposition**
They implement Thorup's constructive proof (*"All Structured Programs have Small Tree-Width and
Good Register Allocation"*, 1998) rather than graph-theoretic elimination. The decomposition
reflects source-level nesting structure. Every method has an entry/exit pair that always appears
in the same bag (size ≥ 3 → width ≥ 2), making 2 the structural minimum.

Each flow-affecting construct (FAC) adds at most +1 to treewidth:

| Construct | Treewidth contribution |
|---|---|
| Base (no FACs, series-parallel) | 2 |
| `return` | +1 |
| `break` | +1 |
| `continue` | +1 |
| Short-circuit `&&` / `\|\|` | +1 |
| Labelled `break`/`continue` | potentially unbounded |

Their empirical results: ~20–40% of methods at tw=2, ~60–80% at tw=3, <1% at tw=4+.

---

## Accuracy Comparison

Neither approach computes exact treewidth (NP-hard). Both produce upper bounds.

**Our greedy elimination on MIR CFG:**

| | |
|---|---|
| + | Closer to actual graph-theoretic treewidth |
| + | Correctly identifies trivial functions (tw=0, tw=1) |
| + | Tighter upper bounds than structural decomposition |
| + | Works directly on existing MIR data |
| − | MIR has already lost source structure (loops desugared to jumps) |
| − | Greedy heuristics can still overestimate |
| − | Not directly comparable to Gustedt/Thorup literature results |

**Gustedt/Thorup structural decomposition:**

| | |
|---|---|
| + | Directly connected to Thorup's theoretical bounds |
| + | Linear time, computable during parsing |
| + | Reflects programmer-visible nesting and structural complexity |
| + | Comparable to Java paper and other literature |
| − | Minimum tw=2 by construction — hides trivially simple functions |
| − | Can significantly overestimate actual graph treewidth |
| − | Requires source-level traversal (HIR/AST), not applicable to MIR |

---

## Future Work: Structural Decomposition for Rust

Implementing Thorup's structural decomposition for Rust would require working at the HIR
(High-level IR) level, where structured control flow is still intact, rather than MIR where it
has been desugared into basic blocks.

Rust-specific FACs and their theoretical treewidth contributions:

| Construct | +tw |
|---|---|
| `return` | +1 |
| `break` | +1 |
| `continue` | +1 |
| `?` operator | +1 (multiple exit paths) |
| Short-circuit `&&` / `\|\|` | +1 |
| Labeled `break`/`continue` | potentially unbounded |

**Implementation estimate:** a few hundred lines in a new HIR-level extractor crate, plus the
structural decomposition logic. Medium complexity, primarily due to rustc HIR API familiarity.
Closures and `match` arms require careful handling (separate scope vs. inline).

This comparison would be scientifically valuable for several reasons:

- **Methodological comparison**: whether the two approaches correlate, and which better predicts
  compiler optimization difficulty in practice
- **Granularity question**: how much treewidth information is lost when MIR desugars structured
  control flow into basic blocks
- **Rust-specific FACs**: how `?`, labeled breaks, and closures affect the structural bound
  compared to Java, and whether Rust's tw bound is similar to C's (≤6)

