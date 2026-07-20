# Negation Iteration Status

The semantic generation slice for negation is green in English and Polish.
It uses the existing canonical `SemanticExpression::Not` node and language
package realization data.

| Contract | Result |
| --- | ---: |
| EN semantic negation generation | PASS |
| PL semantic negation generation | PASS |
| EN `Not` inside `And` | PASS |
| PL `Not` inside `And` | PASS |
| Generation budget for nested `Not` | PASS |
| Parser text negation | PENDING |
| Text translation with negation | PENDING |

Current semantic outputs include:

```text
not Tom sees Iza
nie Tomek widzi Izę
not Tom sees Iza and Tom sees Iza
nie Tomek widzi Izę i Tomek widzi Izę
```

The text parser slice remains separate because it must create the correct
negation scope in the interlingua. A surface shortcut is intentionally not
used. The next parser iteration will add language data and compositional
categories for negated assertions, followed by a 200 EN / 200 PL corpus.
