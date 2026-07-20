# Coordination Or Status

The language packages now provide compositional `Or` constructions:

```text
English: or
Polish:  albo
```

Both constructions use the same higher-order category shape as `And` and
produce `SemanticExpression::Or`. No sentence-specific parser branch or
surface-level semantic shortcut is used.

Verified cases:

```text
Tom sees Iza or Iza sees Tom.
-> Iza widzi Tomka albo Tomek widzi Izę

Tomek widzi Izę albo Iza widzi Tomka.
-> Iza sees Tom or Tom sees Iza
```

The normalizer preserves the canonical ordering of disjuncts. Generation is
also covered for a semantic `Or` expression in English and Polish.

This is a focused feature slice, not the complete A2/B1 corpus. Broader
coverage still requires additional lexical, morphological, question, and
negation cases.
