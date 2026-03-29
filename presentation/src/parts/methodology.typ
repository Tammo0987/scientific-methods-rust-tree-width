#import "../theme.typ": *

// ═══════════════════════════════════════════════
//  Part 2 — Methodology
// ═══════════════════════════════════════════════

#let pipe-box(title, sub, active: false) = block(
  fill: if active { accent } else { surface },
  inset: (x: 0.8em, y: 0.8em),
  radius: 5pt,
  width: 100%,
)[
  #text(fill: if active { bg } else { text-cl }, weight: "bold")[#title] \
  #v(0.15em)
  #text(fill: if active { bg } else { subtext }, size: 0.8em)[#sub]
]

#let pipeline(active: none) = {
  v(1.2em)
  grid(
    columns: (1fr, 0.25fr, 1fr, 0.25fr, 1fr, 0.25fr, 1fr),
    align: center + horizon,
    pipe-box("Rust Crate", "source code",       active: active == "input"),
    text(fill: accent, size: 1.6em)[$arrow.r$],
    pipe-box("mir-extractor", "CFG extraction", active: active == "extractor"),
    text(fill: accent, size: 1.6em)[$arrow.r$],
    pipe-box("FlowCutter", "treewidth solver",  active: active == "flowcutter"),
    text(fill: accent, size: 1.6em)[$arrow.r$],
    pipe-box("Results", "per function",         active: active == "results"),
  )
}

// ───────────────────────────────────────────────

#section-slide("Methodology", subtitle: "Tools & Approach")

#tslide(title: "Pipeline Overview")[
  #pipeline()
]

#tslide(title: "Pipeline Overview")[
  #pipeline(active: "input")
]

#tslide(title: "Corpus — Rust Standard Library")[
  - We analyse *core*, *alloc*, and *std* combined
  - Generic monomorphizations compiled as separate functions
  - *27,997* functions total

  #v(0.5em)

  #set text(size: 0.85em)
  #table(
    columns: (1.5fr, 1fr),
    fill: tbl-fill, stroke: none, inset: (x: 0.8em, y: 0.35em),
    th[Category], th[Count],
    [Safe functions], [22,273],
    [Unsafe functions], [5,724],
    text(weight: "bold")[Total], text(weight: "bold")[27,997],
  )
]

#tslide(title: "Pipeline Overview")[
  #pipeline(active: "extractor")
]

#tslide(title: "Why MIR?")[
  - Rust's *Mid-level Intermediate Representation*
  - The compiler's own explicit CFG: basic blocks + terminators
  - Level at which *borrow checking* and flow-sensitive analyses run
  - Captures all control flow including desugared `match`, `?`, `async`

  #v(0.5em)
  #callout[
    MIR gives us the _actual_ graph the compiler reasons about —
    more representative than source-level analysis.
  ]
]

#tslide(title: "Step 1 — CFG Extraction")[
  *mir-extractor*: a custom `rustc` compiler driver

  #v(0.5em)

  + Hooks into the compiler at the *`after_analysis`* phase
  + Extracts the CFG from *optimised MIR* for every function
  + *Cleanup/unwind edges excluded* — only normal execution paths
    #text(fill: subtext, size: 0.82em)[(analogous to Gustedt et al. ignoring Java exceptions)]
  + Multi-edges deduplicated, treated as *undirected*

  #v(0.4em)
  #text(fill: subtext, size: 0.82em)[
    Inspired by Prusti (Astrauskas et al., 2022) and Budde (2024).
  ]
]

#tslide(title: "Step 1 — MIR Example")[
  #set text(size: 0.75em)
  #grid(
    columns: (1fr, 1.2fr),
    gutter: 1.2em,
    [
      *Rust source*
      #v(0.3em)
      #code-block(lang: "rust",
"fn max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}")
    ],
    [
      *MIR (simplified)*
      #v(0.3em)
      #code-block(
"bb0: switchInt(a > b)
        -> [true: bb1, false: bb2]
bb1: _0 = a; goto -> bb3
bb2: _0 = b; goto -> bb3
bb3: return")
    ],
  )

  #v(0.5em)
  #text(fill: subtext, size: 0.85em)[
    4 basic blocks, diamond-shaped CFG $=>$ treewidth 2
  ]
]

#tslide(title: "Pipeline Overview")[
  #pipeline(active: "flowcutter")
]

#tslide(title: "Step 2 — Treewidth via FlowCutter")[
  *FlowCutter* #text(fill: subtext, size: 0.82em)[(Strasser, 2017)]: a practical treewidth solver

  #v(0.5em)

  - *Anytime algorithm*: returns best decomposition found so far
  - *Deterministic*: repeated runs produce identical results
  - Runs in *parallel* — one solver per CPU core
  - *30-second timeout* per function

  #v(0.5em)
  #callout[
    Chosen because treewidth computation is NP-hard in general —
    FlowCutter finds good decompositions quickly in practice.
  ]
]

#tslide(title: "Pipeline Overview")[
  #pipeline(active: "results")
]
