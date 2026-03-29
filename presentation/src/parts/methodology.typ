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

#tslide(title: "Input — Rust Standard Library")[
  - A *crate* is Rust's unit of compilation (library or binary)
  - We analyse *core*, *alloc*, and *std*
  - Generic functions are compiled separately per type (*monomorphization*) \
    #text(fill: subtext, size: 0.82em)[e.g. `Vec<u8>::push` and `Vec<String>::push` are two separate functions]
  - *27,997* functions total
]

#tslide(title: "Pipeline Overview")[
  #pipeline(active: "extractor")
]

#tslide(title: "Why MIR?")[
  - Gustedt et al. had to construct CFGs from source (following Thorup's decomposition)
  - Rust's compiler already gives us MIR: its own explicit CFG
  - MIR is what the compiler actually works with: basic blocks + terminators
]

#tslide(title: "Step 1 — CFG Extraction")[
  *mir-extractor*: a custom `rustc` compiler driver

  #v(0.5em)

  + Hooks into the compiler at the *`after_analysis`* phase
  + Extracts the CFG from *optimised MIR* for every function
  + *Cleanup/unwind edges excluded*, only normal execution paths
    #text(fill: subtext, size: 0.82em)[(analogous to Gustedt et al. ignoring Java exceptions)]
  + Multi-edges deduplicated, treated as *undirected*

  #v(0.5em)
  #callout[
    Hooked in after optimisation, treewidth reflects pure control-flow complexity.
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
  - Runs in *parallel*, one solver per CPU core
  - *30-second timeout* per function

  #v(0.5em)
  #callout[
    Chosen because treewidth computation is NP-hard in general,
    FlowCutter finds good decompositions quickly in practice.
  ]
]

#tslide(title: "Pipeline Overview")[
  #pipeline(active: "results")
]
