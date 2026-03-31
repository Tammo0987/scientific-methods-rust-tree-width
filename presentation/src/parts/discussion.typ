#import "../theme.typ": *

#let mini-card(fill: rgb("#252535"), body) = block(
  fill: fill,
  inset: (x: 0.8em, y: 0.7em),
  radius: 5pt,
  width: 100%,
  body,
)

#let mini-node(fill: surface, stroke: accent, text-fill: text-cl, body) = block(
  fill: fill,
  stroke: 0.7pt + stroke,
  inset: (x: 0.65em, y: 0.38em),
  radius: 4pt,
  text(fill: text-fill, body),
)

// ═══════════════════════════════════════════════
//  Part 4 — Discussion, Limitations, Future Work
// ═══════════════════════════════════════════════

#section-slide("Discussion", subtitle: "Limitations & Future Work")

#tslide(title: "Why the Numbers Differ — Decomposition")[
  #set text(size: 0.89em)
  Thorup/Gustedt analyse at *source level*:
  - Structured control flow is preserved
  - Entry/exit stay together, so methods start at treewidth $>= 2$

  #v(0.05em)

  We analyse at *MIR level*:
  - Straight-line code is collapsed into basic blocks
  - Trivial functions can have *treewidth 0* or *1*

  #callout[
    Direct numerical comparison requires caution.
  ]
]

#tslide(title: "Same Logic")[
  #set text(size: 1.0em)
  #grid(
    columns: (0.22fr, 0.78fr),
    column-gutter: 1.2em,
    row-gutter: 0.6em,
    align: horizon,
    [
      *Java*
      #v(0.15em)
      #text(fill: subtext)[stdlib code]
    ],
    [
      #code-block(lang: "java",
"boolean isNaN(double v) {
    return v != v;
}")
    ],
  )

  #v(0.8em)

  #grid(
    columns: (0.22fr, 0.78fr),
    column-gutter: 1.2em,
    row-gutter: 0.6em,
    align: horizon,
    [
      *Rust*
      #v(0.15em)
      #text(fill: subtext)[stdlib code]
    ],
    [
      #code-block(lang: "rust",
"pub const fn is_nan(self) -> bool {
    self != self
}")
    ],
  )
]

#tslide(title: "Internal Representation")[
  #set text(size: 0.96em)
  #grid(
    columns: (0.22fr, 0.78fr),
    column-gutter: 1.2em,
    row-gutter: 0.6em,
    align: horizon,
    [
      *Java*
      #v(0.15em)
      #text(fill: subtext)[source-level view]
    ],
    [
      #mini-card[
        #align(center)[
          #grid(
            columns: (auto, auto, auto, auto, auto),
            gutter: 0.35em,
            [#mini-node(fill: rgb("#2b3046"), stroke: accent)[entry]],
            [#text(fill: subtext)[→]],
            [#mini-node(fill: rgb("#2b3046"), stroke: accent)[test]],
            [#text(fill: subtext)[→]],
            [#mini-node(fill: rgb("#2b3046"), stroke: accent)[exit]],
          )
        ]
      ]
    ],
  )

  #v(0.8em)

  #grid(
    columns: (0.22fr, 0.78fr),
    column-gutter: 1.2em,
    row-gutter: 0.6em,
    align: horizon,
    [
      *Rust*
      #v(0.15em)
      #text(fill: subtext)[MIR CFG]
    ],
    [
      #mini-card[
        #align(center)[
          #mini-node(fill: rgb("#1f3a31"), stroke: green, text-fill: text-cl)[bb0: compare + return]
        ]
      ]
    ],
  )
]

#tslide(title: "Treewidth")[
  #set text(size: 0.93em)
  #grid(
    columns: (0.22fr, 0.78fr),
    column-gutter: 1.2em,
    row-gutter: 0.6em,
    align: horizon,
    [
      *Java*
      #v(0.15em)
      #text(fill: subtext)[Thorup/Gustedt decomposition]
    ],
    [
      #mini-card(fill: rgb("#3b3140"))[
        #align(center)[
          #mini-node(fill: peach, stroke: peach, text-fill: bg)[entry, test, exit]
        ]
      ]
      #v(0.3em)
      #align(center)[treewidth #hi[$>= 2$]]
    ],
  )

  #v(0.9em)

  #grid(
    columns: (0.22fr, 0.78fr),
    column-gutter: 1.2em,
    row-gutter: 0.6em,
    align: horizon,
    [
      *Rust*
      #v(0.15em)
      #text(fill: subtext)[greedy elimination]
    ],
    [
      #mini-card(fill: rgb("#22372d"))[
        #align(center)[
          #mini-node(fill: rgb("#1f3a31"), stroke: green, text-fill: text-cl)[single remaining node]
        ]
      ]
      #v(0.3em)
      #align(center)[treewidth #hi[$= 0$]]
    ],
  )
]

#tslide(title: "Limitations")[
  + *Scope* — stdlib only
  + *Single configuration* — one compiler + target
  + *MIR-level* — compiler IR, not source structure
  + *Empirical only* — no Rust-specific theoretical bound yet
]

#tslide(title: "Future Work")[
  - *Broader corpus*: analyse crates beyond the stdlib
  - *HIR-level analysis*: compare Rust to Java at source level
  - *Restricted fragments*: derive Thorup-style bounds for Rust subsets
  - *Compiler integration*: leverage low treewidth in CFG optimization
]
