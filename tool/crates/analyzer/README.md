# analyzer

Reads the JSONL produced by `mir-extractor`, computes the treewidth of each
function's CFG, and writes a CSV file plus a summary to stdout.

## Usage

```bash
cargo build --release -p analyzer

./target/release/analyzer mir-data.jsonl --outdir results
```

## Solver

Production runs use the native Rust heuristic solver:

| Variant | Flag | Description |
|---|---|---|
| `Native` | `--solver native` (default) | Pure Rust min-degree + min-fill elimination |

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

### FlowCutter oracle

`flow-cutter-pace17` is a git submodule at `../../flow-cutter-pace17` relative
to this crate. If the submodule is initialised (`git submodule update --init`),
`build.rs` detects the `src/` directory, compiles the C++ sources, and links
them into the crate via FFI as an optional verification oracle.

Pass `--verify-oracle` to cross-check every native result against the oracle.
The command exits non-zero if any mismatch is found.

## Output

CSV columns: `name, crate, blocks, edges, stmt_count, treewidth, is_unsafe`

A summary is printed to stdout including mean, median, max, distribution
histogram, and a comparison to the Java stdlib results from Gustedt et al.
(2002).

## Parallelism

Native solver invocations run in parallel via rayon (one per CPU core by default).
Set `RAYON_NUM_THREADS` to limit concurrency.

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `RAYON_NUM_THREADS` | number of CPUs | Parallel solver threads |
