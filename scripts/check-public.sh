#!/usr/bin/env bash
# check-public.sh: fail when a file in this repository holds something that must not become public.
#
# Scans the files Git knows about (tracked, plus untracked files that Git does not ignore). Outside a Git
# work tree it scans every file except .git/, target/ and tools/icons/.venv/.
#
# Rules (the reasons are in docs/PUBLISHING.md):
#   abs-path     absolute home-directory paths (macOS, Linux, Windows)
#   private-key  PEM private key headers
#   token        well-known API token shapes (the matched text is not printed)
#   env-file     a tracked .env file; templates ending in .example, .sample or .template are fine
#   email        email addresses outside the allow-list below
#   host         internal host names (*.internal, *.lan, *.local; localhost is fine)
#   rfc1918      private IPv4 addresses, except in files listed in scripts/check-public.allow
#   big-binary   binary files larger than 2 MB
#
# Files that contain NUL bytes are binary: they skip the text rules (fonts, for example) and only get the
# size check. This script does not scan itself.
#
# Email allow-list: noreply@anthropic.com, <id>+<user>@users.noreply.github.com, and the documentation
# placeholder security@example.(com|org|invalid).
#
# Portable to bash 3.2 (macOS) and to GNU or BSD userland. Usage: scripts/check-public.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

SELF="scripts/check-public.sh"
ALLOW_FILE="scripts/check-public.allow"
MAX_BINARY_BYTES=2097152

EMAIL_RE='[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,}'
EMAIL_ALLOWED='^[0-9]+:(noreply@anthropic\.com|[0-9]+\+[A-Za-z0-9-]+@users\.noreply\.github\.com|[A-Za-z0-9-]+@users\.noreply\.github\.com|security@example\.(com|org|invalid))$'
ABS_PATH_RE='/Users/[A-Za-z0-9._-]+/|/home/[A-Za-z0-9._-]+/|C:\\Users\\'
PRIVATE_KEY_RE='BEGIN [A-Z ]*PRIVATE KEY'
TOKEN_RE='(^|[^A-Za-z0-9])(ghp_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|sk-ant-[A-Za-z0-9_-]{10,}|sk-[A-Za-z0-9_-]{20,}|AKIA[0-9A-Z]{16}|xox[bp]-[A-Za-z0-9-]{10,})'
HOST_RE='[A-Za-z0-9-]+\.(internal|lan|local)([^A-Za-z0-9_(]|$)'
RFC1918_RE='(^|[^0-9.])(10\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}|192\.168\.[0-9]{1,3}\.[0-9]{1,3}|172\.(1[6-9]|2[0-9]|3[01])\.[0-9]{1,3}\.[0-9]{1,3})([^0-9.]|$)'

FILES="$(mktemp "${TMPDIR:-/tmp}/check-public.files.XXXXXX")"
trap 'rm -f "$FILES"' EXIT

if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  git ls-files --cached --others --exclude-standard >"$FILES"
else
  find . \( -path ./.git -o -path ./target -o -path ./tools/icons/.venv \) -prune \
    -o -type f -print | sed 's|^\./||' >"$FILES"
fi

# The patterns of scripts/check-public.allow that are not comments or blank lines.
ALLOW_LIST=""
if [ -f "$ALLOW_FILE" ]; then
  ALLOW_LIST="$(grep -vE '^[[:space:]]*(#|$)' "$ALLOW_FILE" || true)"
fi

problems=0
scanned=0

report() { # report RULE PATH [DETAIL]
  problems=$((problems + 1))
  if [ -n "${3:-}" ]; then
    printf 'FAIL %-11s %s  %s\n' "$1" "$2" "$3"
  else
    printf 'FAIL %-11s %s\n' "$1" "$2"
  fi
}

# matches FILE REGEX: "LINE:TEXT" for each matching line, nothing when none.
matches() {
  grep -nE -- "$2" "$1" 2>/dev/null || true
}

# first 120 characters of a matched line, with tabs flattened
excerpt() {
  printf '%s' "$1" | tr '\t' ' ' | cut -c1-120
}

while IFS= read -r f; do
  [ -n "$f" ] || continue
  [ -f "$f" ] || continue
  [ "$f" = "$SELF" ] && continue
  scanned=$((scanned + 1))

  # a tracked .env file, whatever its content
  base="${f##*/}"
  case "$base" in
    .env | .env.*)
      case "$base" in
        *.example | *.sample | *.template) ;;
        *) report env-file "$f" ;;
      esac
      ;;
  esac

  size="$(wc -c <"$f" | tr -d ' ')"
  nonnul="$(LC_ALL=C tr -d '\000' <"$f" | wc -c | tr -d ' ')"
  if [ "$size" != "$nonnul" ]; then
    if [ "$size" -gt "$MAX_BINARY_BYTES" ]; then
      report big-binary "$f" "$size bytes (limit $MAX_BINARY_BYTES)"
    fi
    continue
  fi

  # absolute home-directory paths
  while IFS= read -r line; do
    [ -n "$line" ] && report abs-path "$f" "${line%%:*}: $(excerpt "${line#*:}")"
  done < <(matches "$f" "$ABS_PATH_RE")

  # PEM private keys: the header only, never the line
  if [ -n "$(matches "$f" "$PRIVATE_KEY_RE")" ]; then
    report private-key "$f"
  fi

  # token shapes: the matched text is never printed
  if [ -n "$(matches "$f" "$TOKEN_RE")" ]; then
    report token "$f"
  fi

  # email addresses outside the allow-list
  while IFS= read -r line; do
    [ -n "$line" ] && report email "$f" "${line%%:*}: ${line#*:}"
  done < <(grep -noE -- "$EMAIL_RE" "$f" 2>/dev/null | grep -vE -- "$EMAIL_ALLOWED" 2>/dev/null || true)

  # internal host names
  while IFS= read -r line; do
    [ -n "$line" ] && report host "$f" "${line%%:*}: $(excerpt "${line#*:}")"
  done < <(matches "$f" "$HOST_RE")

  # private IPv4 addresses, unless the file is allow-listed
  allowed=0
  while IFS= read -r pattern; do
    [ -n "$pattern" ] || continue
    # shellcheck disable=SC2254
    case "$f" in
      $pattern) allowed=1 ;;
    esac
  done <<<"$ALLOW_LIST"
  if [ "$allowed" -eq 0 ]; then
    while IFS= read -r line; do
      [ -n "$line" ] && report rfc1918 "$f" "${line%%:*}: $(excerpt "${line#*:}")"
    done < <(matches "$f" "$RFC1918_RE")
  fi
done <"$FILES"

if [ "$problems" -gt 0 ]; then
  echo
  echo "check-public: $problems problem(s) in $scanned file(s). Fix them; for a false positive, tighten the pattern in $SELF." >&2
  exit 1
fi

echo "OK: check-public found nothing in $scanned file(s)"
