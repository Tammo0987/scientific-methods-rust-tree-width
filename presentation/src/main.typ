#import "./theme.typ": *

#set page(
  paper: "presentation-16-9",
  fill: bg,
  margin: (x: 2.5em, y: 2em),
)
#set text(fill: text-cl, size: 24pt)
#set raw(theme: "catppuccin-mocha.tmTheme")
#set list(
  marker: (
    text(fill: accent)[▸],
    text(fill: subtext)[--],
  ),
  spacing: 1.0em,
)
#set enum(spacing: 1.0em)

#set page(footer: context [
  #place(right, dy: 0.4em,
    text(fill: text-cl, weight: "semibold", size: 0.7em)[
      #toolbox.slide-number / #toolbox.last-slide-number
    ]
  )
])

// ── Title slide ──
#slide(
  align(center + horizon)[
    #text(fill: accent, size: 1.9em, weight: "bold")[
      Analysing the Treewidth of Rust Programs
    ]
    #v(0.4em)
    #text(fill: subtext, size: 0.95em)[Battini, Haans, Singh, Steffens]
    #v(1.2em)
    #line(length: 55%, stroke: 0.5pt + accent)
    #v(0.7em)
    #text(fill: subtext, size: 0.8em)[Scientific Methods · 2026]
  ]
)

// ── Parts ──
#include "./parts/introduction.typ"
#include "./parts/methodology.typ"
#include "./parts/results.typ"
#include "./parts/discussion.typ"

// ── Closing ──
#slide(
  align(center + horizon)[
    #text(fill: accent, size: 1.5em, weight: "bold")[
      Thank you for listening!
    ]
    #v(0.8em)
    #text(fill: subtext, size: 0.95em)[Questions?]
  ]
)
