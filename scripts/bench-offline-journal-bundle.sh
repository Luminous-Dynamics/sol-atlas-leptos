#!/usr/bin/env bash
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later
# Commercial licensing: see COMMERCIAL_LICENSE.md at repository root
#
# Reproducible native offline-journal bundle benchmark matrix.
# Usage:
#   bash scripts/bench-offline-journal-bundle.sh quick [output-directory]
#   bash scripts/bench-offline-journal-bundle.sh full  [output-directory]
#
# The default mode is full. Each invocation emits independent JSON reports plus
# hardware/toolchain/dirty-state metadata. This script does not upload results.

set -euo pipefail

MODE="${1:-full}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
OUT_DIR="${2:-artifacts/offline-journal-bench/${STAMP}}"

case "$MODE" in
  quick|full) ;;
  *)
    echo "usage: $0 [quick|full] [output-directory]" >&2
    exit 2
    ;;
esac

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
mkdir -p "$OUT_DIR"

GIT_SHA="$(git rev-parse HEAD)"
HOST_LABEL="${HOST_LABEL:-$(hostname)-$(uname -sm)}"
export GIT_SHA HOST_LABEL

{
  printf 'captured_at_utc=%s\n' "$STAMP"
  printf 'git_sha=%s\n' "$GIT_SHA"
  printf 'git_dirty='
  if [[ -n "$(git status --porcelain)" ]]; then printf 'true\n'; else printf 'false\n'; fi
  printf 'host_label=%s\n' "$HOST_LABEL"
  printf '\n[rustc -Vv]\n'
  rustc -Vv
  printf '\n[cargo -V]\n'
  cargo -V
  printf '\n[uname -a]\n'
  uname -a
  printf '\n[lscpu]\n'
  if command -v lscpu >/dev/null 2>&1; then lscpu; else echo "lscpu unavailable"; fi
  printf '\n[free -h]\n'
  if command -v free >/dev/null 2>&1; then free -h; else echo "free unavailable"; fi
  printf '\n[Cargo.lock sha256]\n'
  if [[ -f Cargo.lock ]] && command -v sha256sum >/dev/null 2>&1; then
    sha256sum Cargo.lock
  else
    echo "Cargo.lock or sha256sum unavailable"
  fi
} > "$OUT_DIR/environment.txt"

run_case() {
  local records="$1"
  local blob_bytes="$2"
  local samples="$3"
  local name="records-${records}_blob-${blob_bytes}_samples-${samples}"
  local report="$OUT_DIR/${name}.json"

  echo "Running ${name}"
  cargo run --release \
    --manifest-path sol-atlas-core/Cargo.toml \
    --example offline_journal_bundle_bench -- \
    --records "$records" \
    --blob-bytes "$blob_bytes" \
    --samples "$samples" > "$report"

  jq -e '
    .parameters.samples as $samples |
    .schema_version == 1 and
    .correctness.roundtrip_equal == true and
    .correctness.record_count_preserved == true and
    .correctness.owner_scope_preserved == true and
    .correctness.integrity_digest_stable == true and
    .correctness.canonical_bytes_stable == true and
    .correctness.canonical_hash_only_matches_bundle_digest == true and
    (.raw_samples | length == ($samples * 5))
  ' "$report" >/dev/null
}

if [[ "$MODE" == "quick" ]]; then
  run_case 100 1024 11
  run_case 1000 1024 11
  run_case 10000 1024 11
else
  run_case 100 1024 31
  run_case 1000 1024 31
  run_case 10000 1024 31
  run_case 10000 32768 31
  run_case 1000 262144 31
  run_case 100 1048576 11
  run_case 100000 1024 11
fi

if command -v sha256sum >/dev/null 2>&1; then
  (
    cd "$OUT_DIR"
    sha256sum ./*.json > SHA256SUMS
    sha256sum environment.txt >> SHA256SUMS
  )
fi

echo "Benchmark evidence written to $OUT_DIR"
