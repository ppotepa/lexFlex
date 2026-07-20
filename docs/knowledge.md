# Knowledge

Knowledge snapshots contain verified semantic assertions and their evidence.
Insert, merge and clear operations use candidate-state mutation: validate,
recompute hashes, verify, persist, then replace runtime state. A failed store
operation leaves the in-memory state, index, serialized bytes and query result
unchanged.
