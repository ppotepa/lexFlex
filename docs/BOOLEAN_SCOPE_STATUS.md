# Boolean Scope Status

The parser now supports compositional `Not`, `And`, and `Or`, including
unparenthesized combinations with deterministic Boolean precedence.

```text
Not A or B
Not A and B
```

The precedence contract is:

```text
Not > And > Or
```

The engine selects the corresponding semantic tree before lowering and does
not use a surface-level rewrite.

The selected semantic hash is covered by engine-level tests. Parenthesized
Boolean syntax remains a separate future grammar extension.
