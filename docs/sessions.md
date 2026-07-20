# Sessions

Session identifiers are validated before filesystem access: they are non-empty,
path-safe ASCII identifiers with a maximum length of 128 bytes. Session writes
use a temporary file and atomic rename. Loads validate record schema, requested
identity, payload integrity, and knowledge state before indexes are rebuilt.
Legacy schema migration is explicit and failed migrations leave the original
record unchanged.
