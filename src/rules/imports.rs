//! I-series: goimports - 导入排序与分组检查
//!
//! 包括：导入排序、分组、别名使用等

use super::{Rule, RuleCategory, RuleMetadata, RulePriority};
use crate::{Diagnostic, Severity};
use tree_sitter::Node;

/// I001: unsorted-imports - 导入未按字母顺序排序
pub struct UnsortedImports;

impl Rule for UnsortedImports {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "I001",
            name: "unsorted-imports",
            description: "导入语句应按字母顺序排序",
            category: RuleCategory::Imports,
            priority: RulePriority::Required,
            default_severity: Severity::Info,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "import_declaration" {
            // 检查是否是 import 块
            let import_text = &source[node.byte_range()];

            if import_text.contains('\n') {
                // 多行 import 块
                let lines: Vec<&str> = import_text.lines().collect();
                let mut prev_import: Option<String> = None;

                for line in &lines {
                    let trimmed = line.trim();
                    if trimmed.starts_with('"') || trimmed.contains("\"") {
                        // 提取导入路径
                        if let Some(start) = trimmed.find('"') {
                            if let Some(end) = trimmed[start + 1..].find('"') {
                                let import_path = &trimmed[start + 1..start + 1 + end];

                                if let Some(ref prev) = prev_import {
                                    if import_path < prev.as_str() {
                                        diagnostics.push(Diagnostic {
                                            code: "I001".to_string(),
                                            message: format!(
                                                "导入 '{}' 应排在 '{}' 之前",
                                                import_path, prev
                                            ),
                                            severity: self.default_severity(),
                                            file_path: file_path.to_string(),
                                            line: node.start_position().row + 1,
                                            column: 1,
                                            fix: None,
                                        });
                                        break;
                                    }
                                }
                                prev_import = Some(import_path.to_string());
                            }
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

/// I002: import-groups-missing - 缺少导入分组
pub struct ImportGroupsMissing;

impl Rule for ImportGroupsMissing {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "I002",
            name: "import-groups-missing",
            description: "应按标准库/第三方/本地包分组",
            category: RuleCategory::Imports,
            priority: RulePriority::Required,
            default_severity: Severity::Info,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "source_file" {
            let mut imports_by_category: Vec<(String, ImportCategory)> = vec![];

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "import_declaration" {
                    if let Some(path_node) = child.child_by_field_name("path") {
                        let path = source[path_node.byte_range()].trim_matches('"');
                        let category = categorize_import(path);
                        imports_by_category.push((path.to_string(), category));
                    }
                }
            }

            // 检查是否混合了不同类别且没有空行分隔
            if imports_by_category.len() >= 3 {
                let mut prev_category: Option<ImportCategory> = None;
                let mut mixed_without_group = false;

                for (_path, category) in &imports_by_category {
                    if let Some(ref prev) = prev_category {
                        if *category != *prev && *category != ImportCategory::StandardLibrary {
                            // 从标准库到非标准库，应该有分组
                            mixed_without_group = true;
                        }
                    }
                    prev_category = Some(*category);
                }

                if mixed_without_group {
                    // 查找第一个 import_declaration 的位置
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if child.kind() == "import_declaration" {
                            let pos = child.start_position();
                            diagnostics.push(Diagnostic {
                                code: "I002".to_string(),
                                message: "导入应按标准库、第三方库、本地包分组".to_string(),
                                severity: self.default_severity(),
                                file_path: file_path.to_string(),
                                line: pos.row + 1,
                                column: pos.column + 1,
                                fix: None,
                            });
                            break;
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ImportCategory {
    StandardLibrary,
    ThirdParty,
    Local,
}

fn categorize_import(path: &str) -> ImportCategory {
    // 标准库列表（Go 1.20+）
    let stdlibs = [
        "archive",
        "bufio",
        "bytes",
        "cmp",
        "compress",
        "container",
        "context",
        "crypto",
        "database",
        "debug",
        "embed",
        "encoding",
        "errors",
        "expvar",
        "flag",
        "fmt",
        "go",
        "hash",
        "html",
        "image",
        "index",
        "io",
        "log",
        "maps",
        "math",
        "mime",
        "net",
        "os",
        "path",
        "plugin",
        "reflect",
        "regexp",
        "runtime",
        "slices",
        "sort",
        "strconv",
        "strings",
        "sync",
        "syscall",
        "testing",
        "text",
        "time",
        "unicode",
        "unsafe",
    ];

    let first_part = path.split('/').next().unwrap_or(path);

    if stdlibs.contains(&first_part) {
        ImportCategory::StandardLibrary
    } else if path.starts_with('.') {
        ImportCategory::Local
    } else {
        ImportCategory::ThirdParty
    }
}

/// I003: wrong-import-order - 导入顺序错误
pub struct WrongImportOrder;

impl Rule for WrongImportOrder {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "I003",
            name: "wrong-import-order",
            description: "标准库导入应在第三方库之前",
            category: RuleCategory::Imports,
            priority: RulePriority::Required,
            default_severity: Severity::Warning,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "import_declaration" {
            // 检查是否是 import 块
            let import_text = &source[node.byte_range()];

            if import_text.contains('\n') {
                let lines: Vec<&str> = import_text.lines().collect();
                let mut saw_third_party = false;

                for line in &lines {
                    let trimmed = line.trim();
                    if trimmed.starts_with('"')
                        || (trimmed.contains("\"") && !trimmed.starts_with("import"))
                    {
                        if let Some(start) = trimmed.find('"') {
                            if let Some(end) = trimmed[start + 1..].find('"') {
                                let import_path = &trimmed[start + 1..start + 1 + end];
                                let category = categorize_import(import_path);

                                match category {
                                    ImportCategory::ThirdParty => {
                                        saw_third_party = true;
                                    }
                                    ImportCategory::StandardLibrary if saw_third_party => {
                                        diagnostics.push(Diagnostic {
                                            code: "I003".to_string(),
                                            message: format!(
                                                "标准库导入 '{}' 应在第三方库之前",
                                                import_path
                                            ),
                                            severity: self.default_severity(),
                                            file_path: file_path.to_string(),
                                            line: node.start_position().row + 1,
                                            column: 1,
                                            fix: None,
                                        });
                                        return diagnostics;
                                    }
                                    _ => {}
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

/// I004: blank-import-unused - 空白导入未使用
pub struct BlankImportUnused;

impl Rule for BlankImportUnused {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "I004",
            name: "blank-import-unused",
            description: "空白导入 (_ \"package\") 应仅用于 init 副作用",
            category: RuleCategory::Imports,
            priority: RulePriority::Required,
            default_severity: Severity::Info,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "import_declaration" {
            let import_text = &source[node.byte_range()];

            // 检查是否是空白导入
            if import_text.contains("_ ") && import_text.contains('"') {
                // 提取包名
                if let Some(start) = import_text.find('"') {
                    if let Some(end) = import_text[start + 1..].find('"') {
                        let pkg_path = &import_text[start + 1..start + 1 + end];

                        // 常见的需要空白导入的包
                        let common_side_effect_imports = [
                            "database/sql",
                            "github.com/lib/pq",
                            "github.com/mattn/go-sqlite3",
                            "image/png",
                            "image/jpeg",
                            "image/gif",
                            "golang.org/x/image/webp",
                            "crypto/md5",
                            "crypto/sha256",
                            "crypto/sha512",
                        ];

                        if !common_side_effect_imports.contains(&pkg_path) {
                            // 警告：可能是不必要的空白导入
                            let pos = node.start_position();
                            diagnostics.push(Diagnostic {
                                code: "I004".to_string(),
                                message: format!(
                                    "空白导入 '{}' 可能不必要，确保是为了 init 副作用",
                                    pkg_path
                                ),
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

/// I006: import-alias-necessary - 应使用别名
pub struct ImportAliasNecessary;

impl Rule for ImportAliasNecessary {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "I006",
            name: "import-alias-necessary",
            description: "导入包名与目录名不同时应使用别名",
            category: RuleCategory::Imports,
            priority: RulePriority::Recommended,
            default_severity: Severity::Info,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "import_declaration" {
            let import_text = &source[node.byte_range()];

            // 检查是否有别名
            let has_alias = import_text.contains("\"")
                && import_text
                    .split('"')
                    .next()
                    .map(|before| before.split_whitespace().count() > 1)
                    .unwrap_or(false);

            if !has_alias {
                // 提取包路径
                if let Some(start) = import_text.find('"') {
                    if let Some(end) = import_text[start + 1..].find('"') {
                        let pkg_path = &import_text[start + 1..start + 1 + end];

                        // 检查包路径最后一部分是否与目录名匹配
                        if let Some(last_slash) = pkg_path.rfind('/') {
                            let dir_name = &pkg_path[last_slash + 1..];

                            // 如果目录名包含连字符或点，建议别名
                            if dir_name.contains('-') || dir_name.contains('.') {
                                let suggested_alias = dir_name.replace(['-', '.'], "");
                                let pos = node.start_position();
                                diagnostics.push(Diagnostic {
                                    code: "I006".to_string(),
                                    message: format!(
                                        "建议为导入添加别名: {} \"{}\"",
                                        suggested_alias, pkg_path
                                    ),
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

/// I007: import-alias-unnecessary - 别名不必要
pub struct ImportAliasUnnecessary;

impl Rule for ImportAliasUnnecessary {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "I007",
            name: "import-alias-unnecessary",
            description: "导入别名与包名相同，可以省略",
            category: RuleCategory::Imports,
            priority: RulePriority::Recommended,
            default_severity: Severity::Info,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "import_declaration" {
            let import_text = &source[node.byte_range()];

            // 解析别名和路径
            let parts: Vec<&str> = import_text.split('"').collect();
            if parts.len() >= 2 {
                let before_quote = parts[0];
                let path = parts[1];

                // 提取别名
                let words: Vec<&str> = before_quote.split_whitespace().collect();
                if words.len() >= 2 && words[0] == "import" {
                    let alias = words[1];

                    // 获取包名
                    let pkg_name = path.rfind('/').map(|i| &path[i + 1..]).unwrap_or(path);

                    if alias == pkg_name {
                        let pos = node.start_position();
                        diagnostics.push(Diagnostic {
                            code: "I007".to_string(),
                            message: format!("别名 '{}' 与包名相同，可以省略", alias),
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

/// I008: dot-import - 点导入不推荐
pub struct DotImport;

impl Rule for DotImport {
    fn metadata(&self) -> RuleMetadata {
        RuleMetadata {
            code: "I008",
            name: "dot-import",
            description: "点导入 (import . \"pkg\") 不推荐使用",
            category: RuleCategory::Imports,
            priority: RulePriority::Recommended,
            default_severity: Severity::Warning,
            min_go_version: None,
        }
    }

    fn check(&self, node: Node, source: &str, file_path: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        if node.kind() == "import_declaration" {
            let import_text = &source[node.byte_range()];

            // 检查是否是点导入
            if import_text.contains(". \"") || import_text.contains(".\"") {
                if let Some(start) = import_text.find('"') {
                    if let Some(end) = import_text[start + 1..].find('"') {
                        let pkg_path = &import_text[start + 1..start + 1 + end];
                        let pos = node.start_position();

                        diagnostics.push(Diagnostic {
                            code: "I008".to_string(),
                            message: format!(
                                "点导入 '. \"{}\"' 不推荐使用，会使命名空间混乱",
                                pkg_path
                            ),
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
        Box::new(UnsortedImports),
        Box::new(ImportGroupsMissing),
        Box::new(WrongImportOrder),
        Box::new(BlankImportUnused),
        Box::new(ImportAliasNecessary),
        Box::new(ImportAliasUnnecessary),
        Box::new(DotImport),
    ]
}
