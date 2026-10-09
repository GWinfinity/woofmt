# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ (best effort — pre-1.0) |

## Reporting a vulnerability

**Do not open a public issue for security reports.**

Email the maintainers via GitHub Security Advisories
("Report a vulnerability" on the repository's Security tab), or open a
private advisory. Include:

- affected version (`woofmt --version`)
- a minimal Go input file that reproduces the issue
- expected vs actual behavior

## Threat model notes

woofmt reads and (with `--fix` / `fmt`) rewrites Go source files:

- It does **not** execute parsed code.
- `--fix` performs byte-range splicing only; ranges are bounds-checked and
  overlap-protected (`src/fixer.rs`).
- Auto-fix output is **never guaranteed safe** — review diffs before
  committing. CI should use `fmt --check` / `check` rather than blind `--fix`.

## Known limitations (not vulnerabilities, but be aware)

- The LSP mode is experimental; do not expose it over untrusted sockets.
- Configuration files (`woof.toml`) are parsed with `toml`/`serde`; no
  arbitrary code execution is possible, but path fields in config are
  resolved relative to CWD — run woofmt only on trusted repositories.
