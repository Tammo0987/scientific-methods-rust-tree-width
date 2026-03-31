#import "../theme.typ": *
#import "@preview/polylux:0.4.0": later, uncover, only

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
  - Rust's compiler already gives us MIR: its own explicit CFG of basic blocks + terminators
]

#tslide(title: "Step 1 — CFG Extraction")[
  *mir-extractor*: a custom `rustc` compiler driver

  #v(0.5em)

  + Hooks into the compiler, extracts CFG from *optimised MIR* for every function
  + *Cleanup/unwind edges excluded*, only normal execution paths
    #text(fill: subtext, size: 0.82em)[(analogous to Gustedt et al. ignoring Java exceptions)]
  + Multi-edges deduplicated, treated as *undirected* \
    #text(fill: subtext, size: 0.82em)[(e.g. `match x { 1 | 2 => ... }` produces two edges to the same block)]

]

#tslide(title: "Step 1 — CFG Example")[
  #set text(size: 0.8em)

  #let node(label, body) = block(
    fill: surface, inset: (x: 0.5em, y: 0.4em), radius: 4pt, width: 100%,
    align(center)[
      #set text(size: 0.85em)
      #text(fill: accent, weight: "bold")[#label] \
      #raw(body)
    ]
  )

  #grid(
    columns: (1.4fr, 1fr),
    align: (left + top, center + top),
    gutter: 1.5em,
    {
      set text(size: 0.75em)
      code-block(lang: "rust", "fn max(a: i32, b: i32) -> i32 {\n    if a > b { a } else { b }\n}")
    },
    align(center)[
      #block(width: 75%)[#node("bb0", "switchInt(a > b)")]
      #only("2-")[
        #v(0.2em)
        #grid(columns: (1fr, 1fr), gutter: 0.3em,
          align(right)[#text(fill: accent)[⤢]],
          align(left)[#text(fill: accent)[⤡]],
        )
        #v(0.2em)
        #grid(columns: (1fr, 1fr), gutter: 0.5em,
          node("bb1", "_0 = a"),
          node("bb2", "_0 = b"),
        )
      ]
      #only("3-")[
        #v(0.2em)
        #grid(columns: (1fr, 1fr), gutter: 0.3em,
          align(right)[#text(fill: accent)[⤡]],
          align(left)[#text(fill: accent)[⤢]],
        )
        #v(0.2em)
        #block(width: 75%)[#node("bb3", "return _0")]
      ]
    ]
  )
]

#tslide(title: "Pipeline Overview")[
  #pipeline(active: "flowcutter")
]

#tslide(title: "Step 2 — Treewidth via FlowCutter")[
  - Treewidth is *NP-hard*, implementing a solver ourselves is not feasible
  - *FlowCutter* #text(fill: subtext, size: 0.82em)[(Strasser, 2017)] already works well in practice for this kind of graph
  - We transform our CFGs to FlowCutter's input format and reuse it as a black box
]

#tslide(title: "Pipeline Overview")[
  #pipeline(active: "results")
]

