//! E-series: gocodestyle - 代码风格与语法错误
//!
//! 包括：布局格式、导入、空白、代码结构、比较判断等

use super::{Rule, RuleCategory, RuleMetadata, RulePriority};
use crate::{Diagnostic, Severity};
use tree_sitter::Node;

// ==================== E1xx: 布局与格式 ====================

/// E101: mixed-indentation - 混合使用空格和 Tab
pub struct MixedIndentation;

impl Rule for MixedIndentation {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "E101",
            name: "mixed-indentation",
            description: "混合使用空格和 Tab 缩进",
            category: RuleCategory::Codestyle,
            priority: RulePriority::Required,
            default_severity: Severity::Warning,
            min_go_version: None,
        }
    }

    fn check(&self, _node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // 检查所有行是否同时包含空格和 Tab
        for (line_num, line) in source.lines().enumerate() {
            let has_space = line.starts_with(' ');
            let has_tab = line.starts_with('\t');

            if has_space && has_tab {
                diagnostics.push(Diagnostic {
                    code: "E101".to_string(),
                    message: "混合使用空格和 Tab 缩进".to_string(),
                    severity: self.default_severity(),
                    file_path: file_path.to_string(),
                    line: line_num + 1,
                    column: 1,
                    fix: None,
                });
            }
        }

        diagnostics
    }
}

/// E115: trailing-whitespace - 行尾空白字符
pub struct TrailingWhitespace;

impl Rule for TrailingWhitespace {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "E115",
            name: "trailing-whitespace",
            description: "行尾存在空白字符",
            category: RuleCategory::Codestyle,
            priority: RulePriority::Required,
            default_severity: Severity::Info,
            min_go_version: None,
        }
    }

    fn check(&self, _node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // Walk lines with byte offsets so we can emit precise safe fixes.
        let mut offset = 0usize;
        for (line_num, line) in source.split('\n').enumerate() {
            let line_start = offset;
            offset += line.len() + 1; // +1 for the '\n' (last line has none)

            // Trailing whitespace: spaces/tabs (and a stray \r on CRLF files).
            let trimmed = line.trim_end_matches([' ', '\t', '\r']);
            if trimmed.len() < line.len() {
                let fix_start = line_start + trimmed.len();
                let fix_end = line_start + line.len();
                diagnostics.push(Diagnostic {
                    code: "E115".to_string(),
                    message: "行尾存在空白字符".to_string(),
                    severity: self.default_severity(),
                    file_path: file_path.to_string(),
                    line: line_num + 1,
                    column: line.len(),
                    fix: Some(crate::Fix::safe(
                        "Remove trailing whitespace",
                        "",
                        fix_start,
                        fix_end,
                    )),
                });
            }
        }

        diagnostics
    }
}

/// E116: multiple-trailing-newlines - 文件末尾多余空行
pub struct MultipleTrailingNewlines;

impl Rule for MultipleTrailingNewlines {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "E116",
            name: "multiple-trailing-newlines",
            description: "文件末尾存在多个空行",
            category: RuleCategory::Codestyle,
            priority: RulePriority::Required,
            default_severity: Severity::Info,
            min_go_version: None,
        }
    }

    fn check(&self, _node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // 检查末尾是否有超过两个换行
        let trailing_newlines = source.chars().rev().take_while(|&c| c == '\n').count();

        if trailing_newlines > 2 {
            let lines: Vec<&str> = source.lines().collect();
            let last_line = lines.len();

            diagnostics.push(Diagnostic {
                code: "E116".to_string(),
                message: format!(
                    "文件末尾存在 {} 个空行，建议保留最多 1 个",
                    trailing_newlines - 1
                ),
                severity: self.default_severity(),
                file_path: file_path.to_string(),
                line: last_line,
                column: 1,
                fix: None,
            });
        }

        diagnostics
    }
}

/// E117: no-newline-at-end - 文件末尾缺少换行
pub struct NoNewlineAtEnd;

impl Rule for NoNewlineAtEnd {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "E117",
            name: "no-newline-at-end",
            description: "文件末尾缺少换行符",
            category: RuleCategory::Codestyle,
            priority: RulePriority::Required,
            default_severity: Severity::Warning,
            min_go_version: None,
        }
    }

    fn check(&self, _node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if !source.is_empty() && !source.ends_with('\n') {
            let lines: Vec<&str> = source.lines().collect();
            let last_line = lines.len();
            let last_line_len = lines.last().map(|l| l.len()).unwrap_or(0);

            diagnostics.push(Diagnostic {
                code: "E117".to_string(),
                message: "文件末尾应有一个换行符".to_string(),
                severity: self.default_severity(),
                file_path: file_path.to_string(),
                line: last_line,
                column: last_line_len,
                fix: None,
            });
        }

        diagnostics
    }
}

/// E118: line-too-long - 行过长
pub struct LineTooLong {
    max_length: usize,
}

impl LineTooLong {
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }
}

impl Default for LineTooLong {
    fn default() -> Self {
        Self::new(120)
    }
}

