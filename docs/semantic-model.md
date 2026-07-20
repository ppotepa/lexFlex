# Semantic Model

`SemanticExpression` is the single canonical meaning representation. Entities,
concepts, values, variables, applications, relations, boolean composition and
quantifiers are normalized before canonical hashing. Assertions and goals add
verified projections, evidence and query policy around that expression.

Natural-language parsing produces `LinguaExpression`; verified Lingua
compilation is the boundary before execution. Generation and translation
consume the same semantic expression instead of introducing another meaning
engine.
