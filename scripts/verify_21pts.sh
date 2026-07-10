#!/bin/bash
set -euo pipefail
SCRATCH=/tmp/grok-goal-a4a7591ecd62/implementer
mkdir -p "$SCRATCH"

echo "=== verify_21pts.sh $(date) ===" > "$SCRATCH/harness_scope_gate.txt"

# Sibling clean gate (run from parent view)
for repo in lightlm sshy; do
  echo "=== $repo ===" >> "$SCRATCH/harness_scope_gate.txt"
  git -C "/home/ppotepa/git/$repo" status --porcelain 2>&1 | tee -a "$SCRATCH/harness_scope_gate.txt" || true
done

# Check if clean (lightlm empty, sshy only .qwen tolerated or fail)
if git -C /home/ppotepa/git/lightlm status --porcelain | grep -q . ; then
  echo "FAIL: lightlm not clean" >> "$SCRATCH/harness_scope_gate.txt"
  exit 1
fi
if git -C /home/ppotepa/git/sshy status --porcelain | grep -v '.qwen' | grep -q . ; then
  echo "FAIL: sshy has core changes" >> "$SCRATCH/harness_scope_gate.txt"
  exit 1
fi
echo "GATE PASSED (lightlm clean, sshy non-core tolerated)" >> "$SCRATCH/harness_scope_gate.txt"

# Now lexFlex verification
cd "$(dirname "$0")/.."
echo "Working in $(pwd)"
echo "toplevel: $(git rev-parse --show-toplevel)"

# Step 1
cargo check 2>&1 | tail -3 > "$SCRATCH/verif_step1.txt"
cargo test --test integration_test 2>&1 | tail -5 >> "$SCRATCH/verif_step1.txt"

# Step 2 - CLIs (twice)
for i in 1 2; do
  cargo run --quiet --bin lexflex -- translate -f pl -t en 'Czy lepszy student ma kota?' 2>&1 | tail -1 > "$SCRATCH/cli_pl_en_$i.txt"
  cargo run --quiet --bin lexflex -- translate -f en -t pl 'Does a better student have a cat?' 2>&1 | tail -1 > "$SCRATCH/cli_en_pl_$i.txt"
  cargo run --quiet --bin lexflex -- translate -f pl -t en 'Tomek i Iza dał duży czerwony jabłko' 2>&1 | tail -1 > "$SCRATCH/cli_list_$i.txt"
  cargo run --quiet --bin lexflex -- translate -f pl -t en 'Tomek ma 30 jabłek' 2>&1 | tail -1 > "$SCRATCH/cli_30_$i.txt"
done

# Step 3 rg
( rg -c 'contains\(".*(duży|lepszy|jabł|kot|dobry|apple|student)|find\('\'' |name\.find|split_whitespace.*name|if verb_lemma == "ma"|verb_token.form == "ma"|"ma" ==|starts_with_vowel_sound|degree_surface_to_base|degree_base_stem|if .*contains\("i"|\.contains\("tomek|unknown.*replace|progressive.*eat|force.*APPLE' --glob '*.rs' src ; rg -c 'workaround|grouped concept|name-concat' --glob '*.rs' src ) > "$SCRATCH/verif_step3.txt" 2>/dev/null || echo "0 0" > "$SCRATCH/verif_step3.txt"

# Step 4 bench
./scripts/generate_benchmark.sh 2>/dev/null | head -50 > "$SCRATCH/bench_sample.txt"
cargo run --bin benchmark -- --input input.txt 2>&1 | head -20 > "$SCRATCH/bench_result.txt"

# Step 5 docs (simple reads)
echo "ERRORS: $(head -5 ERRORS.MD | grep -o 'RESOLVED' || echo '')" > "$SCRATCH/verif_step5.txt"
echo "PLAN: $(grep -o '34 pass' PLAN.md | head -1)" >> "$SCRATCH/verif_step5.txt"
echo "STATUS: $(grep '34/34 integration' STATUS.md)" >> "$SCRATCH/verif_step5.txt"
rg -c 'hack|workaround' --glob '*.rs' src >> "$SCRATCH/verif_step5.txt" 2>/dev/null || echo "0" >> "$SCRATCH/verif_step5.txt"

# Step 6
ls -l "$SCRATCH"/cli_*.txt "$SCRATCH"/final_verif.txt "$SCRATCH"/bench_*.txt 2>/dev/null || echo "captures" > "$SCRATCH/ls.txt"
cargo test --test integration_test 2>&1 | tail -3 > "$SCRATCH/final_test.txt"

# Append lexFlex git proof
echo "=== lexFlex git diff --name-only ===" >> "$SCRATCH/final_verif.txt"
git diff --name-only >> "$SCRATCH/final_verif.txt"
echo "=== lexFlex git diff --stat ===" >> "$SCRATCH/final_verif.txt"
git diff --stat >> "$SCRATCH/final_verif.txt"

echo "Verification complete. See $SCRATCH/final_verif.txt"
