# 从 golangci-lint 迁移到 woofmt

woofmt 的定位是 **golangci-lint 的速度补丁**，不是完全替代品。
本文给出三条渐进式迁移路线，按采纳成本从低到高排列。

## 路线一：编辑器内实时检查（零迁移成本）

保留现有 `golangci-lint` CI 不动，只在编辑器/LSP 里加 woofmt 做毫秒级实时反馈：

```jsonc
// VS Code settings.json（配合 woofmt 的 LSP/CLI 快速模式）
{
  "go.lintTool": "woofmt",
  "go.lintOnSave": "workspace"
}
```

价值：golangci-lint 冷启动 ~100ms~数秒，不适合保存时实时反馈；woofmt 热跑 2ms 级。
CI 的深度检查（staticcheck 类型分析）照旧。

## 路线二：CI 分层（推荐）

把 golangci-lint 的规则拆成两层：

| 层 | 工具 | 时机 | 规则集 |
|----|------|------|--------|
| 快速层 | woofmt | 每次 push / 保存 | E/I/F/S/D 系列（语法级） |
| 深度层 | golangci-lint（staticcheck 等） | PR / 合并前 | SA 系列（类型流分析） |

`woof.toml` 示例：

```toml
[linter]
select = ["E", "I", "F", "S", "D"]
ignore = ["E118"]   # 行长交给 golangci-lint 管就忽略它
```

## 路线三：完全替换（不推荐，除非项目浅）

仅当你的项目不依赖 staticcheck 的类型敏感检查
（接口断言、nil 逃逸、锁的流分析）时才可行。

## 规则映射表（golangci-lint linter → woofmt code）

| golangci-lint linter | woofmt 规则 | 覆盖度 |
|----------------------|-------------|--------|
| govet（copylocks 除外） | F 系列、GEN 系列 | 部分 |
| staticcheck SA* | SA* 同名码 | 语法级近似 ⚠️ |
| gosimple | E7xx | 部分 |
| gofmt / gofumpt | `woofmt fmt`（gofmt 兼容性见 scripts/gofmt_compat.sh） | 结构级 |
| goimports | I001–I003 | 部分 |
| revive exported | D001 / D002 | 对应 |
| gocritic | B 系列 | 子集 |
| unused (go-tools) | F401 / F402 / F841 | ⚠️ 语法级，无跨文件分析 |
| errcheck | S002 / F404 | ⚠️ 语法级 |
| lll | E118 | 对应 |
| wsl | E 系列（部分） | 子集 |

## 配置迁移速查

```yaml
# .golangci.yml（旧）
linters:
  enable: [govet, staticcheck, revive, goimports]
issues:
  exclude-rules:
    - path: _test\.go
      linters: [dupl]
```

```toml
# woof.toml（新）
[linter]
select = ["E", "I", "F", "S", "D"]
ignore = ["E118"]

[lint.exclude]
paths = ["_test\\.go"]

[formatter]
use_tabs = true      # gofmt 默认 Tab
tab_width = 4
line_length = 120
```

## 已知边界（迁移前必读）

1. **无类型信息**：涉及类型流分析的检查（`SA5011` nil 解引用、`SA5010`
   类型断言等）只能做语法近似，无法达到 staticcheck 精度。
2. **无跨包分析**：`unused` 只能看单文件，跨包死代码仍需 golangci-lint。
3. **Auto-Fix 分级**：`--fix` 只应用 safe 级修复（如 E115 行尾空白）；
   改语义的修复需显式 `--unsafe-fixes`。
4. **版本纪律**：0.x 阶段规则集与 CLI 面可能在 minor 版本内变化，
   锁定版本使用（CI 中 `cargo install woofmt --version 0.1.x --locked`）。
