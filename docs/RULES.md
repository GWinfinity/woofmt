# woofmt 规则目录

> 生成口径：`woofmt rules --all`（v0.1.5 之后）。每条规则给出 **Bad / Good** 示例与对应的 golangci-lint 生态工具映射。
>
> ⚠️ 诚实声明：woofmt 基于 tree-sitter（容错解析器，**无类型信息**）。
> 因此所有规则均为**语法级**检查；标 ⚠️ 的规则在边界场景下可能误报，
> 标 🧪 的规则目标是 staticcheck 的同名 SA 规则，但**检查深度弱于 staticcheck**
> （staticcheck 有完整类型流分析）。生产环境建议 `woofmt` 与 `staticcheck` 互补使用。

## 规则前缀与上游映射

| 前缀 | 类别 | 对标工具（golangci-lint 生态） |
|------|------|-------------------------------|
| `SA*` | 运行时错误 / 可疑代码 | staticcheck（同名规则） |
| `F*` | 逻辑错误 | govet / staticcheck |
| `E*` | 代码风格与语法 | gosimple / 自研 |
| `I*` | 导入管理 | gci / goimports |
| `B*` | 常见反模式 | go-critic / bodyclose |
| `C*` | 并发安全 | govet copylocks / 自研 |
| `S*` | 风格约定 | revive / stylecheck |
| `D*` | 文档注释 | revive exported |
| `GEN*` | 泛型 | govet |
| `FUZZ*` | Fuzzing 测试 | govet |
| `WS*` | Go workspace | 自研 |

## P0 · 运行时错误与并发（精选）

### SA5000 · assignment-to-nil-map（向 nil map 赋值 → panic）

```go
// Bad
var m map[string]int
m["k"] = 1 // panic: assignment to entry in nil map

// Good
m := make(map[string]int)
m["k"] = 1
```

### SA2000 · sync-waitgroup-add-goroutine（WaitGroup.Add 放进 goroutine）

```go
// Bad
var wg sync.WaitGroup
for i := 0; i < 10; i++ {
    go func() {
        wg.Add(1) // 主 goroutine 可能先 Wait
        defer wg.Done()
    }()
}
wg.Wait()

// Good
var wg sync.WaitGroup
for i := 0; i < 10; i++ {
    wg.Add(1)
    go func() {
        defer wg.Done()
    }()
}
wg.Wait()
```

### C001 · bad-lock（拷贝含锁结构体 / 未配对 Unlock）

```go
// Bad
type S struct{ mu sync.Mutex }
func (s S) Lock() { s.mu.Lock() } // 拷贝了 Mutex

// Good
func (s *S) Lock() { s.mu.Lock() }
```

### SA1019 · deprecated-function

```go
// Bad
rand.Seed(42) // 已弃用

// Good
r := rand.New(rand.NewSource(42))
```

### SA1002 · time-parse-format（Go 参考时间是 2006-01-02）

```go
// Bad
t, _ := time.Parse("2024-01-02", s)

// Good
t, _ := time.Parse("2006-01-02", s)
```

### SA1029 · context-withvalue-key

```go
// Bad
ctx = context.WithValue(ctx, "key", v) // string key 易冲突

// Good
type ctxKey struct{}
ctx = context.WithValue(ctx, ctxKey{}, v)
```

## F 系列 · 逻辑错误

### F401 · unused-import

```go
// Bad
import "fmt" // 未使用

// Good
import "fmt"
func main() { fmt.Println("hi") }
```

### F831 / F901 · loop-variable-capture / unreachable-code

```go
// Bad
for i := 0; i < 3; i++ {
    go func() { fmt.Println(i) }() // 捕获循环变量
}
return
log.Println("never") // 不可达

// Good
for i := 0; i < 3; i++ {
    i := i
    go func() { fmt.Println(i) }()
}
return
```

## E 系列 · 代码风格（带 safe fix）

| 规则 | 说明 | Auto-Fix |
|------|------|----------|
| E101 | 混合空格与 Tab 缩进 | unsafe |
| E115 | 行尾空白字符 | **safe**（已实现） |
| E116 | 文件末尾多余空行 | safe（计划） |
| E117 | 文件末尾缺换行 | safe（计划） |
| E118 | 行长超限 | — |
| E201–E203 | 导入区布局 | safe（计划） |
| E301 | 空代码块 | — |
| E712/E713 | 与 true/false 显式比较 | safe（计划） |

## I 系列 · 导入管理

| 规则 | 说明 | 对标 |
|------|------|------|
| I001 | 导入未排序 | gci |
| I002 | 缺少标准库/第三方/本地分组 | gci |
| I003 | 标准库导入顺序不对 | goimports |
| I008 | 点导入不推荐 | revive `dot-imports` |

## S / D 系列 · 风格与文档

| 规则 | 说明 | 对标 |
|------|------|------|
| S001 | 命名返回值裸返回 | revive `bare-return` |
| S005 | 函数名驼峰 | stylecheck |
| S006 | 接收器命名 | revive `receiver-naming` |
| S007 | error 变量命名 ErrXxx | stylecheck ST1012 |
| S008 | 包名规范 | revive `package-name` |
| D001 | 导出标识符缺文档 | revive `exported` |
| D002 | 包缺文档注释 | revive `package-comments` |

## 重复注册问题（已知）

v0.1.5 中 `SA5000/SA5007/SA5011/SA5008/SA5010/SA1020` 等在多个 phase 文件中重复注册，
`woofmt rules --all` 会显示重复项。已在治理计划中列入修复（见 CONTRIBUTING.md「规则注册规范」）。
