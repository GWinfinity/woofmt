# Changelog

All notable changes to this project will be documented in this file.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Fixed

- **Formatter: import declarations were dropped.** `import ( ... )` blocks
  produced empty parens because import specs live one level deeper in the
  tree-sitter AST (`import_spec_list`). All imports are now emitted.
- **Formatter: missing space before method result type** — `(g *Greeter) Greet()string`
  → `Greet() string`.
- **Formatter: non-idempotent blank lines** — each formatting pass added one
  extra blank line before closing braces; `fmt --check` would oscillate forever
  in CI. `Printer::newline` now collapses consecutive breaks.
- **CLI: `woofmt fmt` was impossible to invoke.** The top-level positional
  `FILES` had `default_value = "."`, so clap consumed the subcommand name as a
  file path. Added `fmt` alias for `format` and fixed the positional definition.
- **README/CLI drift**: `woofmt lint --rules` documented but never existed —
  Quick Start now uses the real `check --select` surface.

### Added

- **Real auto-fix engine** (`src/fixer.rs`): byte-exact splicing with
  overlap protection, bounds checking, and safe/unsafe grading.
  `--fix` now actually applies fixes (it previously printed "自动修复功能暂时禁用").
- Safe fix for E115 (trailing whitespace) as the first wired-up rule.
- `--unsafe-fixes` flag on `woofmt check`.
- Formatter idempotency + fixer regression tests (`tests/gofmt_compat.rs`).
- gofmt byte-comparison harness (`scripts/gofmt_compat.sh`) for CI.
- `docs/RULES.md` — rule catalog with Bad/Good examples and golangci-lint mapping.
- `docs/MIGRATION.md` — golangci-lint → woofmt migration guide.
- `docs/SEMVER_POLICY.md`, `CONTRIBUTING.md`, `SECURITY.md`,
  `CODE_OF_CONDUCT.md`, GitHub issue templates.
- CI workflow (`.github/workflows/ci.yml`): 3-OS build/test/clippy,
  gofmt compat job, single-binary install regression guard.

### Changed

- **`cargo install woofmt` now ships exactly one binary.** The
  `benchmark_256core` binary is gated behind `--features benchmark`.

## [0.1.5] — 2026-03-22

Initial crates.io release (see git history).
