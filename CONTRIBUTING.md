# 贡献指南

感谢关注 woofmt。0.x 阶段的贡献尤其需要「可复现」而不是「可吹牛」——
这是项目从 139 次下载走向生产可用的唯一路径。

## 快速开始

```bash
git clone https://github.com/GWinfinity/woofmt.git
cd woofmt
cargo build
cargo test        # 全部测试必须通过（含幂等性回归）
cargo clippy -- -D warnings
```

## 规则注册规范（P0 级约束）

v0.1.5 存在多个 phase 文件重复注册同一规则码（如 `SA5000` 同时出现在
`p0_critical.rs` 与 `p0_runtime.rs`）的问题。新贡献请遵守：

1. **一个规则码只允许注册一次**。提交前运行 `cargo test` 并检查
   `woofmt rules --all` 输出无重复行。
2. 新规则先在本文件追加登记，再写实现。
3. 规则必须携带 `min_go_version`，涉 generics/Fuzzing/Workspace 的规则
   不得对老版本误报。

## Auto-Fix 的安全分级

- **safe**：纯机械、不可能改变程序语义（如删除行尾空白）。
  通过 `Fix::safe(...)` 构造，`--fix` 默认应用。
- **unsafe**：可能改变语义（如删除"未使用"的 import——副作用导入会被误删）。
  通过 `Fix::unsafe_(...)` 构造，仅 `--unsafe-fixes` 时应用。

拿不准就标 unsafe。误报一次 unsafe fix 造成的损失 > 十次漏修。

## 格式化改动的铁律

任何触碰 `src/formatter/` 的 PR 必须保证：

1. **幂等性**：`format(format(x)) == format(x)`（`tests/gofmt_compat.rs` 会拦截）。
2. **gofmt 兼容**：`scripts/gofmt_compat.sh` 在语料上的 diff 只能减少不能增加。
3. 新语法结构请先在 `testdata/gofmt_corpus/` 加语料文件，复现失败，再修。

## 基准测试纪律

- 对外宣传的性能数字必须能被 `benchmark/` 中的脚本复现。
- 禁止混用 workload 口径（冷启动 vs 热跑、单目录 vs 全仓库）。
- README 中的数字变更需在 PR 中贴出完整 hyperfine 输出。

## 提交规范

- Conventional Commits：`fix:` / `feat:` / `docs:` / `perf:` / `refactor:`
- 每个 PR 聚焦一件事；跨 P0/P1/P2 的混合 PR 会被要求拆分。

## 版本与发布

见 [docs/SEMVER_POLICY.md](docs/SEMVER_POLICY.md)。发布由维护者执行
（`.github/workflows/release.yml`）。

## 行为准则

见 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)。
