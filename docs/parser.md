# Parser

The parser pipeline is tokenization, lexical candidate selection, chart
composition and complete-candidate validation. It emits `LinguaExpression`
inputs and preserves metrics, derivations and ambiguity. Every stage has an
explicit budget for tokens, chart items, derivations, depth and semantic nodes.
Invalid candidates are rejected locally; a valid sibling is not hidden by a
better-scored invalid candidate.
