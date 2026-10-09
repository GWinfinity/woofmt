---
name: Bug report
about: Report a wrong diagnostic, bad formatting, or a crash
labels: bug
---

## What happened

<!-- A clear description of the bug. -->

## Reproduction

```go
// Minimal Go file that triggers the issue:
```

```bash
# Command you ran:
woofmt check --format json .
# Actual output:
```

## Environment

- woofmt version (`woofmt --version`):
- OS:
- Go version (if gofmt-compat related): 

## Checklist

- [ ] I verified the issue reproduces with a minimal file
- [ ] For formatting bugs: the output differs from `gofmt` (paste both)
- [ ] For auto-fix bugs: state whether `--unsafe-fixes` was used
