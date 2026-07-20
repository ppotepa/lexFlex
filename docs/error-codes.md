# Error Codes

The CLI uses stable numeric classes: `0` success, `2` usage, `4` invalid
input/not parsed, `5` ambiguity or runtime budget, `6` model/integrity,
`7` store I/O, `8` internal invariant/serialization and `9` local file/RON
errors. JSON responses are written to stdout; human-readable diagnostics are
written to stderr.
