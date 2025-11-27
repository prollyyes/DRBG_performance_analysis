# Random Numbers DRBG Benchmark

Benchmarks three deterministic random bit generators (DRBGs) implemented in Rust:
- ChaCha20-based DRBG (`rand_chacha`)
- AES-256-CTR DRBG with BLAKE3-derived key/counter
- BLAKE3 XOF DRBG

The harness produces packed bit strings for lengths 10^4, 10^5, 10^6, and 10^7 bits, runs each configuration 50 times, and records timing, space, and bit-balance metrics. Plotters renders summary figures, and `report_final.pdf` documents the methodology and results.

## Prerequisites
- Rust toolchain (stable) with `cargo`

## Run the benchmark
```sh
cargo run --release
```
Outputs:
- `results/metrics.csv`: per-run measurements (50 runs per generator/length)
- `results/summary.csv`: mean and sample standard deviation per configuration
- `results/plots/{time_ms,memory_bytes,ones_ratio}.png`: plots derived from summary data

## Notes
- `.gitignore` excludes LaTeX intermediates and build artifacts while keeping PDFs and source files.
- Adjust `RUNS` or `TARGET_LENGTHS` in `src/main.rs` to explore different repetition counts or output sizes. Re-run the benchmark to refresh metrics and plots.
