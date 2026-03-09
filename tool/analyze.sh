#!/usr/bin/env bash
#
# Usage: analyze <crate-name> [analyzer-flags...]
#
# Builds mir-extractor and analyzer (release), extracts MIR from <crate-name>,
# computes treewidth, and writes all outputs to results/<crate-name>/.
#
# Special crate name: "std"
#   Extracts core + alloc + std via -Z build-std using test-crate as the
#   trigger crate (see crates/mir-extractor/README.md).
#
# Additional flags are forwarded to the analyzer binary.
set -euo pipefail

CRATE=${1:-}
if [[ -z "$CRATE" ]]; then
    echo "Usage: $0 <crate-name> [analyzer-flags...]" >&2
    exit 1
fi
shift

REPO=$(cd "$(dirname "$0")" && pwd)

# Detect --solver from forwarded args so the output dir reflects which solver ran.
SOLVER="auto"
prev=""
for arg in "$@"; do
    [[ "$prev" == "--solver" ]] && SOLVER="$arg"
    [[ "$arg" == --solver=* ]] && SOLVER="${arg#--solver=}"
    prev="$arg"
done

OUTDIR="$REPO/results/$CRATE/$SOLVER"
MIR="$REPO/results/$CRATE/mir.jsonl"
EXTRACTOR="$REPO/target/release/mir-extractor"
ANALYZER="$REPO/target/release/analyzer"

rm -rf "$OUTDIR"
mkdir -p "$OUTDIR"

echo "==> Extracting MIR for '$CRATE'…"
rm -f "$MIR"

if [[ "$CRATE" == "std" ]]; then
    cargo clean
    echo "==> Building tools…"
    cargo build --release -p mir-extractor -p analyzer
    MIR_OUTPUT="$MIR" MIR_CRATES="core,alloc,std" RUSTC="$EXTRACTOR" \
        cargo build -p test-crate \
            -Z build-std=core,alloc,std \
            --target x86_64-unknown-linux-gnu
else
    echo "==> Building tools…"
    cargo build --release -p mir-extractor -p analyzer
    cargo clean -p "$CRATE"
    MIR_OUTPUT="$MIR" RUSTC="$EXTRACTOR" \
        cargo build -p "$CRATE"
fi

echo "==> Computing treewidth…"
"$ANALYZER" "$MIR" --outdir "$OUTDIR" "$@"

echo ""
echo "Results in $OUTDIR/:"
ls -lh "$OUTDIR"
