# Migrations

Session migrations are explicit and schema-bound. The store validates the
requested session ID and record identity, reads the declared source schema,
applies the caller-supplied migration, verifies the resulting payload and
atomically replaces the record. Unsupported schemas and migration failures do
not rewrite the old payload.
