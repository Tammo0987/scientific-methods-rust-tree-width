#import "@preview/polylux:0.4.0": *

// --- Catppuccin Mocha palette ---
#let bg      = rgb("#1e1e2e")
#let surface = rgb("#313244")
#let text-cl = rgb("#cdd6f4")
#let subtext = rgb("#a6adc8")
#let accent  = rgb("#89b4fa")
#let yellow  = rgb("#f9e2af")
#let peach   = rgb("#fab387")
#let green   = rgb("#a6e3a1")
#let red     = rgb("#f38ba8")

// --- Slide helpers ---
#let slide-title(t) = block(
  width: 100%,
  fill: surface,
  inset: (x: 1.2em, y: 0.65em),
  radius: (top-left: 5pt, top-right: 5pt, bottom-left: 0pt, bottom-right: 0pt),
  text(fill: accent, size: 1.25em, weight: "bold", t),
)

#let tslide(title: none, body) = slide({
  if title != none {
    slide-title(title)
    v(0.7em)
  }
  body
})

#let callout(body) = block(
  fill: surface,
  stroke: (left: 3pt + accent),
  inset: (x: 1.2em, y: 0.9em),
  radius: (right: 5pt),
  width: 100%,
  body,
)

#let code-block(lang: none, body) = block(
  fill: surface,
  inset: (x: 1.2em, y: 0.7em),
  radius: 5pt,
  width: 100%,
  if lang != none { raw(lang: lang, body) } else { raw(body) },
)

// --- Table helpers ---
#let th(x) = text(fill: accent, weight: "bold", x)
#let hi(x) = text(fill: peach, weight: "bold", x)
#let tbl-fill(col, row) = {
  if row == 0 { surface }
  else if calc.odd(row) { rgb("#252535") }
  else { bg }
}

// --- Section divider ---
#let section-slide(title, subtitle: none) = slide(
  align(center + horizon)[
    #text(fill: accent, size: 1.6em, weight: "bold")[#title]
    #if subtitle != none {
      v(0.5em)
      text(fill: subtext, size: 0.9em)[#subtitle]
    }
  ]
)
