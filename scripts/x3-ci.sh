#!/usr/bin/env bash
# Clasy X3 portable validation: no Host8 storage required.
set -euo pipefail
cd "$(dirname "$0")/.."
toolchain="+1.81.0"
cargo "$toolchain" fmt --all -- --check
cargo "$toolchain" clippy --offline --release --all-targets -- -D warnings
cargo "$toolchain" test --offline --release --all-targets
cargo "$toolchain" build --offline --release --bins
if [[ "$(printenv RUN_FULL_VERIFY || echo 0)" == "1" ]]; then
  cargo "$toolchain" run --offline --release --bin verify
fi
printf 'X3_LOCAL_GATE|format=PASS|clippy=PASS|tests=PASS|bins=PASS|host8=NOT_RUN|status=PASS\n'
