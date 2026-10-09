//! P0 Critical Rules - 核心必须实现的规则
//!
//! 包括:
//! - SA1000-SA1030: API 使用错误 (staticcheck)
//! - SA2000-SA2003: 并发问题
//! - SA5000-SA5012: 运行时问题
//! - 泛型、Fuzzing、Workspace 关键错误

use super::{Rule, RuleCategory, RuleMetadata, RulePriority};
use crate::{Diagnostic, Severity};
use tree_sitter::Node;

// ==================== SA1xxx: 标准库 API 使用错误 ====================

/// SA1000: invalid-regex - 无效的正则表达式
pub struct InvalidRegex;

impl Rule for InvalidRegex {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "SA1000",
            name: "invalid-regex",
            description: "无效的正则表达式",
            category: RuleCategory::Logic,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "call_expression" {
            let call_text = &source[node.byte_range()];

            // 检查 regexp.Compile 或 regexp.MustCompile 调用
            if call_text.contains("regexp.")
                && (call_text.contains("Compile(") || call_text.contains("MustCompile("))
            {
                // 尝试提取正则表达式参数
                if let Some(arg) = extract_first_string_arg(&node, source) {
                    // 简单检查常见的正则错误
                    if has_regex_error(&arg) {
                        let pos = node.start_position();
                        diagnostics.push(Diagnostic {
                            code: "SA1000".to_string(),
                            message: format!("无效的正则表达式: {}", arg),
                            severity: self.default_severity(),
                            file_path: file_path.to_string(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                }
            }
        }

        diagnostics
    }
}

fn extract_first_string_arg(node: &Node, source: &str) -> Option<String> {
    if let Some(args) = node.child_by_field_name("arguments") {
        let mut cursor = args.walk();
        for child in args.children(&mut cursor) {
            if child.kind() == "interpreted_string_literal" || child.kind() == "raw_string_literal"
            {
                let text = &source[child.byte_range()];
                return Some(text.trim_matches('"').trim_matches('`').to_string());
            }
        }
    }
    None
}

fn has_regex_error(pattern: &str) -> bool {
    // 检查常见的正则错误
    let mut depth = 0;
    let mut in_class = false;
    let chars: Vec<char> = pattern.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        match c {
            '(' if !in_class => depth += 1,
            ')' if !in_class => {
                depth -= 1;
                if depth < 0 {
                    return true; // 未匹配的右括号
                }
            }
            '[' if i == 0 || chars[i - 1] != '\\' => in_class = true,
            ']' => in_class = false,
            // 检查量词语法 {n,m}
            '{' if !in_class && i + 1 < chars.len() => {
                let rest: String = chars[i + 1..].iter().collect();
                if !rest.contains('}') {
                    return true;
                }
            }
            _ => {}
        }
    }

    depth != 0 || in_class
}

/// SA1001: invalid-template - 无效的模板
pub struct InvalidTemplate;

impl Rule for InvalidTemplate {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "SA1001",
            name: "invalid-template",
            description: "无效的 text/template 或 html/template",
            category: RuleCategory::Logic,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "call_expression" {
            let call_text = &source[node.byte_range()];

            if call_text.contains("template.")
                && (call_text.contains("Parse(") || call_text.contains("ParseFiles("))
            {
                if let Some(arg) = extract_first_string_arg(&node, source) {
                    // 检查模板语法错误
                    if has_template_error(&arg) {
                        let pos = node.start_position();
                        diagnostics.push(Diagnostic {
                            code: "SA1001".to_string(),
                            message: "模板语法错误".to_string(),
                            severity: self.default_severity(),
                            file_path: file_path.to_string(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                }
            }
        }

        diagnostics
    }
}

fn has_template_error(template: &str) -> bool {
    // 检查常见的模板错误
    let mut depth = 0;
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            if chars.peek() == Some(&'{') {
                chars.next(); // consume second {
                depth += 1;
            }
        } else if c == '}' && chars.peek() == Some(&'}') {
            chars.next(); // consume second }
            depth -= 1;
            if depth < 0 {
                return true;
            }
        }
    }

    depth != 0
}

/// SA1006: printf-with-dynamic-format - 使用动态格式字符串
pub struct PrintfDynamicFormat;

impl Rule for PrintfDynamicFormat {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "SA1006",
            name: "printf-dynamic-format",
            description: "Printf 使用动态格式字符串且没有其他参数",
            category: RuleCategory::Logic,
            priority: RulePriority::Required,
            default_severity: Severity::Warning,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "call_expression" {
            let call_text = &source[node.byte_range()];

            // 检查 fmt.Printf 或 fmt.Sprintf 等
            if call_text.contains("fmt.")
                && (call_text.contains("Printf(") || call_text.contains("Sprintf("))
            {
                // 检查参数数量
                if let Some(args) = node.child_by_field_name("arguments") {
                    let arg_count = count_arguments(&args);

                    if arg_count == 1 {
                        // 只有一个参数（格式字符串）
                        // 检查第一个参数是否是变量而非字符串字面量
                        if let Some(first_arg) = get_first_argument(&args) {
                            if first_arg.kind() != "interpreted_string_literal"
                                && first_arg.kind() != "raw_string_literal"
                            {
                                let pos = node.start_position();
                                diagnostics.push(Diagnostic {
                                    code: "SA1006".to_string(),
                                    message: "Printf 使用动态格式字符串，可能引入安全漏洞"
                                        .to_string(),
                                    severity: self.default_severity(),
                                    file_path: file_path.to_string(),
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

fn count_arguments(args: &Node) -> usize {
    let mut count = 0;
    let mut cursor = args.walk();
    for child in args.children(&mut cursor) {
        if !matches!(child.kind(), "(" | ")" | ",") {
            count += 1;
        }
    }
    count
}

fn get_first_argument<'a>(args: &'a Node<'a>) -> Option<Node<'a>> {
    let mut cursor = args.walk();
    for child in args.children(&mut cursor) {
        if !matches!(child.kind(), "(" | ")" | ",") {
            return Some(child);
        }
    }
    None
}

/// SA1012: nil-context - 传递 nil context
pub struct NilContext;

impl Rule for NilContext {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "SA1012",
            name: "nil-context",
            description: "向函数传递 nil context",
            category: RuleCategory::Logic,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "call_expression" {
            if let Some(args) = node.child_by_field_name("arguments") {
                let args_text = &source[args.byte_range()];

                // 检查是否直接传递 nil 作为 context
                if args_text.contains("nil") {
                    // 检查函数签名是否包含 context.Context 类型
                    let func_node = node.child_by_field_name("function");
                    if let Some(func) = func_node {
                        let func_name = &source[func.byte_range()];

                        // 常见需要 context 的函数模式
                        if func_name.contains("WithContext")
                            || (func_name.contains(".") && args_text.contains("nil"))
                        {
                            let pos = args.start_position();
                            diagnostics.push(Diagnostic {
                                code: "SA1012".to_string(),
                                message: "不应传递 nil context，使用 context.Background() 或 context.TODO()".to_string(),
                                severity: self.default_severity(),
                                file_path: file_path.to_string(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

/// SA1019: deprecated-function - 使用已弃用的函数
pub struct DeprecatedFunction;

impl Rule for DeprecatedFunction {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "SA1019",
            name: "deprecated-function",
            description: "使用了已弃用的函数、变量或字段",
            category: RuleCategory::Logic,
            priority: RulePriority::Required,
            default_severity: Severity::Warning,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "call_expression" || node.kind() == "selector_expression" {
            let text = &source[node.byte_range()];

            // 检查已弃用的函数
            let deprecated_items = [
                ("ioutil.ReadFile", "os.ReadFile"),
                ("ioutil.WriteFile", "os.WriteFile"),
                ("ioutil.ReadAll", "io.ReadAll"),
                ("ioutil.NopCloser", "io.NopCloser"),
                ("ioutil.TempDir", "os.MkdirTemp"),
                ("ioutil.TempFile", "os.CreateTemp"),
                ("math/rand.Seed", ""),
                ("math/rand.Read", "crypto/rand.Read"),
            ];

            for (deprecated, replacement) in &deprecated_items {
                if text.contains(deprecated) {
                    let msg = if replacement.is_empty() {
                        format!("{} 已弃用", deprecated)
                    } else {
                        format!("{} 已弃用，请使用 {}", deprecated, replacement)
                    };

                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        code: "SA1019".to_string(),
                        message: msg,
                        severity: self.default_severity(),
                        file_path: file_path.to_string(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}

// ==================== SA2xxx: 并发问题 ====================

/// SA2000: sync-waitgroup-add-goroutine - WaitGroup.Add 在 goroutine 中调用
pub struct SyncWaitgroupAddGoroutine;

impl Rule for SyncWaitgroupAddGoroutine {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "SA2000",
            name: "sync-waitgroup-add-goroutine",
            description: "sync.WaitGroup.Add 在 goroutine 中调用，导致竞态条件",
            category: RuleCategory::Concurrency,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // 检查 go 语句中的 WaitGroup.Add
        if node.kind() == "go_statement" {
            let go_text = &source[node.byte_range()];

            if go_text.contains("Add(") && go_text.contains("WaitGroup") {
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    code: "SA2000".to_string(),
                    message: "WaitGroup.Add 不应在 goroutine 中调用，应在启动 goroutine 之前调用"
                        .to_string(),
                    severity: self.default_severity(),
                    file_path: file_path.to_string(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }

        diagnostics
    }
}

/// SA2001: empty-critical-section - 空的临界区
pub struct EmptyCriticalSection;

impl Rule for EmptyCriticalSection {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "SA2001",
            name: "empty-critical-section",
            description: "空的临界区，可能遗漏了 defer Unlock",
            category: RuleCategory::Concurrency,
            priority: RulePriority::Required,
            default_severity: Severity::Warning,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // 检查 Lock() 后面直接 Unlock() 没有实际代码
        if node.kind() == "expression_statement" {
            let text = &source[node.byte_range()];

            if text.contains("Lock()") {
                // 检查下一个兄弟节点
                if let Some(parent) = node.parent() {
                    let mut found_unlock = false;
                    let mut cursor = parent.walk();
                    let mut after_lock = false;

                    for child in parent.children(&mut cursor) {
                        if after_lock && !found_unlock {
                            let child_text = &source[child.byte_range()];
                            if child_text.contains("Unlock()") {
                                found_unlock = true;
                                let pos = node.start_position();
                                diagnostics.push(Diagnostic {
                                    code: "SA2001".to_string(),
                                    message: "空的临界区，可能意图使用 defer Unlock()".to_string(),
                                    severity: self.default_severity(),
                                    file_path: file_path.to_string(),
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                            } else if !child.kind().contains("comment")
                                && child.kind() != "{"
                                && child.kind() != "}"
                            {
                                break; // 中间有其他代码
                            }
                        }

                        if child.id() == node.id() {
                            after_lock = true;
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

/// SA2002: t-failnow-goroutine - 在 goroutine 中调用 t.FailNow
pub struct TestFailNowGoroutine;

impl Rule for TestFailNowGoroutine {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "SA2002",
            name: "t-failnow-goroutine",
            description: "在 goroutine 中调用 t.FailNow 或 t.SkipNow 不被允许",
            category: RuleCategory::Concurrency,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "go_statement" {
            let go_text = &source[node.byte_range()];

            if go_text.contains("FailNow()") || go_text.contains("SkipNow()") {
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    code: "SA2002".to_string(),
                    message: "不应在 goroutine 中调用 t.FailNow 或 t.SkipNow".to_string(),
                    severity: self.default_severity(),
                    file_path: file_path.to_string(),
                    line: pos.row + 1,
                    column: pos.column + 1,
                    fix: None,
                });
            }
        }

        diagnostics
    }
}

// ==================== SA5xxx: 运行时问题 ====================

/// SA5000: assignment-to-nil-map - 向 nil map 赋值
pub struct AssignmentToNilMap;

impl Rule for AssignmentToNilMap {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "SA5000",
            name: "assignment-to-nil-map",
            description: "向 nil map 赋值会导致 panic",
            category: RuleCategory::Logic,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "assignment_statement" || node.kind() == "short_var_declaration" {
            let text = &source[node.byte_range()];

            // 检查是否是 map[key] = value 模式
            if text.contains('[') && text.contains("]=") || text.contains("] =") {
                // 这里需要更复杂的分析来确定 map 是否为 nil
                // 简化版：检查是否有明显的 nil 声明
                if text.contains("= nil") || text.contains(":= nil") {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        code: "SA5000".to_string(),
                        message: "可能向 nil map 赋值，使用 make() 初始化 map".to_string(),
                        severity: self.default_severity(),
                        file_path: file_path.to_string(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}

/// SA5001: defer-close-before-check - 在检查错误前 defer Close
pub struct DeferCloseBeforeCheck;

impl Rule for DeferCloseBeforeCheck {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "SA5001",
            name: "defer-close-before-check",
            description: "在检查错误前 defer Close 可能导致 panic",
            category: RuleCategory::Logic,
            priority: RulePriority::Required,
            default_severity: Severity::Warning,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "defer_statement" {
            let defer_text = &source[node.byte_range()];

            if defer_text.contains("Close()") {
                // 检查前一个语句是否是错误检查
                if let Some(parent) = node.parent() {
                    let parent_text = &source[parent.byte_range()];

                    // 简单检查：defer Close 前没有 if err != nil
                    if parent_text.contains("os.Open(")
                        || parent_text.contains("os.Create(")
                        || parent_text.contains("net")
                    {
                        // 检查是否检查了错误
                        if !parent_text.contains("if err != nil")
                            && !parent_text.contains("if err==nil")
                        {
                            let pos = node.start_position();
                            diagnostics.push(Diagnostic {
                                code: "SA5001".to_string(),
                                message: "应在检查错误后再 defer Close()".to_string(),
                                severity: self.default_severity(),
                                file_path: file_path.to_string(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

/// SA5007: infinite-recursion - 无限递归
pub struct InfiniteRecursion;

impl Rule for InfiniteRecursion {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "SA5007",
            name: "infinite-recursion",
            description: "函数无条件地递归调用自身",
            category: RuleCategory::Logic,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "function_declaration" {
            if let Some(body) = node.child_by_field_name("body") {
                if let Some(name) = node.child_by_field_name("name") {
                    let func_name = &source[name.byte_range()];
                    let body_text = &source[body.byte_range()];

                    // 检查函数体是否直接调用自身（无条件递归）
                    // 简化检查：函数体以函数名开头，后面跟着括号
                    let lines: Vec<&str> = body_text.lines().collect();
                    if lines.len() >= 2 {
                        let first_stmt = lines[1].trim(); // 跳过 {

                        if first_stmt.starts_with(func_name)
                            && first_stmt.contains("(")
                            && !first_stmt.contains("if ")
                            && !first_stmt.contains("for ")
                        {
                            // 检查是否有返回语句或其他终止条件
                            if !body_text.contains("return")
                                || body_text.matches(func_name).count()
                                    == body_text.matches("return").count()
                            {
                                let pos = node.start_position();
                                diagnostics.push(Diagnostic {
                                    code: "SA5007".to_string(),
                                    message: format!("函数 '{}' 可能无限递归", func_name),
                                    severity: self.default_severity(),
                                    file_path: file_path.to_string(),
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

/// SA5008: invalid-struct-tag - 无效的 struct tag
pub struct InvalidStructTag;

impl Rule for InvalidStructTag {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "SA5008",
            name: "invalid-struct-tag",
            description: "无效的 struct tag 格式",
            category: RuleCategory::Logic,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "field_declaration" {
            let field_text = &source[node.byte_range()];

            // 检查 struct tag
            if field_text.contains('`') {
                // 提取 tag 部分
                if let Some(tag_start) = field_text.find('`') {
                    if let Some(tag_end) = field_text[tag_start + 1..].find('`') {
                        let tag = &field_text[tag_start + 1..tag_start + 1 + tag_end];

                        // 检查常见错误
                        if has_struct_tag_error(tag) {
                            let pos = node.start_position();
                            diagnostics.push(Diagnostic {
                                code: "SA5008".to_string(),
                                message: format!("无效的 struct tag: {}", tag),
                                severity: self.default_severity(),
                                file_path: file_path.to_string(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

fn has_struct_tag_error(tag: &str) -> bool {
    // 检查常见的 struct tag 错误
    // 例如：重复的 key，格式错误等
    let keys: Vec<&str> = tag
        .split(' ')
        .map(|s| s.split(':').next().unwrap_or(""))
        .collect();

    // 检查重复 key
    for (i, key) in keys.iter().enumerate() {
        if keys.iter().skip(i + 1).any(|k| k == key) {
            return true;
        }
    }

    // 检查格式：key:"value" 格式
    for part in tag.split(' ') {
        if !part.is_empty() && !part.contains(':') {
            return true;
        }
        if part.contains(':') && !part.contains('"') {
            return true;
        }
    }

    false
}

/// SA5011: possible-nil-pointer-dereference - 可能的 nil 指针解引用
pub struct PossibleNilPointerDereference;

impl Rule for PossibleNilPointerDereference {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "SA5011",
            name: "possible-nil-pointer-dereference",
            description: "可能的 nil 指针解引用",
            category: RuleCategory::Logic,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // 检查 selector_expression (如 x.Field)
        if node.kind() == "selector_expression" {
            let text = &source[node.byte_range()];

            // 简单检查：如果之前有对该变量的 nil 检查但没有返回
            if let Some(parent) = node.parent().and_then(|p| p.parent()) {
                let block_text = &source[parent.byte_range()];

                // 提取变量名
                if let Some(dot_pos) = text.find('.') {
                    let var_name = &text[..dot_pos];

                    // 检查是否有 nil 检查后继续使用
                    if block_text.contains(&format!("{} == nil", var_name))
                        || block_text.contains(&format!("{}==nil", var_name))
                    {
                        // 检查 nil 检查后是否有返回
                        let lines: Vec<&str> = block_text.lines().collect();
                        let mut found_nil_check = false;
                        let mut found_deref = false;

                        for line in lines {
                            if line.contains(&format!("{} == nil", var_name))
                                || line.contains(&format!("{}==nil", var_name))
                            {
                                found_nil_check = true;
                            }

                            if found_nil_check && line.contains(text) {
                                found_deref = true;
                            }

                            if found_nil_check
                                && (line.contains("return") || line.contains("panic("))
                            {
                                found_nil_check = false; // 安全，因为有返回
                            }
                        }

                        if found_deref {
                            let pos = node.start_position();
                            diagnostics.push(Diagnostic {
                                code: "SA5011".to_string(),
                                message: format!("可能的 nil 指针解引用: '{}'", var_name),
                                severity: self.default_severity(),
                                file_path: file_path.to_string(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

// ==================== 并发相关规则 ====================

/// BadLock: 锁的常见误用
pub struct BadLock;

impl Rule for BadLock {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "C001",
            name: "bad-lock",
            description: "锁的常见误用，如拷贝 Mutex、未配对 Unlock 等",
            category: RuleCategory::Concurrency,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // 检查 sync.Mutex 作为值传递
        if node.kind() == "parameter_declaration" || node.kind() == "field_declaration" {
            let text = &source[node.byte_range()];

            if text.contains("sync.Mutex") || text.contains("sync.RWMutex") {
                // 检查是否是值传递（不是指针）
                if !text.contains('*') && text.contains("func") {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        code: "C001".to_string(),
                        message: "sync.Mutex 应使用指针传递，避免拷贝".to_string(),
                        severity: self.default_severity(),
                        file_path: file_path.to_string(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        // 检查 Lock 后没有 Unlock
        if node.kind() == "function_declaration" {
            if let Some(body) = node.child_by_field_name("body") {
                let body_text = &source[body.byte_range()];

                let lock_count = body_text.matches(".Lock()").count();
                let unlock_count = body_text.matches(".Unlock()").count();
                let defer_unlock_count = body_text
                    .matches("defer")
                    .filter(|_| body_text.contains("Unlock()"))
                    .count();

                if lock_count > unlock_count + defer_unlock_count {
                    let pos = body.start_position();
                    diagnostics.push(Diagnostic {
                        code: "C001".to_string(),
                        message: "Lock 和 Unlock 不匹配，可能缺少 Unlock".to_string(),
                        severity: self.default_severity(),
                        file_path: file_path.to_string(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}

// ==================== 泛型规则 ====================

/// GEN002: type-parameter-shadow - 类型参数遮蔽外部类型
pub struct TypeParameterShadow;

impl Rule for TypeParameterShadow {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "GEN002",
            name: "type-parameter-shadow",
            description: "类型参数名遮蔽了外部作用域的类型",
            category: RuleCategory::Upgrade,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "type_parameter_declaration" {
            // 提取类型参数名
            if let Some(name_node) = node.child_by_field_name("name") {
                let param_name = &source[name_node.byte_range()];

                // 检查是否是常见类型名
                let common_types = [
                    "int",
                    "string",
                    "bool",
                    "float64",
                    "error",
                    "byte",
                    "rune",
                    "any",
                    "interface",
                ];

                if common_types.contains(&param_name) {
                    let pos = name_node.start_position();
                    diagnostics.push(Diagnostic {
                        code: "GEN002".to_string(),
                        message: format!("类型参数 '{}' 遮蔽了内置类型", param_name),
                        severity: self.default_severity(),
                        file_path: file_path.to_string(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}

/// GEN004: comparable-misuse - comparable 约束的误用
pub struct ComparableMisuse;

impl Rule for ComparableMisuse {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "GEN004",
            name: "comparable-misuse",
            description: "comparable 约束使用不当",
            category: RuleCategory::Upgrade,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "type_constraint" || node.kind() == "type_parameter_list" {
            let text = &source[node.byte_range()];

            // 检查 comparable 约束在切片或 map 上
            if text.contains("comparable") {
                // 检查函数体内是否对切片/map 使用 ==
                if let Some(parent_func) = find_parent_function(&node) {
                    let func_text = &source[parent_func.byte_range()];

                    // 检查是否比较了切片
                    if func_text.contains("[]") && func_text.contains(" == ") {
                        let pos = node.start_position();
                        diagnostics.push(Diagnostic {
                            code: "GEN004".to_string(),
                            message: "comparable 约束不能用于切片比较".to_string(),
                            severity: self.default_severity(),
                            file_path: file_path.to_string(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                }
            }
        }

        diagnostics
    }
}

fn find_parent_function<'a>(node: &'a Node<'a>) -> Option<Node<'a>> {
    let mut current = *node;

    while let Some(parent) = current.parent() {
        if parent.kind() == "function_declaration" || parent.kind() == "method_declaration" {
            return Some(parent);
        }
        current = parent;
    }

    None
}

/// GEN007: type-inference-fail - 无法推断类型参数
pub struct TypeInferenceFail;

impl Rule for TypeInferenceFail {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "GEN007",
            name: "type-inference-fail",
            description: "泛型函数调用时类型参数推断失败",
            category: RuleCategory::Upgrade,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "call_expression" {
            let text = &source[node.byte_range()];

            // 检查是否显式指定了类型参数（这表明可能需要推断）
            if text.contains("[") {
                // 简单检查：调用使用了类型参数列表 [T]
                if let Some(args) = node.child_by_field_name("arguments") {
                    let args_text = &source[args.byte_range()];

                    // 检查参数是否为空
                    if args_text.trim() == "()" || args_text.trim() == "( )" {
                        let pos = node.start_position();
                        diagnostics.push(Diagnostic {
                            code: "GEN007".to_string(),
                            message: "泛型函数调用缺少类型参数，类型推断可能失败".to_string(),
                            severity: self.default_severity(),
                            file_path: file_path.to_string(),
                            line: pos.row + 1,
                            column: pos.column + 1,
                            fix: None,
                        });
                    }
                }
            }
        }

        diagnostics
    }
}

// ==================== Fuzzing 规则 ====================

/// FUZZ001: fuzz-test-signature - Fuzzing 测试函数签名错误
pub struct FuzzTestSignature;

impl Rule for FuzzTestSignature {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "FUZZ001",
            name: "fuzz-test-signature",
            description: "Fuzzing 测试函数签名错误",
            category: RuleCategory::Docs,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "function_declaration" {
            if let Some(name) = node.child_by_field_name("name") {
                let func_name = &source[name.byte_range()];

                if func_name.starts_with("Fuzz") {
                    // 检查参数
                    if let Some(params) = node.child_by_field_name("parameters") {
                        let params_text = &source[params.byte_range()];

                        // Fuzz 函数必须接受 *testing.F
                        if !params_text.contains("*testing.F") && !params_text.contains("testing.F")
                        {
                            let pos = params.start_position();
                            diagnostics.push(Diagnostic {
                                code: "FUZZ001".to_string(),
                                message: "Fuzzing 测试函数必须接受 *testing.F 参数".to_string(),
                                severity: self.default_severity(),
                                file_path: file_path.to_string(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

/// FUZZ011: fuzz-global-state - Fuzzing 使用全局状态
pub struct FuzzGlobalState;

impl Rule for FuzzGlobalState {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "FUZZ011",
            name: "fuzz-global-state",
            description: "Fuzzing 测试使用了全局状态",
            category: RuleCategory::Docs,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "function_declaration" {
            if let Some(name) = node.child_by_field_name("name") {
                let func_name = &source[name.byte_range()];

                if func_name.starts_with("Fuzz") {
                    if let Some(body) = node.child_by_field_name("body") {
                        let body_text = &source[body.byte_range()];

                        // 检查是否访问了包级变量
                        // 简化检查：检查是否对包级变量赋值
                        if body_text.contains(" = ")
                            || body_text.contains("++")
                            || body_text.contains("--")
                        {
                            // 检查是否使用了 f.Fuzz 回调
                            if body_text.contains("f.Fuzz(") || body_text.contains("f.Add(") {
                                // 进一步的复杂分析... 简化版
                                let lines: Vec<&str> = body_text.lines().collect();
                                for line in lines {
                                    let trimmed = line.trim();
                                    // 检查是否修改了全局变量
                                    if (trimmed.contains(" = ")
                                        || trimmed.ends_with("++")
                                        || trimmed.ends_with("--"))
                                        && !trimmed.starts_with(":=")
                                        && !trimmed.starts_with("var ")
                                    {
                                        let pos = body.start_position();
                                        diagnostics.push(Diagnostic {
                                            code: "FUZZ011".to_string(),
                                            message: "Fuzzing 测试不应修改全局状态".to_string(),
                                            severity: self.default_severity(),
                                            file_path: file_path.to_string(),
                                            line: pos.row + 1,
                                            column: pos.column + 1,
                                            fix: None,
                                        });
                                        break; // 只报告一次
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

/// FUZZ012: fuzz-non-deterministic - Fuzzing 非确定性行为
pub struct FuzzNonDeterministic;

impl Rule for FuzzNonDeterministic {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "FUZZ012",
            name: "fuzz-non-deterministic",
            description: "Fuzzing 测试包含非确定性行为",
            category: RuleCategory::Docs,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "function_declaration" {
            if let Some(name) = node.child_by_field_name("name") {
                let func_name = &source[name.byte_range()];

                if func_name.starts_with("Fuzz") {
                    if let Some(body) = node.child_by_field_name("body") {
                        let body_text = &source[body.byte_range()];

                        // 检查非确定性来源
                        let non_deterministic = [
                            "time.Now()",
                            "rand.Int()",
                            "rand.Float64()",
                            "math/rand",
                            "os.Getenv",
                            "uuid.New()",
                        ];

                        for pattern in &non_deterministic {
                            if body_text.contains(pattern) {
                                let pos = body.start_position();
                                diagnostics.push(Diagnostic {
                                    code: "FUZZ012".to_string(),
                                    message: format!("Fuzzing 测试包含非确定性行为: {}", pattern),
                                    severity: self.default_severity(),
                                    file_path: file_path.to_string(),
                                    line: pos.row + 1,
                                    column: pos.column + 1,
                                    fix: None,
                                });
                                break; // 只报告一次
                            }
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

// ==================== Workspace 规则 ====================

/// WS001: workspace-go-version - Workspace Go 版本不一致
pub struct WorkspaceGoVersion;

impl Rule for WorkspaceGoVersion {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "WS001",
            name: "workspace-go-version",
            description: "Workspace 中各模块的 Go 版本不一致",
            category: RuleCategory::Upgrade,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, _node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // 检查 go.work 文件
        if file_path.ends_with("go.work") {
            // 提取 go 版本
            let go_version = extract_go_version(source);

            // 这是一个简化实现
            // 完整实现需要解析所有引用模块的 go.mod 文件
            if let Some(version) = go_version {
                if version.starts_with("1.17") || version.starts_with("1.16") {
                    diagnostics.push(Diagnostic {
                        code: "WS001".to_string(),
                        message: format!("go.work 使用较旧的 Go 版本: {}", version),
                        severity: self.default_severity(),
                        file_path: file_path.to_string(),
                        line: 1,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}

fn extract_go_version(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(version) = trimmed.strip_prefix("go ") {
            return Some(version.trim().to_string());
        }
    }
    None
}

/// WS002: workspace-module-path - Workspace 模块路径冲突
pub struct WorkspaceModulePath;

impl Rule for WorkspaceModulePath {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "WS002",
            name: "workspace-module-path",
            description: "Workspace 中的模块路径冲突",
            category: RuleCategory::Upgrade,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, _node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if file_path.ends_with("go.mod") {
            // 提取模块路径
            if let Some(module_path) = extract_module_path(source) {
                // 检查路径是否有效
                if module_path.is_empty() || module_path == "." {
                    diagnostics.push(Diagnostic {
                        code: "WS002".to_string(),
                        message: "无效的模块路径".to_string(),
                        severity: self.default_severity(),
                        file_path: file_path.to_string(),
                        line: 1,
                        column: 1,
                        fix: None,
                    });
                }

                // 检查是否包含大写字母（Go 模块路径规范）
                if module_path.chars().any(|c| c.is_uppercase()) {
                    diagnostics.push(Diagnostic {
                        code: "WS002".to_string(),
                        message: "模块路径不应包含大写字母".to_string(),
                        severity: self.default_severity(),
                        file_path: file_path.to_string(),
                        line: 1,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}

fn extract_module_path(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(path) = trimmed.strip_prefix("module ") {
            return Some(path.trim().to_string());
        }
    }
    None
}

/// WS103: workspace-dep-cycle - Workspace 模块间循环依赖
pub struct WorkspaceDepCycle;

impl Rule for WorkspaceDepCycle {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "WS103",
            name: "workspace-dep-cycle",
            description: "Workspace 模块间存在循环依赖",
            category: RuleCategory::Upgrade,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, _node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if file_path.ends_with("go.mod") {
            // 这是一个简化检查
            // 完整实现需要分析整个 Workspace 的依赖图

            // 检查是否有 replace 指令指向自己
            if source.contains("replace ") {
                if let Some(module_path) = extract_module_path(source) {
                    if source.contains(&format!("replace {}", module_path)) {
                        diagnostics.push(Diagnostic {
                            code: "WS103".to_string(),
                            message: "模块不应 replace 自己".to_string(),
                            severity: self.default_severity(),
                            file_path: file_path.to_string(),
                            line: 1,
                            column: 1,
                            fix: None,
                        });
                    }
                }
            }
        }

        diagnostics
    }
}

/// WS105: workspace-dep-security - Workspace 依赖存在安全漏洞
pub struct WorkspaceDepSecurity;

impl Rule for WorkspaceDepSecurity {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "WS105",
            name: "workspace-dep-security",
            description: "Workspace 依赖存在已知安全漏洞",
            category: RuleCategory::Upgrade,
            priority: RulePriority::Required,
            default_severity: Severity::Error,
            min_go_version: None,
        }
    }

    fn check(&self, _node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if file_path.ends_with("go.mod") {
            // 检查已知有漏洞的依赖版本
            let vulnerable_deps = [
                (
                    "golang.org/x/crypto",
                    "0.0.0-20200220183623-bac4c82f6975",
                    "CVE-2020-9283",
                ),
                ("github.com/gin-gonic/gin", "1.6.2", "CVE-2020-28483"),
                ("github.com/dgrijalva/jwt-go", "3.2.0", "CVE-2020-26160"),
            ];

            for (dep, bad_version, cve) in &vulnerable_deps {
                if source.contains(dep) && source.contains(bad_version) {
                    diagnostics.push(Diagnostic {
                        code: "WS105".to_string(),
                        message: format!("依赖 {} 版本 {} 存在安全漏洞 {}", dep, bad_version, cve),
                        severity: self.default_severity(),
                        file_path: file_path.to_string(),
                        line: 1,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}

// ==================== 规则导出 ====================

pub fn get_p0_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // SA1xxx: 标准库 API 错误
        Box::new(InvalidRegex),
        Box::new(InvalidTemplate),
        Box::new(PrintfDynamicFormat),
        Box::new(NilContext),
        Box::new(DeprecatedFunction),
        // SA2xxx: 并发问题
        Box::new(SyncWaitgroupAddGoroutine),
        Box::new(EmptyCriticalSection),
        Box::new(TestFailNowGoroutine),
        // SA5xxx: 运行时问题
        Box::new(AssignmentToNilMap),
        Box::new(DeferCloseBeforeCheck),
        Box::new(InfiniteRecursion),
        Box::new(InvalidStructTag),
        Box::new(PossibleNilPointerDereference),
        // 并发相关
        Box::new(BadLock),
        // 泛型规则
        Box::new(TypeParameterShadow),
        Box::new(ComparableMisuse),
        Box::new(TypeInferenceFail),
        // Fuzzing 规则
        Box::new(FuzzTestSignature),
        Box::new(FuzzGlobalState),
        Box::new(FuzzNonDeterministic),
        // Workspace 规则
        Box::new(WorkspaceGoVersion),
        Box::new(WorkspaceModulePath),
        Box::new(WorkspaceDepCycle),
        Box::new(WorkspaceDepSecurity),
    ]
}
