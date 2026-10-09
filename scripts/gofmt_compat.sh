#!/usr/bin/env bash
# gofmt byte-level compatibility check.
#
# Runs `woofmt fmt --stdout` over testdata/gofmt_corpus and compares the
# output byte-for-byte with `gofmt -s`. Requires the Go toolchain; exits 0
# with a notice when gofmt is unavailable (local dev machines), and the CI
# job that installs Go is the enforcement point.
#
# Usage: scripts/gofmt_compat.sh [path-to-woofmt-binary]
set -uo pipefail

cd "$(dirname "$0")/.." || exit 1

WOOFMT="${1:-./target/release/woofmt}"
if [ ! -x "$WOOFMT" ]; then
  WOOFMT="./target/debug/woofmt"
fi
if [ ! -x "$WOOFMT" ]; then
  echo "ERROR: woofmt binary not found. Run 'cargo build' first." >&2
  exit 2
fi

if ! command -v gofmt >/dev/null 2>&1; then
  echo "SKIP: gofmt not installed; byte-level comparison not run (CI enforces this)."
  exit 0
fi

CORPUS_DIR="testdata/gofmt_corpus"
failures=0
checked=0

for f in "$CORPUS_DIR"/*.go; do
  [ -e "$f" ] || continue
  checked=$((checked + 1))

  if ! "$WOOFMT" fmt --stdout "$f" > /tmp/woofmt_out.$$ 2>/tmp/woofmt_err.$$; then
    echo "FAIL(format): $f — woofmt exited non-zero:"
    sed 's/^/  /' /tmp/woofmt_err.$$
    failures=$((failures + 1))
    continue
  fi

  if ! gofmt "$f" > /tmp/gofmt_out.$$ 2>/dev/null; then
    echo "FAIL(corpus): $f — gofmt itself rejected this file. Fix the corpus." >&2
    failures=$((failures + 1))
    continue
  fi

  if ! diff -u /tmp/gofmt_out.$$ /tmp/woofmt_out.$$ > /tmp/gofmt_diff.$$; then
    echo "DIFF: $f (woofmt output != gofmt output)"
    sed 's/^/  /' /tmp/gofmt_diff.$$ | head -20
    failures=$((failures + 1))
  fi
done

rm -f /tmp/woofmt_out.$$ /tmp/gofmt_out.$$ /tmp/gofmt_diff.$$ /tmp/woofmt_err.$$

echo "Checked $checked file(s) against gofmt."
if [ "$failures" -gt 0 ]; then
  echo "FAILED: $failures file(s) diverge from gofmt."
  exit 1
fi
echo "PASS: woofmt output is byte-identical to gofmt on the corpus."
