# Evidence

Evidence identifies a source, optional span and canonical source hash. Evidence
and evidence sets verify IDs, spans, keys and conflicts at construction and
deserialization. Incoming, stored and generated-text origins have separate
error mappings: invalid input, persisted integrity corruption and impossible
generated-state failures are not collapsed into one code.
