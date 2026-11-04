#!/usr/bin/env bash
# Native offline evaluation; Rust emits hot HBP receipts before sealing/indexing.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p logs/x3 receipts/x3
store_path="${1:-/mnt/d/ASOLARIA-HOST8-ROOMS-300G}"
cargo +1.81.0 fmt --check >logs/x3/fmt.log 2>&1
cargo +1.81.0 clippy --offline --release --all-targets -- -D warnings >logs/x3/clippy.log 2>&1
cargo +1.81.0 test --offline --release --all-targets >logs/x3/tests.log 2>&1
cargo +1.81.0 build --offline --release --bins >logs/x3/build.log 2>&1
target/release/sector_contract >receipts/x3/LATTICE.hbp
target/release/verify >receipts/x3/VERIFY81.hbp
target/release/sweep_rooms --rooms 4 --threads 1 --sector /dev/null --no-step >receipts/x3/OLD-SWEEP-NEGATIVE.hbp
target/release/audit_rooms --store "$store_path" --group 81 --windows 9 --steps 3 --threads 9 >receipts/x3/ROOMS81.hbp
target/release/audit_rooms --store "$store_path" --group 243 --windows 9 --steps 3 --threads 9 >receipts/x3/ROOMS243.hbp
# Negative controls must fail, never generate substitute rooms.
if target/release/audit_rooms --store "$store_path/definitely-missing-x3" >logs/x3/missing-store.log 2>&1; then
    echo 'FAIL: missing store accepted' >&2; exit 1
fi
if target/release/audit_rooms --store "$store_path" --threads 0 >logs/x3/zero-threads.log 2>&1; then
    echo 'FAIL: zero threads accepted' >&2; exit 1
fi
if target/release/audit_rooms --store "$store_path" --group 242 >logs/x3/bad-width.log 2>&1; then
    echo 'FAIL: unsupported width accepted' >&2; exit 1
fi
printf 'X3NEGATIVE|missing_store_rejected=1|zero_threads_rejected=1|unsupported_width_rejected=1|status=PASS|json=0\n' >receipts/x3/NEGATIVE.hbp
(cd receipts/x3; for packet in *.hbp; do sha256sum "$packet" >"$packet.sha256"; done)
printf 'X3INDEX|schema=CLASY-X3-INDEX-V1|json=0\n' >receipts/x3/INDEX.hbi
for packet in receipts/x3/*.hbp; do printf 'PACKET|path=%s|sha256=%s|json=0\n' "$packet" "$(sha256sum "$packet" | cut -d' ' -f1)" >>receipts/x3/INDEX.hbi; done
sha256sum receipts/x3/INDEX.hbi >receipts/x3/INDEX.hbi.sha256
printf 'X3GATE|rust=1.81.0|fmt=PASS|clippy=PASS|tests=PASS|lattice=PASS|stored_rooms81=PASS|stored_rooms243=PASS|negative_controls=PASS|json=0\n'
