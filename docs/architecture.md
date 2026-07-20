# LexFlex Architecture

LexFlex keeps one semantic path: surface text is parsed into `LinguaExpression`,
compiled and verified, then evaluated as `SemanticExpression`. Knowledge writes
are candidate-state transactions; persisted state is verified before indexing.

The generation crate consumes semantic expressions and language-package lexical
anchors. It does not parse text or contain sentence-specific rules. Translation
uses the same semantic generation boundary with a target language model.

## Integrity boundaries

Model declarations are compiled into an immutable, Arc-shared verified context.
Each request compiles a verified entry against that context. Session IDs are
validated value objects and storage migrations are explicit operations.
