#!/usr/bin/env bash
# Prediction-first 3-step / 3-leaf exact Taylor–Green to Clay observable bridge.
# The Taylor–Green checkout is pinned to ee71e0ee.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MODEL="$ROOT/bridge/taylor-green/convert/Cargo.toml"
INPUT="$ROOT/bridge/taylor-green/tg-source/results/2026-10-30-noise-test"
OUTPUT="$ROOT/bridge/taylor-green/out"
if [ "$#" -ge 1 ]; then INPUT="$1"; fi
if [ "$#" -ge 2 ]; then OUTPUT="$2"; fi
mkdir -p "$OUTPUT"
INPUT="$(cd "$INPUT" && pwd)"
OUTPUT="$(cd "$OUTPUT" && pwd)"
(cd "$INPUT" && sha256sum -c INIT.sha256)
grep -q '^BRIDGE_3LEAVES_PREDICTION|version=1|' "$ROOT/bridge/taylor-green/PREDICTION-3-LEAVES.hbp"
export CARGO_TARGET_DIR="$OUTPUT/target"
cargo +1.81.0 fmt --all --manifest-path "$MODEL"
cargo +1.81.0 clippy --offline --release --all-targets --manifest-path "$MODEL" -- -D warnings
cargo +1.81.0 test --offline --release --all-targets --manifest-path "$MODEL"
cargo +1.81.0 build --offline --release --locked --manifest-path "$MODEL"
BIN="$CARGO_TARGET_DIR/release/taylor-clay-bridge"
"$BIN" "$INPUT/INIT_TGV.trits" "$INPUT/INIT_NULL.trits" "$INPUT/GATES.lut" > "$OUTPUT/RUN.hbp"
grep -q '^BRIDGE_RESULT|steps=3|leaves=3|arms=3|.*status=PASS|' "$OUTPUT/RUN.hbp"
prev=GENESIS
printf 'BRIDGE_HBI|source=TaylorGreen-noise-test|steps=3|leaf_count=3|json=0\n' >"$OUTPUT/LEAVES.hbi"
for t in 1 2 3; do
  f="$OUTPUT/LEAF-$t.hbp"
  printf 'LEAF_HEADER|t=%s|previous_leaf_sha256=%s|arms=TGV,SCR,NULL|json=0\n' "$t" "$prev" > "$f"
  grep "^LEAF|t=$t|" "$OUTPUT/RUN.hbp" >> "$f"
  n="$(grep -c "^LEAF|t=$t|" "$f")"
  [ "$n" -eq 3 ] || { echo "expected 3 arms at step $t; found $n" >&2; exit 1; }
  grep -q "^STEP_RESULT|t=$t|leaves=3|status=PASS|" "$OUTPUT/RUN.hbp"
  printf 'LEAF_FOOTER|t=%s|rows=3|status=PASS|json=0\n' "$t" >> "$f"
  prev_hash="$prev"
  prev="$(sha256sum "$f" | cut -d' ' -f1)"
  printf 'LEAF_INDEX|t=%s|file=LEAF-%s.hbp|sha256=%s|previous_leaf_sha256=%s|rows=3|json=0\n' "$t" "$t" "$prev" "$prev_hash" >> "$OUTPUT/LEAVES.hbi"
done
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
cp "$INPUT/INIT_TGV.trits" "$TMP/BAD.trits"
printf '\377' | dd of="$TMP/BAD.trits" bs=1 seek=0 conv=notrunc status=none
if "$BIN" "$TMP/BAD.trits" "$INPUT/INIT_NULL.trits" "$INPUT/GATES.lut" >"$TMP/out" 2>"$TMP/err"; then
  echo "invalid trit accepted" >&2; exit 1
fi
grep -q '^BRIDGE_FAIL|' "$TMP/err"
cp "$INPUT/GATES.lut" "$TMP/BAD.lut"
printf '\377' | dd of="$TMP/BAD.lut" bs=1 seek=0 conv=notrunc status=none
if "$BIN" "$INPUT/INIT_TGV.trits" "$INPUT/INIT_NULL.trits" "$TMP/BAD.lut" >"$TMP/out" 2>"$TMP/err"; then
  echo "corrupt gate accepted" >&2; exit 1
fi
grep -q '^BRIDGE_FAIL|' "$TMP/err"
printf 'NEGATIVE|nontrit=REJECTED|gate_corruption=REJECTED|json=0\n' > "$OUTPUT/NEGATIVE.hbp"
(cd "$OUTPUT" && sha256sum LEAF-1.hbp LEAF-2.hbp LEAF-3.hbp LEAVES.hbi RUN.hbp NEGATIVE.hbp > SHA256SUMS)
python3 "$ROOT/bridge/taylor-green/verify_receipts.py" "$OUTPUT"
mkdir -p "$TMP/attest"
cp "$OUTPUT/LEAF-1.hbp" "$OUTPUT/LEAF-2.hbp" "$OUTPUT/LEAF-3.hbp" "$OUTPUT/LEAVES.hbi" "$OUTPUT/RUN.hbp" "$OUTPUT/NEGATIVE.hbp" "$OUTPUT/SHA256SUMS" "$TMP/attest/"
printf '\377' | dd of="$TMP/attest/LEAF-1.hbp" bs=1 seek=0 conv=notrunc status=none
if python3 "$ROOT/bridge/taylor-green/verify_receipts.py" "$TMP/attest" >"$TMP/audit-out" 2>"$TMP/audit-err"; then
  echo "corrupted leaf passed independent audit" >&2; exit 1
fi
grep -q '^INDEPENDENT_BRIDGE_AUDIT|status=FAIL|' "$TMP/audit-err"

cat "$OUTPUT/LEAVES.hbi"
tail -n 1 "$OUTPUT/RUN.hbp"
echo 'BRIDGE_GATE|steps=3|leaves=3|arms=3|input_sha256=PASS|tests=PASS|clippy=PASS|negative_controls=PASS|chain=PASS|status=PASS|json=0'
