# Lingua

Lingua is the typed executable layer between parsing and semantic execution.
Model declarations are compiled once into an immutable, catalog-bound verified
model. Each request is compiled into a verified entry referencing that model.
The interpreter accepts only verified executable values and applies explicit
execution policies and resource limits.
