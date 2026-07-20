# Benchmarks

`lexflex-benchmark` emits JSON measurements for compile, execution, solving,
analysis, query, and parser metrics. Use `--iterations N` to control sampling
and `--output PATH` for a reproducible artifact. CI can enforce a p95 budget
with `--max-p95-ns N`; any case above the threshold exits with a typed failure.

The release gate runs a bounded smoke benchmark. Production thresholds should
be selected from a versioned baseline for the target hardware before enabling
stricter limits.
