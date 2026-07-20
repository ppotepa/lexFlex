# Boolean Scope Status

The parser now supports compositional `Not`, `And`, and `Or` individually.
Combined unparenthesized clauses currently expose scope ambiguity explicitly.

```text
Not A or B
Not A and B
```

Both operator scopes are returned as `TextAmbiguous` with two semantic
alternatives. The engine does not silently choose a reading or apply a
surface-level rewrite.

The next implementation slice is an explicit precedence/scope contract for
Boolean operators, followed by tests that require the selected semantic hash.
Until that contract exists, combined negation and coordination are not counted
as deterministic translation cases.
