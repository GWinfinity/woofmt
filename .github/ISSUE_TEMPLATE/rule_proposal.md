---
name: Rule / feature proposal
about: Propose a new lint rule, formatter feature, or ecosystem integration
labels: enhancement
---

## Proposal

<!-- One sentence: what should woofmt do that it doesn't? -->

## Which upstream tool has this?

<!-- golangci-lint / staticcheck / gofumpt / gci ... and the exact rule name. -->

## Why tree-sitter (no type info) is sufficient

<!-- woofmt rules are syntax-level. If this needs type flow analysis,
     the honest answer may be "keep using staticcheck" — explain your case. -->

## Bad / Good example

```go
// Bad — should be flagged:
```

```go
// Good — should pass:
```

## Auto-fix?

- [ ] Safe fix proposed (purely mechanical)
- [ ] Unsafe fix proposed (may change semantics) — requires `--unsafe-fixes`
- [ ] No fix
