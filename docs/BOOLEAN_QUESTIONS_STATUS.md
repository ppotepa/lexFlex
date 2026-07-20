# Boolean Questions Status

Boolean questions use a regular semantic goal shape:

```text
Equals(answer, proposition)
```

The answer is a Boolean query variable and the proposition remains the same
canonical interlingua expression as the corresponding assertion.

Verified production cases:

```text
Is Paris the capital of France?
-> Czy Paryż jest stolicą Francji?

Czy Paryż jest stolicą Francji?
-> Is Paris the capital of France?
```

Both directions preserve assertion/goal kind and semantic hash. English uses a
fronted copula realization, while Polish uses the `Czy` prefix. This slice does
not yet claim auxiliary `does not` morphology or negative yes/no questions.
