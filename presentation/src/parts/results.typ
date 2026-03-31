#import "../theme.typ": *

// ═══════════════════════════════════════════════
//  Part 3 — Results
// ═══════════════════════════════════════════════

#section-slide("Results", subtitle: "Experimental Findings")

#tslide(title: "Setup")[
  - Target: Rust standard library for `aarch64-apple-darwin`
  - Corpus: `core`, `alloc` and `std`
  - Treewidth computed on MIR CFG using *FlowCutter* with timeout
]

#tslide(title: "Functions")[
  - Total: 27,997 
  - Safe: 22,273 
  - Unsafe: 5,724
  - Includes: monomorphised generics
]

#tslide(title: "Summary Results")[
  #set text(size: 0.8em)
  #table(
    columns: (1.7fr, 1fr, 0.8fr, 0.8fr, 0.8fr, 1fr),
    fill: tbl-fill, stroke: none, inset: (x: 0.7em, y: 0.4em),
    th[Group], th[Count], th[Mean], th[Median], th[Max], th[$<= 3$],
    [All], [27,997], hi[1.04], [1], [6], hi[99.65%],
    [Safe], [22,273], [1.06], [1], [6], [99.59%],
    [Unsafe], [5,724], [0.97], [1], [4], [99.90%],
  )

  #v(0.5em)
  #callout[
    Mean treewidth #hi[1.04] — the vast majority of Rust functions
    have near-tree-like CFGs.
  ]
]

#tslide(title: "Treewidth Distribution")[
  #set text(size: 0.8em)
  #table(
    columns: (1.2fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr),
    fill: tbl-fill, stroke: none, inset: (x: 0.6em, y: 0.4em),
    th[tw], th[0], th[1], th[2], th[3], th[4], th[5], th[6],
    [%], hi[13.76], hi[70.63], [13.51], [1.75], [0.31], [0.03], [0.004],
    [cum.], [13.76], [84.39], [97.90], [99.65], [99.96], [99.99], [100],
    [count.], hi[3,853], hi[19,773], [3,783], [491], [90], [5], [2],
  )

  #v(0.5em)
  - Total *27,997* functions
  - #hi[84%] of functions have treewidth $<= 1$ (trees or forests)
  - Only *97 functions* (0.35%) exceed treewidth 3
]

#tslide(title: "Safe vs. Unsafe")[
  #v(0.3em)
  - Unsafe functions have slightly *lower* mean treewidth (0.97 vs 1.06)
  - Max treewidth: safe = *6*, unsafe = *4*
  - Unsafe Rust does *not* exhibit higher structural complexity
]

#tslide(title: "Comparison with Java")[
  #set text(size: 0.85em)
  #table(
    columns: (1.5fr, 1fr, 1fr, 1fr, 1fr),
    fill: tbl-fill, stroke: none, inset: (x: 0.7em, y: 0.4em),
    th[Study], th[Functions], th[Mean], th[Max], th[$<= 3$],
    [*Rust* (MIR)], [27,997], hi[1.04], [6], [99.65%],
    [*Java* (source)], [9,387], [2.7], [5], [95%],
  )

  #v(0.5em)
  - Rust's mean treewidth is significantly lower
  - But: different methodology — MIR-level vs source-level analysis
]
