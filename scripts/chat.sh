#!/bin/bash
set -euo pipefail

# chat.sh — Interactive "AI-chat-like" interface for lexFlex
# Shows high verbosity + live streaming of internal steps (parse → deduction/learner/LLM → graph → generation)
# + typewriter effect for the final translation, just like a real chat.

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Force proper UTF-8 so Polish characters (ł, ą, ę, etc.) work in input and are passed cleanly to the binary.
export LANG=C.UTF-8
export LC_ALL=C.UTF-8
export LC_CTYPE=C.UTF-8

# Auto-configure common local LLM (LM Studio etc.) so you don't have to export every time.
# You can still override by setting the variables BEFORE running the script.
export LEXFLEX_LLM_BASE_URL="${LEXFLEX_LLM_BASE_URL:-http://192.168.195.88:1234/v1}"
export LEXFLEX_LLM_MODEL="${LEXFLEX_LLM_MODEL:-google/gemma-4-e2b}"
# Use "nothing" / short system prompt by default (faster responses, less tokens).
# Set LEXFLEX_LLM_NOTHINK=0 to use the full meta-prompt if you want.
export LEXFLEX_LLM_NOTHINK="${LEXFLEX_LLM_NOTHINK:-1}"
# Default: do NOT auto-write LLM proposals to lexicons (prevents pollution).
# Set LEXFLEX_NO_AUTO_WRITE=0 to allow writes (or remove for learning runs).
export LEXFLEX_NO_AUTO_WRITE="${LEXFLEX_NO_AUTO_WRITE:-1}"

# --- helpers for live "AI chat" feel ---------------------------------

have_color() { [ -t 1 ] && [ "${TERM:-}" != "dumb" ]; }

say() {
  # lexflex speaking, with optional typewriter
  local msg="$*"
  if have_color; then
    printf "\033[1;36mlexflex>\033[0m "
  else
    printf "lexflex> "
  fi
  typewriter "$msg"
}

