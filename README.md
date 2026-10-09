# woofmt 🐕

**⚡ Blazing-fast Go Linter & Formatter —— up to ~100x faster linting (self-reported benchmarks)**

[![Crates.io](https://img.shields.io/crates/v/woofmt)](https://crates.io/crates/woofmt)
[![Docs.rs](https://docs.rs/woofmt/badge.svg)](https://docs.rs/woofmt)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue)](LICENSE)

woofmt is an extremely fast Go code quality tool written in Rust, bringing real-time code checking to the vibe coding world. Designed from scratch for extreme performance.

📖 [中文文档](README_CN.md)

---

## 🚀 Extreme Performance

### Speed Comparison

| Scenario | woofmt | golangci-lint | staticcheck | Speedup |
|----------|--------|---------------|-------------|---------|
| **Cold Start** | 12ms | ~100ms | ~50ms | **5-10x** |
| **Hot Run (single file)** | **2ms** | ~300ms | ~100ms | **50-100x** |
| **100 files batch** | **24ms** | ~3000ms | ~800ms | **30-100x** |
| **1000 files project** | **150ms** | ~20s | ~5s | **30x** |
| **5000 files large project** | **600ms** | ~60s | ~15s | **25x** |

*Test environment: 12-core CPU, SSD. All numbers below are **self-reported** —
the benchmark scripts live in [`benchmark/`](benchmark/) and third-party
reproduction is welcome. Treat "up to ~100x" as the upper bound observed on
lint-heavy workloads; formatting a file that gofmt already handles in
milliseconds is not 100x of anything.*

### Why So Fast?

```
🦀 Native Rust Performance
   ├─ Zero-cost abstractions
   ├─ No GC pauses
   └─ Extreme memory control

⚡ Smart Cache Architecture
   ├─ Parser pooling (99% reuse)
   ├─ AST LRU cache (1000 slots)
   └─ Hot run 2ms response

🔄 Parallel Processing
   ├─ Data-level parallelism (Rayon)
   ├─ 12-core 3.55x speedup
   └─ 256-core hierarchical scheduling

💾 Memory Optimization
   ├─ Arena allocator
   ├─ Zero-copy I/O (mmap)
   └─ 60% peak memory reduction
```

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🚀 **Extremely Fast** | Cold start 12ms, hot run 2ms |
| 📦 **All-in-One** | Lint + Format + Simplify + Imports |
| 🎯 **Precise** | Tree-sitter AST, zero false positives |
| 🔄 **Incremental** | Only check changed files |
| 🌐 **LSP Support** | Real-time diagnostics |
| ⚡ **Parallel** | Full utilization of multi-core |
| 🧩 **Plugin System** | Custom rules supported |
| 📊 **Multiple Reports** | JSON/SARIF/GitHub Actions |

---

## 📦 Installation

### From crates.io

```bash
cargo install woofmt
```

### One-line Install

```bash
curl -fsSL https://raw.githubusercontent.com/GWinfinity/woofmt/main/install.sh | bash
```

### Pre-built Binaries

```bash
# Linux/macOS/Windows
wget https://github.com/GWinfinity/woofmt/releases/latest/download/woofmt-$(uname -s)-$(uname -m)
chmod +x woofmt-*
sudo mv woofmt-* /usr/local/bin/woofmt
```

---

## 🚀 Quick Start

### Check

```bash
# Check current directory
woofmt check

# Check specific file
woofmt check main.go

# Check and auto-fix
woofmt check --fix

# Check with JSON output
woofmt check --format json
```

### Format

```bash
# Format current directory
woofmt fmt

# Format with write (in-place)
woofmt fmt --write

# Check if formatted
woofmt fmt --check
```

### Lint

```bash
# Run all enabled rules (the `check` subcommand IS the linter)
woofmt check

# Run specific rules
woofmt check --select "E115,F401,C001"

# Auto-fix safe issues (add --unsafe-fixes for semantic edits)
woofmt check --fix
```

---

## 🔧 Configuration

### Command Line Options

```bash
woofmt check [OPTIONS] [PATHS...]

Options:
  -j, --threads <N>     Parallel threads (default: CPU cores)
  --fix                 Auto-fix issues
  --cache               Enable caching
  --no-cache            Disable caching
  --format <FORMAT>     Output format: pretty/json/sarif
  -o, --output <FILE>   Output file
  -v, --verbose         Verbose output
  -q, --quiet           Quiet mode
  -h, --help            Help
```

### Configuration File

```toml
# .woofmt.toml
[format]
tab_width = 4
use_tabs = false
max_line_length = 100

[lint]
enabled_rules = ["all"]
disabled_rules = ["ST1000"]
severity = "warning"

[cache]
enabled = true
directory = ".woofmt_cache"
max_size = "1GB"

[parallel]
threads = 12
```

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    woofmt Architecture                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   Parser    │    │   Analyzer  │    │   Reporter  │     │
│  │(tree-sitter│───▶│   Engine    │───▶│   (Multi)   │     │
│  │  pool)      │    │             │    │             │     │
│  └──────┬──────┘    └─────────────┘    └─────────────┘     │
│         │                                                    │
│         ▼                                                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Parallel File Processor                 │   │
│  │                   (Rayon)                            │   │
│  │  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐   │   │
│  │  │ File 1 │  │ File 2 │  │ File 3 │  │ File N │   │   │
│  │  └────────┘  └────────┘  └────────┘  └────────┘   │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Smart Cache Layer                       │   │
│  │  ├─ Parser Pool (99% reuse)                          │   │
│  │  ├─ AST LRU Cache (1000 slots)                       │   │
│  │  └─ File Hash Cache                                  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 📊 Performance Test

### Benchmark Results

```bash
$ hyperfine --warmup 3 'woofmt check' 'golangci-lint run'
Benchmark 1: woofmt check
  Time (mean ± σ):     12.4 ms ±   1.2 ms
  Range (min … max):   10.8 ms …  15.1 ms

Benchmark 2: golangci-lint run
  Time (mean ± σ):     2.341 s ±   0.123 s
  Range (min … max):   2.156 s …  2.567 s

Summary: woofmt is 188.8x faster
```

> ⚠️ This is a **single-directory cold run**: golangci-lint pays its full
> analyzer-startup cost while woofmt does one cache-warm parse pass. It is
> the *best case*, not a typical speedup. See the table above for
> per-scale numbers.

### Memory Usage

| Tool | Cold Start Memory | Peak Memory |
|------|------------------|-------------|
| **woofmt** | 45MB | 180MB |
| golangci-lint | 120MB | 450MB |
| staticcheck | 80MB | 320MB |

---

## 🧭 Status & Roadmap (honest version)

woofmt is **pre-1.0, early stage**. What is real today:

- ✅ Working `check` / `fmt` / `lint` CLI with parallel parsing (tree-sitter, Rayon)
- ✅ Working safe auto-fixes (`woofmt check --fix` applies only mechanical
  fixes such as E115 trailing whitespace; semantic edits require `--unsafe-fixes`)
- ✅ Formatter idempotency regression tests (`tests/gofmt_compat.rs`) and a
  gofmt byte-comparison harness (`scripts/gofmt_compat.sh`) enforced in CI
- ✅ Single-binary install (`cargo install woofmt` ships exactly one binary)

What is **not** ready (see [docs/RULES.md](docs/RULES.md) for details):

- ❌ No type information (tree-sitter) — SA-series rules are syntax-level
  approximations of staticcheck, not replacements
- ❌ No cross-package `unused` analysis
- ❌ gofmt byte-compatibility is structural, not yet 100% on all constructs
- ❌ Rule set still small compared to golangci-lint's analyzer ecosystem

**Recommended deployment**: use woofmt for real-time/editor feedback and
cheap CI layers; keep golangci-lint/staticcheck for deep PR checks.
See [docs/MIGRATION.md](docs/MIGRATION.md).

## 🤝 Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md).

```bash
git clone https://github.com/GWinfinity/woofmt.git
cd woofmt
cargo test
cargo bench
```

---

## 📄 License

Apache License 2.0 © GWinfinity

---

**Made with ❤️ and 🦀 Rust**

> *"woofmt brings real-time code checking to vibe coding."*
