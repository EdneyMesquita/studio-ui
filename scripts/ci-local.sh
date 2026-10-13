#!/usr/bin/env bash
# Everything the GitHub workflow checks, run on this machine. While the repository is private the hosted
# runners are not used (Actions minutes), so this script and the pre-push hook are the gate.
#   bash scripts/ci-local.sh          full run
#   bash scripts/ci-local.sh --quick  skip the example build and the history scan
set -euo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || dirname "$0"/..)"

quick=0
[ "${1:-}" = "--quick" ] && quick=1

step() { printf '\n== %s\n' "$*"; }

step "public-readiness scan"
bash scripts/check-public.sh

step "format"
cargo fmt --all --check

step "clippy (default features)"
cargo clippy --all-targets -- -D warnings

step "clippy (dev-hooks)"
cargo clippy --all-targets --features dev-hooks -- -D warnings

step "tests"
cargo test --quiet

if [ "$quick" -eq 0 ]; then
  step "examples"
  cargo build --examples --quiet
fi

if command -v gitleaks >/dev/null 2>&1; then
  if [ "$quick" -eq 0 ] && git rev-parse --git-dir >/dev/null 2>&1; then
    step "gitleaks (whole history)"
    gitleaks git . --no-banner --redact
  fi
  step "gitleaks (working tree)"
  gitleaks dir . --no-banner --redact
else
  printf '\nwarning: gitleaks not installed; secret scan skipped (brew install gitleaks)\n' >&2
fi

printf '\nOK: local CI passed\n'