typewriter() {
  # Print text one character at a time (like ChatGPT streaming)
  local text="$*"
  local delay="${TYPEWRITER_DELAY:-0.012}"
  local i
  for (( i=0; i<${#text}; i++ )); do
    printf "%s" "${text:$i:1}"
    sleep "$delay"
  done
  printf "\n"
}

thinking() {
  # Little live status line
  if have_color; then
    printf "\033[2m🤔 %s\033[0m\n" "$*"
  else
    printf "🤔 %s\n" "$*"
  fi
}

phase() {
  # Big visible step
  if have_color; then
    printf "\n\033[1;35m▶ %s\033[0m\n" "$*"
  else
    printf "\n▶ %s\n" "$*"
  fi
}

pretty_log() {
  # Color + emoji interesting trace lines so verbosity is readable and "alive"
  local line="$1"
  if [[ "$line" =~ (ERROR|error|failed|Failed) ]]; then
    if have_color; then
      printf "  \033[1;31m❌ %s\033[0m\n" "$line"
    else
      printf "  ❌ %s\n" "$line"
    fi
  elif [[ "$line" =~ (LLM|LocalLlm|local_llm|chat/completions|deduce|learner) ]]; then
    if have_color; then
      printf "  \033[1;95m🧠 %s\033[0m\n" "$line"
    else
      printf "  🧠 %s\n" "$line"
    fi
  elif [[ "$line" =~ (unknown|resolve_concept|UnknownConcept|on.the.fly|write_ron) ]]; then
    if have_color; then
      printf "  \033[1;33m🔍 %s\033[0m\n" "$line"
    else
      printf "  🔍 %s\n" "$line"
    fi
  elif [[ "$line" =~ (parse|Parse|to_interlingua|Interlingua|graph|frame|concept:) ]]; then
    if have_color; then
      printf "  \033[0;36m🧩 %s\033[0m\n" "$line"
    else
      printf "  🧩 %s\n" "$line"
    fi
  elif [[ "$line" =~ (generate|from_interlingua|realize|output) ]]; then
    if have_color; then
      printf "  \033[0;32m🗣️  %s\033[0m\n" "$line"
    else
      printf "  🗣️  %s\n" "$line"
    fi
  else
    # normal trace line — still show for high verbosity
    printf "  %s\n" "$line"
  fi
}

show_learned() {
  # Report any direct appends the LLM+learner path did this turn
  if ! command -v git >/dev/null 2>&1; then
    echo "   (git not available — check data/ manually)"
    return
  fi
  if git diff --quiet -- data/ 2>/dev/null; then
    if have_color; then
      printf "   \033[2m(no new RON proposals appended on this turn)\033[0m\n"
    else
      echo "   (no new RON proposals appended on this turn)"
    fi
  else
    echo "   📚 New data written live by the learner:"
    git diff --shortstat -- data/ 2>/dev/null | cat
    echo
    echo "   Changed files:"
    git diff --name-only -- data/ 2>/dev/null | sed 's/^/     - /'
    echo
    echo "   Sample of appended content (first hunks):"
    git diff --unified=0 -- data/concepts/concepts.ron data/lexicons/pl/lexicon.ron data/lexicons/en/lexicon.ron 2>/dev/null \
      | head -40 | cat
  fi
}

# --- startup ---------------------------------------------------------

clear || true
cat <<'EOF'
╔══════════════════════════════════════════════════════════════════╗
║   lexFlex  —  Live AI-Style Chat  (full internal verbosity)      ║
╚══════════════════════════════════════════════════════════════════╝
EOF

if [ ! -x ./target/debug/lexflex ]; then
  thinking "Binary not found — building (lexflex + learner)..."
  cargo build --quiet
  echo "✓ Build complete."
fi

echo "Choose source language (the language you will type in):"
echo "  1) pl  (Polish)"
echo "  2) en  (English)"
read -p "  [1-2] > " choice

case "$choice" in
  2) SRC="en"; TGT="pl";;
  *) SRC="pl"; TGT="en";;
esac

echo
if have_color; then
  printf "\033[1mSource:\033[0m %s   →   \033[1mTarget:\033[0m %s\n" "$SRC" "$TGT"
else
  echo "Source: $SRC  →  Target: $TGT"
fi
echo

# LLM is now auto-configured with defaults at the top of the script.
# Override by exporting before running, e.g. LEXFLEX_LLM_BASE_URL= ./scripts/chat.sh
if [ -z "${LEXFLEX_LLM_BASE_URL}" ]; then
  echo "⚠️  LEXFLEX_LLM_BASE_URL is empty — on-the-fly learning disabled for this run."
else
  echo "✓ LLM auto-configured (from script defaults or your env):"
  echo "    BASE_URL: $LEXFLEX_LLM_BASE_URL"
  echo "    MODEL:    ${LEXFLEX_LLM_MODEL:-<default in learner>}"
  if [ "${LEXFLEX_LLM_NOTHINK:-1}" != "0" ]; then
    echo "    (using short /nothink system prompt for faster responses)"
  fi
  echo

  # Proactive warmup so first real sentence is fast.
  echo "🔥 Starting LLM warmup in background..."
  (
    LEXFLEX_DATA_DIR=/tmp/lexflex-warmup.$$ \
    LEXFLEX_LLM_NOTHINK=1 \
    LEXFLEX_NO_AUTO_WRITE=1 \
    RUST_LOG=error \
      ./target/debug/lexflex translate "xqzzyyy-warmup" --from "$SRC" --to "$TGT" >/dev/null 2>&1 || true
  ) &
  disown 2>/dev/null || true
  sleep 0.15
fi

echo "Type sentences. Special commands:"
echo "  quit / exit          — leave"
echo "  full                 — raw huge debug dump for next sentence"
echo "  json                 — machine-readable semantic digest for next sentence"
echo "  help                 — this text again"
echo
echo "Everything you see below is real: parsers, deduction, LLM calls, graph, on-the-fly RON writes..."
echo "Note: on-the-fly writes are now conservative (skip low-confidence or low-quality proposals) to stop trashing the RON files."
echo

# High verbosity by default (user asked for "więcej werbosity")
export RUST_LOG="${RUST_LOG:-lexflex=trace,lexflex_learner=trace,tracing=info}"

# --- main chat loop --------------------------------------------------

while true; do
  if have_color; then
    printf "\033[1;32m[%s]>\033[0m " "$SRC"
  else
    printf "[%s]> " "$SRC"
  fi
  read -r INPUT || break

  # Sanitize input: remove control characters (ESC, arrows etc.), keep everything printable.
  # Locale-safe sed with explicit C.UTF-8 (no destructive tr -cd '[:print:]' which destroys diacritics in C).
  # Polish letters (ł ą ę ź ż ó ć ń) survive to binary; no "invalid UTF-8" or "ksik".
  INPUT=$(printf '%s' "$INPUT" | LC_ALL=C.UTF-8 sed 's/[\x00-\x1F\x7F]//g' | LC_ALL=C.UTF-8 sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
  [[ -z "$INPUT" ]] && continue

  case "$INPUT" in
    quit|exit|q)
      say "Bye! (all on-the-fly RON changes are already persisted in data/)"
      break
      ;;
    full)
      echo "Enter sentence for FULL raw Interlingua dump (can be 100-200 KB):"
      read -r FULL_INPUT || continue
      phase "FULL PARSE DEBUG"
      ./target/debug/lexflex parse "$FULL_INPUT" --lang "$SRC" 2>&1 | cat
      continue
      ;;
    json)
      echo "Enter sentence for JSON semantic digest:"
      read -r JSON_INPUT || continue
      phase "JSON SEMANTIC DIGEST"
      ./target/debug/lexflex explain "$JSON_INPUT" --lang "$SRC" --format json 2>/dev/null
      continue
      ;;
    help|h)
      echo "Commands: quit, full, json, help"
      continue
      ;;
  esac

  # --- 1. Show the input like a chat message
  if have_color; then
    printf "\n\033[1;32m[you]\033[0m %s\n" "$INPUT"
  else
    printf "\n[you] %s\n" "$INPUT"
  fi

  # --- 2. Semantic digest (IL-derived actors, actions, roles, discourse)
  phase "1. Parsing + concept resolution (semantic digest)"
  thinking "Running parser + deduction + discourse → IL-derived digest..."

  DIGEST_OUT=$(mktemp)
  if ./target/debug/lexflex explain "$INPUT" --lang "$SRC" --format human 2>/dev/null >"$DIGEST_OUT"; then
    echo "   Semantic digest:"
    sed 's/^/     /' "$DIGEST_OUT"
    echo "   (Raw Interlingua/graph: use 'full'. JSON digest: use 'json'.)"
  else
    echo "   (Digest unavailable — see translation trace below.)"
  fi
  rm -f "$DIGEST_OUT"

  # --- 3. The main event: TRANSLATE with full streaming trace
  phase "2. Translation pipeline (full RUST_LOG trace — live)"
  thinking "Tokenizing → lexicon lookup → unknown resolver (LLM first if configured) → deduction → frame building → target generation"

  # Pre-steps (narrative)
  sleep 0.05
  echo "   [1/5] Surface forms tokenized and lexicon consulted..."
  sleep 0.04
  echo "   [2/5] Unknown / novel items → deduction service (LLM if configured)..."

  TMP_OUT=$(mktemp)
  TMP_TRACE=$(mktemp)

  # Run the actual translate (which may call LLM for new words like "czarny")
  # in background so we can show a live visible spinner.
  echo "   lexflex> thinking (LLM may be slow on first real inference for new words)..."
  RUST_LOG="$RUST_LOG" \
    stdbuf -oL -eL ./target/debug/lexflex translate "$INPUT" --from "$SRC" --to "$TGT" \
      >"$TMP_OUT" 2>"$TMP_TRACE" &
  CMD_PID=$!

  # Live foreground spinner (very visible "AI chat" style busy indication).
  # This runs in main shell so it *always* shows updating progress while the binary is blocked on LLM.
  # No more "frozen" feeling.
  spinner='|/-\'
  i=0
  while kill -0 "$CMD_PID" 2>/dev/null; do
    if [ -t 1 ]; then
      printf "\r   lexflex> thinking %s " "${spinner:$i:1}"
    else
      # non-tty fallback: occasional dots
      if (( i % 5 == 0 )); then printf "."; fi
    fi
    i=$(( (i+1) % 4 ))
    sleep 0.2
  done
  wait "$CMD_PID" || true
  if [ -t 1 ]; then
    printf "\r   lexflex> thinking... done             \n"
  else
    echo " done"
  fi

  echo "   [3/5] Interlingua + generation complete."
  echo "   [4/5] Live trace replay (new lines appear gradually):"
  echo "   --- live internal trace (RUST_LOG=$RUST_LOG) ---"

  # Replay trace with small pacing + pretty colors so it feels "streaming" like tokens arriving
  if [ -s "$TMP_TRACE" ]; then
    while IFS= read -r line || [[ -n "$line" ]]; do
      pretty_log "$line"
      sleep 0.004
    done < "$TMP_TRACE"
  else
    echo "   (Rust tracing was quiet this run — enable LLM or add more instrument!() spans for richer logs)"
  fi

  # --- Final answer with typewriter (the "new characters appear" effect)
  echo
  RESULT="$(cat "$TMP_OUT" | tr -d '\r' | tail -n1 | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
  if [ -n "$RESULT" ]; then
    if have_color; then
      printf "\033[1;32m[%s → %s]\033[0m " "$SRC" "$TGT"
    else
      printf "[%s → %s] " "$SRC" "$TGT"
    fi
    typewriter "$RESULT"
  else
    say "Translation produced no output (see trace above for errors)."
  fi

  # --- 5. Learner / on-the-fly RON activity
  phase "5. On-the-fly knowledge (LLM proposals appended directly)"
  show_learned

  echo
  if have_color; then
    printf "\033[2m────────────────────────────────────────────────────────────────────\033[0m\n"
  else
    echo "────────────────────────────────────────────────────────────────────"
  fi

  rm -f "$TMP_OUT" "$TMP_TRACE"
done

echo
echo "Session finished. Any proposals written by the learner are already in data/."
echo "You can review them with:  git diff data/"
echo "Or run the big benchmark collector:  ./scripts/llm_full_benchmark.sh"
