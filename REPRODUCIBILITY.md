# Reproducibility

This project depends on a nightly Rust toolchain because `mir-extractor`
links against `rustc_private` crates and the `std` analysis path uses
`-Z build-std`.

The exact toolchain and environment used for the experiments in the paper are:
- Rust toolchain: `nightly-2026-03-10`
- `rustc`: `1.96.0-nightly (0c68443b0 2026-03-10)`
- `cargo`: `1.96.0-nightly (90ed291a5 2026-03-05)`
- Rust components required by this repo: `rustc-dev`, `rust-src`, `llvm-tools`, `rustfmt`, `clippy`, `rust-analyzer`
- Host/target triple used in this environment: `aarch64-apple-darwin`
- Operating system: `macOS 26.3.1 (build 25D2128)`
- Kernel: `Darwin 25.3.0`
- Native C++ compiler used to build the FlowCutter FFI bridge: `Apple clang 17.0.0 (clang-1700.6.4.2)`
- FlowCutter oracle source revision: `7f94541b0119284ea9322d528cef420e041539b6`

The Rust dependency closure is pinned in `Cargo.lock`. The most relevant direct crate versions are:
- `serde 1.0.228`
- `serde_json 1.0.149`
- `clap 4.5.60`
- `rayon 1.11.0`
- `cc 1.2.56`