impl Rule for LineTooLong {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "E118",
            name: "line-too-long",
            description: "行长度超过限制",
            category: RuleCategory::Codestyle,
            priority: RulePriority::Required,
            default_severity: Severity::Warning,
            min_go_version: None,
        }
    }

    fn check(&self, _node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        for (line_num, line) in source.lines().enumerate() {
            let line_length = line.chars().count();
            if line_length > self.max_length {
                diagnostics.push(Diagnostic {
                    code: "E118".to_string(),
                    message: format!("行长度 {} 超过 {} 字符限制", line_length, self.max_length),
                    severity: self.default_severity(),
                    file_path: file_path.to_string(),
                    line: line_num + 1,
                    column: self.max_length,
                    fix: None,
                });
            }
        }

        diagnostics
    }
}

// ==================== E2xx: 导入 ====================

/// E201: blank-line-after-import - 导入后缺少空行
pub struct BlankLineAfterImport;

impl Rule for BlankLineAfterImport {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "E201",
            name: "blank-line-after-import",
            description: "导入语句后应有一个空行",
            category: RuleCategory::Codestyle,
            priority: RulePriority::Required,
            default_severity: Severity::Info,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, _source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "import_declaration" {
            if let Some(next_sibling) = node.next_sibling() {
                // 检查是否有空行
                let import_end = node.end_position().row;
                let next_start = next_sibling.start_position().row;

                if next_start == import_end + 1 {
                    diagnostics.push(Diagnostic {
                        code: "E201".to_string(),
                        message: "import 语句后应有一个空行".to_string(),
                        severity: self.default_severity(),
                        file_path: file_path.to_string(),
                        line: import_end + 1,
                        column: 1,
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}

/// E202: import-not-at-top - 导入不在文件顶部
pub struct ImportNotAtTop;

impl Rule for ImportNotAtTop {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "E202",
            name: "import-not-at-top",
            description: "import 语句应在文件顶部（package 之后）",
            category: RuleCategory::Codestyle,
            priority: RulePriority::Required,
            default_severity: Severity::Warning,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, _source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "import_declaration" {
            // 检查是否有非 import 的兄弟节点在它之前
            if let Some(parent) = node.parent() {
                let mut found_non_import = false;
                let mut cursor = parent.walk();
                for child in parent.children(&mut cursor) {
                    if child.id() == node.id() {
                        break;
                    }
                    if child.kind() != "package_clause"
                        && child.kind() != "import_declaration"
                        && child.kind() != "comment"
                    {
                        found_non_import = true;
                        break;
                    }
                }

                if found_non_import {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        code: "E202".to_string(),
                        message: "import 语句应位于文件顶部".to_string(),
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

/// E203: duplicate-import - 重复导入
pub struct DuplicateImport;

impl Rule for DuplicateImport {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "E203",
            name: "duplicate-import",
            description: "重复导入同一个包",
            category: RuleCategory::Codestyle,
            priority: RulePriority::Required,
            default_severity: Severity::Warning,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "source_file" {
            use std::collections::HashSet;
            let mut seen: HashSet<String> = HashSet::new();

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "import_declaration" {
                    // 提取导入路径
                    if let Some(path_node) = child.child_by_field_name("path") {
                        let path_text = &source[path_node.byte_range()];
                        let clean_path = path_text.trim_matches('"').to_string();

                        if seen.contains(&clean_path) {
                            let pos = child.start_position();
                            diagnostics.push(Diagnostic {
                                code: "E203".to_string(),
                                message: format!("包 '{}' 被重复导入", clean_path),
                                severity: self.default_severity(),
                                file_path: file_path.to_string(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                        } else {
                            seen.insert(clean_path);
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

// ==================== E3xx: 空白与格式 ====================

/// E301: empty-block - 空代码块
pub struct EmptyBlock;

impl Rule for EmptyBlock {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "E301",
            name: "empty-block",
            description: "空代码块",
            category: RuleCategory::Codestyle,
            priority: RulePriority::Required,
            default_severity: Severity::Info,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, _source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        let block_kinds = [
            "block",
            "function_declaration",
            "if_statement",
            "for_statement",
            "range_clause",
        ];

        if block_kinds.contains(&node.kind()) {
            let mut has_content = false;
            let mut cursor = node.walk();

            for child in node.children(&mut cursor) {
                if !matches!(child.kind(), "{" | "}" | "comment") {
                    has_content = true;
                    break;
                }
            }

            if !has_content {
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    code: "E301".to_string(),
                    message: "空代码块".to_string(),
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

/// E302: missing-whitespace-around-operator - 运算符周围缺少空格
pub struct MissingWhitespaceAroundOperator;

impl Rule for MissingWhitespaceAroundOperator {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "E302",
            name: "missing-whitespace-around-operator",
            description: "二元运算符两侧应有空格",
            category: RuleCategory::Codestyle,
            priority: RulePriority::Required,
            default_severity: Severity::Info,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        let binary_ops = [
            "+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", "&&", "||", "&", "|", "^",
            "<<", ">>", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=",
        ];

        if binary_ops.contains(&node.kind()) {
            let pos = node.start_position();
            let byte_start = node.start_byte();
            let byte_end = node.end_byte();

            // 检查前后字符
            let before = if byte_start > 0 {
                source.get(byte_start - 1..byte_start)
            } else {
                None
            };
            let after = source.get(byte_end..byte_end + 1);

            if let Some(b) = before {
                if !b.chars().next().map(|c| c.is_whitespace()).unwrap_or(true) {
                    diagnostics.push(Diagnostic {
                        code: "E302".to_string(),
                        message: format!("运算符 '{}' 左侧应有一个空格", node.kind()),
                        severity: self.default_severity(),
                        file_path: file_path.to_string(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        fix: None,
                    });
                }
            }

            if let Some(a) = after {
                if !a.chars().next().map(|c| c.is_whitespace()).unwrap_or(true) {
                    diagnostics.push(Diagnostic {
                        code: "E302".to_string(),
                        message: format!("运算符 '{}' 右侧应有一个空格", node.kind()),
                        severity: self.default_severity(),
                        file_path: file_path.to_string(),
                        line: pos.row + 1,
                        column: pos.column + node.kind().len(),
                        fix: None,
                    });
                }
            }
        }

        diagnostics
    }
}

// ==================== E4xx: 代码结构 ====================

/// E401: multiple-statements - 多语句在同一行
pub struct MultipleStatements;

impl Rule for MultipleStatements {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "E401",
            name: "multiple-statements",
            description: "每行应只有一个语句",
            category: RuleCategory::Codestyle,
            priority: RulePriority::Required,
            default_severity: Severity::Warning,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, _source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // 检查是否在同一行有多个语句
        if node.kind() == "block" || node.kind() == "statement_list" {
            let mut last_line = 0;
            let mut cursor = node.walk();

            for child in node.children(&mut cursor) {
                if child.kind().ends_with("_statement") || child.kind().ends_with("_declaration") {
                    let line = child.start_position().row;
                    if line == last_line && last_line > 0 {
                        diagnostics.push(Diagnostic {
                            code: "E401".to_string(),
                            message: "每行应只有一个语句".to_string(),
                            severity: self.default_severity(),
                            file_path: file_path.to_string(),
                            line: line + 1,
                            column: child.start_position().column + 1,
                            fix: None,
                        });
                    }
                    last_line = line;
                }
            }
        }

        diagnostics
    }
}

// ==================== E7xx: 比较与判断 ====================

/// E712: comparison-to-true - 与 true 显式比较
pub struct ComparisonToTrue;

impl Rule for ComparisonToTrue {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "E712",
            name: "comparison-to-true",
            description: "不应与 true 显式比较",
            category: RuleCategory::Codestyle,
            priority: RulePriority::Required,
            default_severity: Severity::Info,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "binary_expression" {
            let node_text = &source[node.byte_range()];

            if node_text.contains(" == true") || node_text.contains("==true") {
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    code: "E712".to_string(),
                    message: "不应与 true 显式比较，直接使用布尔值即可".to_string(),
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

/// E713: comparison-to-false - 与 false 显式比较
pub struct ComparisonToFalse;

impl Rule for ComparisonToFalse {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "E713",
            name: "comparison-to-false",
            description: "不应与 false 显式比较",
            category: RuleCategory::Codestyle,
            priority: RulePriority::Required,
            default_severity: Severity::Info,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "binary_expression" {
            let node_text = &source[node.byte_range()];

            if node_text.contains(" == false") || node_text.contains("==false") {
                let pos = node.start_position();
                diagnostics.push(Diagnostic {
                    code: "E713".to_string(),
                    message: "不应与 false 显式比较，使用 ! 操作符即可".to_string(),
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

/// E721: type-comparison - 类型比较方式
pub struct TypeComparison;

impl Rule for TypeComparison {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "E721",
            name: "type-comparison",
            description: "类型比较应使用类型断言或反射",
            category: RuleCategory::Codestyle,
            priority: RulePriority::Required,
            default_severity: Severity::Warning,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // 检查 reflect.TypeOf(x) == reflect.TypeOf(y) 模式
        if node.kind() == "call_expression" {
            let node_text = source[node.byte_range()].to_lowercase();
            if node_text.contains("reflect.typeof") {
                // 检查是否在比较上下文中
                if let Some(parent) = node.parent() {
                    if parent.kind() == "binary_expression" {
                        let pos = node.start_position();
                        diagnostics.push(Diagnostic {
                            code: "E721".to_string(),
                            message: "类型比较方式可能不当，考虑使用类型断言".to_string(),
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

// ==================== 规则导出 ====================

pub fn get_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(MixedIndentation),
        Box::new(TrailingWhitespace),
        Box::new(MultipleTrailingNewlines),
        Box::new(NoNewlineAtEnd),
        Box::new(LineTooLong::default()),
        Box::new(BlankLineAfterImport),
        Box::new(ImportNotAtTop),
        Box::new(DuplicateImport),
        Box::new(EmptyBlock),
        Box::new(MissingWhitespaceAroundOperator),
        Box::new(MultipleStatements),
        Box::new(ComparisonToTrue),
        Box::new(ComparisonToFalse),
        Box::new(TypeComparison),
    ]
}
