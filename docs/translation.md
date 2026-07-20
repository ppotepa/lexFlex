# Translation

Translation is not a second parser. `text-translate-text` analyzes source text
through the normal parser and Lingua runtime, obtains a canonical semantic
expression, and realizes it with the target language package. The semantic
input API is exposed by `TranslationRequest`; both paths use typed generation
errors, deterministic ordering, traces and generation budgets.
