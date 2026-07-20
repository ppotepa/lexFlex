# Document Context

`lexflex-documents` performs deterministic line segmentation and records source
hash, segment ID, byte span, and optional semantic analysis for every segment.
`DocumentStore` validates documents before insertion and removes one document
without changing unrelated documents. `replace` builds and verifies a complete
candidate store before committing, so failed replacement leaves documents,
hashes, and indexes unchanged. Document text is an evidence source; it does
not create domain-specific parser rules. Segment analyses can be queried by
canonical semantic hash through `find_expression`; there is no surface-text
search shortcut. 
`evidence_for_segment` creates the canonical `Evidence` value with document
source ID, source hash, and the exact segment span; the caller can then use the
normal knowledge transaction path. `create_assertion_for_segment` delegates
normalization and catalog validation to `SemanticAssertion::create`; it never
inserts into knowledge implicitly. `DocumentStore::save_to_file` and
`load_from_file` use verified JSON and an atomic temporary-file replacement for
durable document context.
