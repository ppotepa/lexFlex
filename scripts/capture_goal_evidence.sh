#!/bin/bash
set -euo pipefail
if [ $# -lt 1 ]; then
  echo "Usage: $0 <scratch_dir>"
  exit 1
fi
export GROK_GOAL_SCRATCH=$1
mkdir -p "$GROK_GOAL_SCRATCH"

# Thin runner per strategy: rtk run + full tee only. No grep/head/tail on parse/bulk for captures.

rtk run 'cargo test --workspace 2>&1' | tee "$GROK_GOAL_SCRATCH/cargo_test_full.txt"

rtk rg -i --glob '!target/**' --glob '!*/target/**' --glob '!*.lock' --glob '!goal/**' --glob '!results/**' --glob '!/tmp/**' 'AbcProcessor|abc::|concept_for_unknown' src/ tests/ lexflex-learner/src/ 2>&1 | grep -v 'resolve_concept_for_unknown' | tee "$GROK_GOAL_SCRATCH/rg_abc_clean.txt"

rtk ./target/debug/lexflex parse "Mieszkam z blargxyz." --lang pl 2>&1 | tee "$GROK_GOAL_SCRATCH/parse_blarg.txt"
rtk ./target/debug/lexflex parse "I saw blargxyz." --lang en 2>&1 | tee "$GROK_GOAL_SCRATCH/parse_blarg_en.txt"
rtk ./target/debug/lexflex parse "Widzę xyzqwe." --lang pl 2>&1 | tee "$GROK_GOAL_SCRATCH/parse_xyzqwe.txt"

rtk ./target/debug/lexflex translate "Mieszkam razem z żoną, córką i psem." --from pl --to en 2>&1 | tee "$GROK_GOAL_SCRATCH/adam_key.txt"

rtk run 'cargo test -p lexflex-learner test_bulk_adam_25_offline_ac4 -- --nocapture 2>&1' | tee "$GROK_GOAL_SCRATCH/bulk_output.txt"

echo "Thin atomic capture complete. Evidence emitted via test if GROK_GOAL_SCRATCH set."
ls -l "$GROK_GOAL_SCRATCH/"

