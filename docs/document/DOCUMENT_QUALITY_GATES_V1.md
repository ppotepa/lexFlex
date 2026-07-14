# Document Quality Gates V1

## Chapter Gates

### Chapter 01 gate

- profile loads
- documentation and RON agree
- validation tests pass
- translator behavior is unchanged

### Chapter 02 gate

- corpus is valid
- every case produces a report
- baseline is frozen
- errors are classified
- no failed case is silently skipped

## Final Document MVP Gates

### Technical gates

- 100% cases end in a controlled result
- 0 panic
- 100% determinism
- 100% non-empty outputs for non-Fatal cases
- 100% report artifacts generated successfully

### Semantic gates

- 100% critical proper names preserved
- 100% marked numbers preserved
- 100% marked negations preserved
- 0 Agent/Patient/Recipient swaps in blocking cases
- at least 95% simple reference chains correct
- at least 90% zero-anaphora correct
- at least 95% required frame signatures preserved
- at least 95% glossary constraints satisfied

### Structural gates

- at least 99% paragraphs preserved
- 100% cases preserve block order
- no full sentence or paragraph loss

### Manual gates

- adequacy average >= 4.0/5
- context coherence average >= 4.0/5
- terminology average >= 4.5/5
- fluency average >= 3.5/5
- 0 Fatal errors on holdout gate

Chapter 01 stores these gates as a contract. Chapter 02 freezes a baseline and does not need to meet them yet.
