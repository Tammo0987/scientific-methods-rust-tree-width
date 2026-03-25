#import "../theme.typ": *

// ═══════════════════════════════════════════════
//  Part 4 — Discussion, Limitations, Future Work
// ═══════════════════════════════════════════════

#section-slide("Discussion", subtitle: "Limitations & Future Work")

#tslide(title: "Why the Numbers Differ — Decomposition")[
  Thorup/Gustedt analyse at *source level*:
  - Base graph is *series-parallel* (treewidth 2)
  - Every method starts at treewidth $>= 2$

  #v(0.4em)

  We analyse at *MIR level*:
  - Flat list of basic blocks, no nesting structure
  - Trivial functions can have treewidth *0* or *1*

  #v(0.4em)
  #callout[
    The approaches measure *different things* — a direct numerical
    comparison requires caution.
  ]
]

#tslide(title: "Example — A Simple Function")[
  #set text(size: 0.72em)
  #grid(
    columns: (1fr, 1fr),
    gutter: 1.2em,
    [
      *Java (source-level)*
      #v(0.3em)
      #code-block(lang: "java",
"int add(int a, int b) {
    return a + b;
}")
      #v(0.3em)
      Series-parallel decomposition \
      $=>$ treewidth #hi[$>= 2$]
    ],
    [
      *Rust (MIR-level)*
      #v(0.3em)
      #code-block(
"bb0: _0 = a + b; return")
      #v(0.3em)
      Single basic block \
      $=>$ treewidth #hi[$= 0$]
    ],
  )

  #v(0.5em)
  #text(fill: subtext, size: 0.85em)[
    The same logic yields fundamentally different treewidths
    depending on the level of analysis.
  ]
]

#tslide(title: "Limitations")[
  + *Scope* — only the Rust standard library, not third-party crates
  + *Single configuration* — one compiler version, one target triple
  + *MIR-level* — results reflect post-optimization IR,
    not the source structure programmers write
  + *Undirected graphs* — cleanup/unwind edges removed
    (analogous to Java ignoring exceptions)
  + *No formal bound* — MIR erases source-level nesting,
    preventing a Thorup-style proof
]

#tslide(title: "Future Work")[
  - *Broader corpus*: analyse popular crates from crates.io
  - *HIR-level analysis*: study Rust's High-level IR for
    source-level comparison with Java
  - *Restricted fragments*: Rust without labeled breaks or async —
    derive Thorup-style theoretical bounds
  - *Compiler integration*: prototype treewidth-based optimizations
    in the Rust compiler
]
