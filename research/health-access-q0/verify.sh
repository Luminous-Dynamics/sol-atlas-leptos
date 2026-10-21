#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$script_dir"

revision="${HEALTH_ACCESS_CODE_REVISION:-$(git rev-parse HEAD 2>/dev/null || printf 'unknown')}"
export HEALTH_ACCESS_CODE_REVISION="$revision"

cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run --quiet --bin health-access-q0 -- fixtures/reference-cases.v0.json
