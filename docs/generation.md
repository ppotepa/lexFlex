# Generation and Translation

`lexflex-generation` realizes entity and concept anchors from a validated
`LanguageModel`. Candidate forms are selected by language-package features and
deterministic priority; no surface form is hardcoded in Rust. Scalar semantic
values use their canonical representation. Structural `Satisfies`, `Equals`,
and `Apply` expressions recurse through the same realization path. Compound
`And`, `Or`, and `Not` realization is available through an explicit
`GenerationStyle`; callers provide language-specific separators, negation, and
quantifier prefixes rather than relying on hidden sentence rules.
Feature-constrained generation unifies requested features with the lexical
sense features before selecting a form. Conflicts are typed failures, never a
fallback to an incompatible surface form.
`GenerationBudget` bounds expression depth, node count, and realized output
bytes before and after generation.

Quantified expressions preserve their binder and body in the generated
structure. A language package can provide the desired prefixes through
`GenerationStyle`.

`TranslationRequest` accepts an already parsed semantic expression and uses the
same realization path for the target language. Unsupported expression shapes
return a typed error instead of falling back to text templates.

CLI command `text-generate` accepts a JSON `SemanticExpression` file. The
`text-translate` command retains that semantic-input form, while
`text-translate-text` parses source text with the normal engine analysis path,
then realizes the resulting semantic expression in the target language. Both
forms return JSON with optional trace data.
