# analyzer

Reads the JSONL produced by `mir-extractor`, computes the treewidth of each
function's CFG, and writes a CSV file plus a summary to stdout.

## Usage

```bash
cargo build --release -p analyzer

./target/release/analyzer mir-data.jsonl --output results.csv
```

## Solvers

Treewidth computation is abstracted behind a `Solver` enum with three variants,
selectable via `--solver`:

| Variant | Flag | Description |
|---|---|---|
| `Auto` | `--solver auto` (default) | Native for ≤500 nodes, FlowCutter for larger |
| `Native` | `--solver native` | Pure Rust, no subprocess |
| `FlowCutter` | `--solver flowcutter` | FlowCutter subprocess |

For the CFG graphs in this study (typically < 100 nodes) `auto` always uses the
native solver.

### Native solver

Implements the two greedy elimination heuristics that FlowCutter
(`greedy_order.cpp`) runs for small graphs:

**Min-degree** (`compute_greedy_min_degree_order`): repeatedly eliminate the
vertex with the smallest current degree.

**Min-fill** (`compute_greedy_min_shortcut_order`): repeatedly eliminate the
vertex that requires adding the fewest new edges (fill-in edges).  Tiebreak by
degree.  The `* 100` weight on fill count is copied directly from FlowCutter's
priority criterion.

Both heuristics produce an upper bound on treewidth.  We run both and take the
minimum.  For CFG-sized graphs these almost always give the exact treewidth.

The core operation in both is **vertex elimination**: remove vertex v, connect
all its neighbours to each other (add fill-in edges), and continue.  The
treewidth upper bound is the maximum degree of any vertex at the time it is
eliminated.  Adjacency lists are kept sorted throughout to make the set-union
merge in each elimination step efficient.

**Complexity**: O(n³) in the worst case, versus FlowCutter's O(n log n) with a
min-heap.  The simpler implementation is sufficient for graphs under ~500 nodes.

**Deliberate omission**: FlowCutter returns an explicit tree decomposition.  We
only need the width, so we compute it inline during elimination and skip
constructing the decomposition.

### FlowCutter subprocess

Invokes `flow_cutter_pace17 <tempfile>` directly (the C implementation from
kit-algo/flow-cutter-pace17).  Timeout is enforced in Rust, so no external GNU
`timeout` command is required on macOS/Linux.  On timeout, we send `SIGTERM`
first (to let FlowCutter print its best decomposition) and then hard-kill if
it does not exit promptly.  We parse the treewidth from bag lines rather than
the `s` header, because FlowCutter's field order deviates from the PACE 2017
spec.

### FlowCutter FFI (preferred)

If `../../flow-cutter-pace17/src` exists while building `analyzer`, build.rs
compiles the C++ sources and links them into analyzer. In this mode we call
FlowCutter C++ functions directly from Rust (no temp graph files, no process
spawn, no string parsing).

## Output

CSV columns: `name, crate, blocks, edges, stmt_count, treewidth, is_unsafe`

A summary is printed to stdout including mean, median, max, distribution
histogram, and a comparison to the Java stdlib results from Gustedt et al.
(2002).

## Parallelism

Solver invocations run in parallel via rayon (one per CPU core by default).
Set `RAYON_NUM_THREADS` to limit concurrency.

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `FLOW_CUTTER_BIN` | `flow_cutter_pace17` | FlowCutter binary name or path |
| `FLOW_CUTTER_TIMEOUT_SECS` | `30` | Per-function time limit (FlowCutter only) |
| `RAYON_NUM_THREADS` | number of CPUs | Parallel solver processes |

When `--solver flow-cutter` is selected and FFI is not available, analyzer
checks `FLOW_CUTTER_BIN` up front and exits immediately if the binary is
missing.
