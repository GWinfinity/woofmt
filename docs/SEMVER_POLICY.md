# 语义化版本与兼容性策略（0.x 阶段）

woofmt 遵循 [Semantic Versioning](https://semver.org/)，但 0.x 阶段有特殊说明。

## 当前阶段：0.1.x — 不稳定期

**任何 minor 版本内都可能变化的内容：**

- 规则集（新增/删除/重命名规则码）
- CLI 参数面（子命令、旗标）
- `woof.toml` 配置 schema
- `Fix` / `Diagnostic` 等库 API 字段

**承诺不变的内容（即使在 0.x）：**

- 既有规则码的语义方向（`SA5000` 永远指 nil map 赋值，不会改指别的）
- safe/unsafe fix 分级的安全承诺：`--fix` 永不应用 unsafe 级修复
- 格式化输出幂等性：`format(format(x)) == format(x)`（回归测试强制）

## CI 使用建议

```bash
# 锁定版本 + --locked 禁止依赖漂移
cargo install woofmt --version 0.1.5 --locked
```

在 `woof.toml` 中不要写通配规则集依赖（`select = ["all"]`），
显式列出前缀以便升级时可审阅差异。

## 1.0 门槛（进入 stable 的条件）

1. 规则重复注册清零，规则码空间冻结
2. gofmt 逐字节兼容语料覆盖 ≥ 95% 常见构造（CI 红线）
3. 第三方项目试点 ≥ 3 个，误报率数据公开
4. ≥ 2 名核心维护者，重大变更走 RFC 流程
5. 库 API（`Diagnostic` / `Fix` / `Config`）冻结并补齐迁移文档

## 变更记录

所有用户可感知的变更必须写入 [CHANGELOG.md](../CHANGELOG.md)，
格式遵循 [Keep a Changelog](https://keepachangelog.com/)。
