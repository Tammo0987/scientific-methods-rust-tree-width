# mir-extractor

A custom `rustc` compiler driver that extracts control-flow graphs from MIR and
writes one JSON record per function to a JSONL file.

## How it works

`mir-extractor` is passed as the `RUSTC` binary to `cargo build`.  It behaves
like a normal compiler but hooks into the pipeline via `after_analysis` — after
MIR optimisation — to extract CFG data before compilation continues.

For every `fn` and associated function (`impl` method) in the crate being
compiled it:

1. Fetches `optimized_mir` — the post-optimisation MIR body.
2. Drops edges that lead into cleanup/unwind blocks, then remaps the remaining
   block indices to a compact 0-based space.
3. Collects the remaining directed edges, deduplicating multi-edges.
4. Serialises the result as a JSON record and appends it to `$MIR_OUTPUT`.

Closures, generators, constants and statics are skipped.

## Output format

One JSON record per line:

```json
{"name":"core::slice::sort::merge","crate":"core","blocks":12,"edges":[[0,1],[0,2],[1,3]],"stmt_count":34,"is_unsafe":false}
```

| Field | Description |
|---|---|
| `name` | Fully-qualified function path |
| `crate` | Crate the function belongs to |
| `blocks` | Number of non-cleanup basic blocks |
| `edges` | Directed CFG edges as `[src, tgt]` pairs (deduplicated) |
| `stmt_count` | Total MIR statements across all non-cleanup blocks |
| `is_unsafe` | Whether the function is declared `unsafe fn` |

## Usage

Build first, then point `RUSTC` at the binary:

```bash
cargo build --release -p mir-extractor

MIR_OUTPUT=$(pwd)/mir-data.jsonl \
RUSTC=$(pwd)/target/release/mir-extractor \
cargo build -p <your-crate>
```

For the Rust standard library, use `-Z build-std` with `test-crate` as the
trigger crate (any crate that depends on std will do):

```bash
MIR_OUTPUT=$(pwd)/stdlib-full.jsonl \
RUSTC=$(pwd)/target/release/mir-extractor \
cargo build -p test-crate \
  -Z build-std=core,alloc,std \
  --target x86_64-unknown-linux-gnu
```

`cargo build` must be re-run from scratch (or the target crate touched) if the
JSONL file was deleted — Cargo will not recompile unchanged crates.

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `MIR_OUTPUT` | stdout | Path to append JSONL records to |
| `MIR_CRATES` | (all) | Comma-separated crate allowlist; crates not in the list are skipped entirely |

## Extraction decisions

These decisions define the scope and shape of the extracted data.  Each one is
a deliberate methodological choice that should be stated in the paper.

**What is extracted**

Only items with `DefKind::Fn` or `DefKind::AssocFn` are processed — i.e. free
functions and methods defined in `impl` blocks.  This matches the unit of
analysis in Gustedt et al. (2002).

**What is excluded**

| Excluded | Reason |
|---|---|
| Closures (`DefKind::Closure`) | Anonymous, inflate function counts, not independently callable |
| Struct/enum constructors (`DefKind::Ctor`) | Trivial single-block bodies, not authored control flow |
| Constants and statics (`DefKind::Const`, `DefKind::Static`) | Evaluated at compile time, no runtime CFG |
| `extern fn` declarations | No body, no MIR |
| Edges into cleanup/unwind blocks (`is_cleanup`) | We remove edges that represent exceptional control flow (panic/unwind paths), not the blocks themselves.  In practice cleanup blocks are unreachable without these edges, so they vanish from the graph.  Gustedt et al. use a parse-tree representation in which exceptions are absent by construction (a `throws` declaration contributes no nodes or edges).  The effect is the same — both representations capture only normal execution paths — but the mechanism differs: they never model exceptional flow, we model it and drop it. |

**MIR stage**

`optimized_mir` is used rather than the pre-optimisation MIR.  This means the
extracted CFG reflects the code shape after inlining, copy propagation and
simplification — i.e. what actually executes.  A consequence is that generic
functions appear once per monomorphisation (e.g. `Vec<u8>::push` and
`Vec<String>::push` are separate entries).  This is intentional: each
monomorphisation has a distinct CFG and distinct treewidth.

**Multi-edges**

The terminator of a MIR basic block can list the same successor twice (e.g. a
`SwitchInt` with multiple arms jumping to the same target).  These are
deduplicated to produce a simple graph, as treewidth is defined on simple
graphs.

**Edge direction**

Edges are stored as directed pairs `(src, tgt)` but the treewidth computation
treats them as undirected (the `treewidth` crate symmetrises the adjacency
list).  The directed representation is preserved in the JSONL for potential
future use.

## Notes

- Requires the `rustc-dev` component (provided by `devenv.nix` via
  `rust-toolchain.toml`).
- Uses `#![feature(rustc_private)]` to link against rustc internals.
