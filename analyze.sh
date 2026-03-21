#!/usr/bin/env bash
#
# Usage: analyze.sh <crate-name> [analyzer-flags...]
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

# Prefer rustup proxies if installed. This avoids accidentally using Homebrew
# stable rustc/cargo when rust-toolchain.toml requires nightly.
if [[ -x "$HOME/.cargo/bin/cargo" && -x "$HOME/.cargo/bin/rustc" ]]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

check_toolchain() {
    if ! command -v cargo >/dev/null 2>&1 || ! command -v rustc >/dev/null 2>&1; then
        echo "error: cargo/rustc not found in PATH." >&2
        exit 1
    fi

    local rustc_path cargo_path rustc_ver cargo_ver
    rustc_path=$(command -v rustc)
    cargo_path=$(command -v cargo)

    if ! rustc_ver=$(rustc --version 2>&1); then
        cat >&2 <<EOF
error: failed to run rustc from PATH.
rustc path: $rustc_path
rustc output:
$rustc_ver

Ensure rustup nightly is installed and selected for this repo:
  rustup toolchain install nightly
EOF
        exit 1
    fi

    if ! cargo_ver=$(cargo --version 2>&1); then
        cat >&2 <<EOF
error: failed to run cargo from PATH.
cargo path: $cargo_path
cargo output:
$cargo_ver
EOF
        exit 1
    fi

    if [[ "$rustc_ver" != *nightly* ]]; then
        cat >&2 <<EOF
error: mir-extractor requires nightly rustc with rustc internals.
current rustc: $rustc_ver ($rustc_path)
current cargo: $cargo_ver ($cargo_path)

Fix with rustup on macOS/Linux:
  export PATH="\$HOME/.cargo/bin:\$PATH"
  rustup toolchain install nightly
  rustup component add --toolchain nightly rustc-dev rust-src llvm-tools-preview
EOF
        exit 1
    fi

    local sysroot host libdir
    sysroot=$(rustc --print sysroot)
    host=$(rustc -vV | awk -F': ' '/^host: / { print $2 }')
    libdir="$sysroot/lib/rustlib/$host/lib"

    if ! compgen -G "$libdir/librustc_driver-*" >/dev/null; then
        cat >&2 <<EOF
error: rustc-dev component not detected for active toolchain.
expected rustc_driver in: $libdir

Install missing components:
  rustup component add --toolchain nightly rustc-dev rust-src llvm-tools-preview
EOF
        exit 1
    fi

    if [[ ! -d "$sysroot/lib/rustlib/src/rust/library" ]]; then
        cat >&2 <<EOF
error: rust-src component not detected for active toolchain.
expected std sources at: $sysroot/lib/rustlib/src/rust/library

Install missing components:
  rustup component add --toolchain nightly rust-src
EOF
        exit 1
    fi
}

check_toolchain

configure_rustc_private_runtime() {
    local sysroot host libdir os
    sysroot=$(rustc --print sysroot)
    host=$(rustc -vV | awk -F': ' '/^host: / { print $2 }')
    libdir="$sysroot/lib/rustlib/$host/lib"
    os=$(uname -s)

    export LIBRARY_PATH="$libdir:${LIBRARY_PATH:-}"

    if [[ "$os" == "Darwin" ]]; then
        export DYLD_FALLBACK_LIBRARY_PATH="$libdir:${DYLD_FALLBACK_LIBRARY_PATH:-}"
        export DYLD_LIBRARY_PATH="$libdir:${DYLD_LIBRARY_PATH:-}"
    else
        export LD_LIBRARY_PATH="$libdir:${LD_LIBRARY_PATH:-}"
    fi
}

configure_rustc_private_runtime

HOST_TARGET=$(rustc -vV | awk -F': ' '/^host: / { print $2 }')
TARGET_TRIPLE=${ANALYZE_TARGET:-$HOST_TARGET}

OUTDIR="$REPO/results/$CRATE"
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
            --target "$TARGET_TRIPLE"
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
