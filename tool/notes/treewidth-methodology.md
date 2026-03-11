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
