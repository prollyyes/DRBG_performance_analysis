# 🎲 DRBG Performance Analysis

<div align="center">

**A comprehensive benchmarking suite for Deterministic Random Bit Generators in Rust**

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[Features](#-features) • [Quick Start](#-quick-start) • [Generators](#-generators-tested) • [Results](#-results) • [Documentation](#-documentation)

</div>

---

## 📖 Overview

This project implements and rigorously benchmarks three state-of-the-art Deterministic Random Bit Generator (DRBG) architectures in Rust, comparing their performance, memory efficiency, and statistical quality across multiple output scales. The implementations leverage Rust's zero-cost abstractions and type safety to provide both high performance and cryptographic reliability.

### 🎯 Generators Tested

| Generator | Primitive | Key Features |
|-----------|-----------|--------------|
| **ChaCha20** | Stream Cipher | 20-round ARX construction, excellent software performance |
| **AES-256-CTR** | Block Cipher + Counter Mode | FIPS-validated, hardware acceleration (AES-NI) |
| **BLAKE3** | Extendable Output Function | Parallel tree hashing, optimized for modern CPUs |

Each generator processes entropy through a carefully designed trait-based architecture, producing cryptographically secure bit strings with statistical properties validated through comprehensive testing.

## ✨ Features

- **🚀 High-Performance Implementation**: Built in Rust with release-mode optimizations
- **📊 Statistical Analysis**: 50 runs per configuration with mean, standard deviation, and coefficient of variation
- **📈 Visual Reports**: Automated plot generation showing performance trends across scales
- **🔬 Bit Distribution Testing**: Monobit frequency analysis validates randomness quality
- **📦 Packed Representation**: Memory-efficient bit storage with byte-level packing
- **⚡ Multiple Scales**: Tests from 10⁴ to 10⁷ bits (four orders of magnitude)

## 🏆 Key Results

From 50-run statistical analysis across all configurations:

- **BLAKE3** achieves **1.33ms** (±0.17ms) for 10⁷ bits — **fastest at scale**
- **ChaCha20** delivers **2.07ms** (±0.46ms) for 10⁷ bits — **low variance, consistent performance**
- **AES-256-CTR** completes in **6.99ms** (±1.11ms) for 10⁷ bits — **FIPS compliance**

All generators maintain excellent bit balance (ones ratio within ±0.004% of 0.5) and pass monobit frequency tests with p-values > 0.82.

## 🚀 Quick Start

### Prerequisites

- **Rust toolchain** (stable channel recommended)
- `cargo` build system

### Installation

```bash
git clone https://github.com/prollyyes/DRBG_performance_analysis.git
cd DRBG_performance_analysis
```

### Running the Benchmark

```bash
cargo run --release
```

The benchmark automatically:
1. Initializes all three generators with identical entropy
2. Generates bit strings at 4 different scales (10⁴ to 10⁷ bits)
3. Repeats each configuration 50 times for statistical validity
4. Records timing, memory consumption, and bit distribution metrics
5. Computes aggregate statistics (mean, std, CV)
6. Generates performance visualization plots

### Output Files

```
results/
├── metrics.csv          # Raw measurements: 600 data points (3 generators × 4 sizes × 50 runs)
├── summary.csv          # Aggregate statistics per configuration
└── plots/
    ├── time_ms.png      # Execution time comparison (log-log scale)
    ├── memory_bytes.png # Memory consumption by output size
    └── ones_ratio.png   # Bit distribution quality
```

## 🔧 Customization

Modify benchmark parameters in `src/main.rs`:

```rust
const RUNS: usize = 50;  // Number of repetitions per configuration
const TARGET_LENGTHS: [usize; 4] = [10_000, 100_000, 1_000_000, 10_000_000];
```

## 🏗️ Architecture

### Core Components

- **`drbg.rs`**: Trait definition and three DRBG implementations
  - `DRBG` trait with `generate_bits()`, `reseed()`, and `name()` methods
  - `BitString` with packed u8 representation and bit counting
  - ChaCha20, AES-CTR, and BLAKE3 implementations

- **`main.rs`**: Benchmarking harness
  - Monotonic timing using `std::time::Instant`
  - 50-run statistical aggregation
  - CSV output and plotting via `plotters`

### Design Principles

1. **Fair Comparison**: All generators initialized with BLAKE3-derived entropy
2. **Zero-Cost Abstractions**: Trait-based polymorphism with monomorphization
3. **Statistical Rigor**: Multiple runs capture variance and ensure reproducibility
4. **Memory Efficiency**: Packed bit representation reduces space overhead

## 📚 Documentation

A comprehensive academic report (`report_final.pdf`) provides:

- Cryptographic foundations for each generator
- Implementation methodology and trait architecture  
- Detailed benchmark results with statistical analysis
- Discussion of performance trade-offs
- Limitations and future work

### Report Highlights

- **9 pages** of detailed analysis
- **3 performance graphs** with log-log scaling
- **10 academic references** (NIST SP 800-90A, RFC 8439, etc.)
- Statistical validation including monobit frequency tests
- Platform-specific considerations (Apple Silicon, ARM Crypto Extensions)

## 🔬 Statistical Validation

Each generator undergoes rigorous testing:

- **Bit Distribution**: Chi-square test for uniform 0/1 distribution
- **Monobit Frequency**: NIST-style z-score calculation with p-values
- **Variance Analysis**: Coefficient of variation tracks measurement stability
- **Scaling Behavior**: Log-log plots reveal algorithmic complexity

## 📊 Performance Insights

**Coefficient of Variation Trends:**
- Small outputs (10⁴ bits): CV up to 177% due to microsecond timing jitter
- Large outputs (10⁷ bits): CV drops to 13-22% as execution time dominates measurement noise

**Throughput Comparison:**
- BLAKE3: ~7.5 Gbps at 10⁷ bits
- ChaCha20: ~4.8 Gbps at 10⁷ bits  
- AES-CTR: ~1.4 Gbps at 10⁷ bits

## 🤝 Contributing

Contributions are welcome! Areas for enhancement:

- Additional generators (XChaCha20, SHAKE256, HC-256)
- Full NIST SP 800-22 test suite integration
- Cross-platform benchmarking (x86, ARM, RISC-V)
- Side-channel analysis (constant-time verification)
- GPU-accelerated implementations

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- **Rust Cryptography Working Group** for high-quality crates
- **NIST** for DRBG standards and statistical test suites
- **D.J. Bernstein** for ChaCha20 design
- **BLAKE3 Team** for XOF specifications

---

<div align="center">

**Built with ❤️ using Rust**

*For academic research in cryptographic systems • MSc Cybersecurity*

</div>
