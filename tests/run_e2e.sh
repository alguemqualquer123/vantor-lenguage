#!/usr/bin/env bash
# Lexicon E2E harness (macOS/Linux twin of tests/run_e2e.ps1).
#
# Runs every tests/e2e_*.lex through `lex run --ci` (non-interactive) and
# counts PASS/FAIL by exit code. Exit 0 when all pass, 1 otherwise.
#
# Usage: ./tests/run_e2e.sh [filter]   (default filter: e2e_*.lex)
set -u
export CI=true

FILTER="${1:-e2e_*.lex}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TESTS_DIR="$ROOT/tests"

# Resolve lex: ../target/debug/lex (CWD=tests) or target/debug/lex (CWD=root).
if [ -x "$TESTS_DIR/../target/debug/lex" ]; then
  LEX="$TESTS_DIR/../target/debug/lex"
elif [ -x "$ROOT/target/debug/lex" ]; then
  LEX="$ROOT/target/debug/lex"
elif [ -x "$ROOT/target/release/lex" ]; then
  LEX="$ROOT/target/release/lex"
else
  echo "ERRO: lex nao encontrado (build primeiro: cargo build -p lexicon-cli)" >&2
  exit 1
fi
echo "lex: $LEX"

pass=0
fail=0
failed=""
total_start=$(date +%s)
for f in "$TESTS_DIR"/$FILTER; do
  [ -f "$f" ] || continue
  name="$(basename "$f")"
  start=$(date +%s)
  if "$LEX" run --ci "$f" >/dev/null 2>&1; then
    code=0
  else
    code=$?
  fi
  ms=$(( ($(date +%s) - start) * 1000 ))
  if [ "$code" -eq 0 ]; then
    echo "PASS $name (${ms}ms)"
    pass=$((pass + 1))
  else
    echo "FAIL $name (exit $code)"
    fail=$((fail + 1))
    failed="$failed $name"
  fi
done
total=$(( $(date +%s) - total_start ))
echo ""
echo "Resumo: PASS $pass  FAIL $fail  tempo total ${total}s"
if [ -n "$failed" ]; then
  echo "Falhas:$failed"
fi
[ "$fail" -eq 0 ]
