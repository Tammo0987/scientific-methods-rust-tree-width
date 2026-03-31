#import "../theme.typ": *

// ═══════════════════════════════════════════════
//  Part 1 — Introduction, Preliminaries, Related Work
// ═══════════════════════════════════════════════

#section-slide("Introduction", subtitle: "Motivation & Background")

#tslide(title: "What Is This About?")[
  - Compilers use *control-flow graphs* (CFGs) to represent execution structure of a program
  - *Treewidth* measures how close a graph is to being a tree
  - Low treewidth $=>$ NP-hard problems admit to linear time
    - e.g. register allocation & data-flow analysis 

  #v(0.6em)
  #callout[
    *Question:* What is the treewidth of real-world Rust programs?
  ]
]

#tslide(title: "Recap — Treewidth")[
  A *tree decomposition* of graph maps nodes into _bags_ on a tree:

  #v(0.3em)

  + Every node appears in at least one bag
  + Every edge's endpoints share at least one bag
  + Bags containing a given node form a connected subtree

  #v(0.5em)
  *Treewidth* = (smallest max bag size over all decompositions) $- 1$

  #v(0.3em)
  #text(fill: subtext, size: 0.82em)[Treewidth 1 = tree · Treewidth 2 = series-parallel graph]
]

#tslide(title: "Prior Work — Thorup (1998)")[
  - Structured programs (`goto`-free) have *bounded* CFG treewidth
  - Results: 
    #text(size: 0.82em)[- Modula-2 $<= 5$]
    #text(size: 0.82em)[- Algol/Pascal $<= 3$]
    #text(size: 0.82em)[- Goto-free C: treewidth *$<= 6$*]
  - Each _flow-affecting construct_ (break, continue, return, short-circuit)
    raises treewidth by at most 1

  #v(0.4em)
]

#tslide(title: "Prior Work — Gustedt et al. (2002)")[
  - Empirical study on *Java standard library* (9,387 methods)
  - No `goto`-statements, but labelled break and continue statements are equivalent to `goto`
  - Restricting label-free subset
  - Theoretical treewidth: $2 + 4 = 6$
    - Exactly 4 FAC's & Base treewidth of 2, since the CFG is series-parallel
]


#tslide(title: "Prior Work — Gustedt et al. (2002)")[
  Empirical study on *Java standard library* (9,387 methods)
  #v(0.3em)

  #set text(size: 0.85em)
  #table(
    columns: (1.5fr, 1fr, 1fr, 1fr),
    fill: tbl-fill, stroke: none, inset: (x: 0.8em, y: 0.35em),
    th[Metric], th[Mean], th[Max], th[$<= 3$],
    [Label-free Java], [2.7], [5], [95%],
  )

  #v(0.4em)
  #set text(size: 1em)
  - Theoretical bound of 6 is *never reached* in practice
]

#tslide(title: "Our Contribution")[
  We perform the *analogous study for Rust*:

  #v(0.3em)

  - Tool to extract CFGs from the compiler's own representation (*MIR*)
  - Compute treewidth of all functions in the *Rust standard library*
  - Compare safe & unsafe Rust
  - Contextualize against the Java results
]
