#import "../theme.typ": *

// ═══════════════════════════════════════════════
//  Part 2 — Methodology
// ═══════════════════════════════════════════════

#section-slide("Methodology", subtitle: "Tools & Approach")

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

#tslide(title: "MIR Example")[
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

#tslide(title: "Pipeline — mir-extractor")[
  #v(0.3em)
  Our tool is a *custom rustc compiler driver* (inspired by Prusti, Budde):

  #v(0.5em)

  + *Compile* the target crate with our driver instead of `rustc`
  + *Extract* CFGs from optimized MIR for every function
    - Cleanup/unwind edges excluded (analogous to Java ignoring exceptions)
    - Multi-edges deduplicated, treated as *undirected*
  + *Compute treewidth* using *FlowCutter* #text(fill: subtext, size: 0.82em)[(Strasser, 2017)]

  #v(0.5em)
  #text(fill: subtext, size: 0.82em)[
    Prusti (Astrauskas et al., 2022) and Budde (2024) demonstrated
    the viability of custom compiler drivers for MIR analysis.
  ]
]

#tslide(title: "Target — Rust Standard Library")[
  - We analyse *core*, *alloc*, and *std* combined
  - Includes generic monomorphizations as separate functions
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
