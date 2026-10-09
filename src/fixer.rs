//! Auto-fix engine.
//!
//! Applies [`Diagnostic`] fixes to files on disk with the following safety
//! guarantees:
//!
//! - **Safe/unsafe gating**: fixes produced by rules are classified as safe or
//!   unsafe (see [`Fix::unsafe_fix`]). By default only safe fixes are applied;
//!   pass `unsafe_fixes = true` to opt in (mirrors `--unsafe-fixes`).
//! - **Overlap protection**: fixes whose byte ranges overlap are skipped
//!   (the fix that starts first wins). Overlapping edits are ambiguous and
//!   must never be applied blindly.
//! - **Bounds checking**: fixes pointing outside the file are skipped.
//! - **Grouped application**: all fixes for one file are applied in a single
//!   pass, sorted from the end of the file backwards, so earlier byte offsets
//!   stay valid.
//!
//! # Example
//!
//! ```
//! use woofmt::{Diagnostic, Fix, Severity};
//! use woofmt::fixer;
//!
//! let diags = vec![Diagnostic {
//!     file_path: "does_not_exist.go".to_string(),
//!     line: 1,
//!     column: 1,
//!     message: "trailing whitespace".to_string(),
//!     code: "E115".to_string(),
//!     severity: Severity::Info,
//!     fix: Some(Fix::safe("remove trailing whitespace", "", 0, 5)),
//! }];
//!
//! // Missing files are reported as errors, not silently ignored.
//! assert!(fixer::apply_fixes(&diags, false).is_err());
//! ```

use crate::Diagnostic;
use anyhow::Result;
use std::collections::BTreeMap;

/// Summary of a fix application run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FixReport {
    /// Number of fixes successfully applied.
    pub applied: usize,
    /// Number of fixes skipped because their ranges overlapped another fix.
    pub skipped_overlap: usize,
    /// Number of unsafe fixes skipped because `unsafe_fixes` was false.
    pub skipped_unsafe: usize,
    /// Number of files that were rewritten.
    pub files_modified: usize,
}

impl FixReport {
    /// Returns true if nothing was changed on disk.
    pub fn is_empty(&self) -> bool {
        self.applied == 0 && self.files_modified == 0
    }
}

/// A single pending edit resolved against a concrete file.
struct PendingFix<'a> {
    start: usize,
    end: usize,
    replacement: &'a str,
}

/// Apply fixes from `diagnostics` to the files they reference.
///
/// File paths are interpreted as-is (relative to the current working
/// directory), matching how the linter reports them.
///
/// When `unsafe_fixes` is false, fixes flagged as unsafe are counted in
/// [`FixReport::skipped_unsafe`] and left untouched.
pub fn apply_fixes(diagnostics: &[Diagnostic], unsafe_fixes: bool) -> Result<FixReport> {
    let mut report = FixReport::default();

    // Group fixes by file, preserving deterministic (file, offset) order via BTreeMap.
    let mut by_file: BTreeMap<String, Vec<PendingFix<'_>>> = BTreeMap::new();
    for diag in diagnostics {
        if let Some(fix) = &diag.fix {
            if fix.unsafe_fix && !unsafe_fixes {
                report.skipped_unsafe += 1;
                continue;
            }
            by_file.entry(diag.file_path.clone()).or_default().push(PendingFix {
                start: fix.start_byte,
                end: fix.end_byte,
                replacement: fix.replacement.as_str(),
            });
        }
    }

    for (path, mut fixes) in by_file {
        // Order by start offset; on ties, prefer the narrower fix.
        fixes.sort_by(|a, b| a.start.cmp(&b.start).then(a.end.cmp(&b.end)));

        let source = std::fs::read(&path)
            .map_err(|e| anyhow::anyhow!("failed to read {} for auto-fix: {}", path, e))?;
        let source_len = source.len();

        // Validate and de-overlap from the front, then apply from the back.
        let mut accepted: Vec<&PendingFix<'_>> = Vec::new();
        let mut last_end: Option<usize> = None;
        for fix in &fixes {
            // Bounds check.
            if fix.start > fix.end || fix.end > source_len {
                // Out-of-range fix: malformed rule output; skip defensively.
                report.skipped_overlap += 1;
                continue;
            }
            // Overlap check: a fix starting before the previous accepted fix
            // ends would corrupt the file if both were applied.
            if let Some(prev_end) = last_end {
                if fix.start < prev_end {
                    report.skipped_overlap += 1;
                    continue;
                }
            }
            last_end = Some(fix.end);
            accepted.push(fix);
        }

        if accepted.is_empty() {
            continue;
        }

        let mut content = source;
        for fix in accepted.iter().rev() {
            content.splice(fix.start..fix.end, fix.replacement.as_bytes().to_vec());
        }

        std::fs::write(&path, &content)
            .map_err(|e| anyhow::anyhow!("failed to write fixed file {}: {}", path, e))?;

        report.applied += accepted.len();
        report.files_modified += 1;
    }

    Ok(report)
}

/// Render a human-readable fix report line for CLI output.
pub fn format_report(report: &FixReport) -> String {
    let mut parts = vec![format!("{} fix(es) applied", report.applied)];
    if report.skipped_unsafe > 0 {
        parts.push(format!(
            "{} unsafe skipped (use --unsafe-fixes to apply)",
            report.skipped_unsafe
        ));
    }
    if report.skipped_overlap > 0 {
        parts.push(format!("{} skipped due to overlap", report.skipped_overlap));
    }
    format!(
        "{} across {} file(s)",
        parts.join(", "),
        report.files_modified
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Diagnostic, Fix, Severity};
    use std::io::Write;

    fn diag(path: &str, code: &str, start: usize, end: usize, replacement: &str) -> Diagnostic {
        Diagnostic {
            file_path: path.to_string(),
            line: 1,
            column: 1,
            message: "test".to_string(),
            code: code.to_string(),
            severity: Severity::Warning,
            fix: Some(Fix::safe("test fix", replacement, start, end)),
        }
    }

    fn tempfile_with(content: &str) -> (tempfile::NamedTempFile, String) {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, "{content}").unwrap();
        let p = f.path().to_string_lossy().to_string();
        (f, p)
    }

    #[test]
    fn applies_simple_replacement() {
        let (_f, p) = tempfile_with("hello world");
        let diags = vec![diag(&p, "T001", 6, 11, "there")];
        let report = apply_fixes(&diags, false).unwrap();
        assert_eq!(report.applied, 1);
        assert_eq!(report.files_modified, 1);
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "hello there");
    }

    #[test]
    fn applies_multiple_fixes_back_to_front() {
        let (_f, p) = tempfile_with("aaa bbb ccc");
        let diags = vec![
            diag(&p, "T001", 0, 3, "X"),
            diag(&p, "T001", 8, 11, "Z"),
            diag(&p, "T001", 4, 7, "Y"),
        ];
        let report = apply_fixes(&diags, false).unwrap();
        assert_eq!(report.applied, 3);
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "X Y Z");
    }

    #[test]
    fn skips_overlapping_fixes() {
        let (_f, p) = tempfile_with("abcdef");
        let diags = vec![
            diag(&p, "T001", 0, 4, "X"),
            diag(&p, "T001", 2, 5, "Y"), // overlaps previous
        ];
        let report = apply_fixes(&diags, false).unwrap();
        assert_eq!(report.applied, 1);
        assert_eq!(report.skipped_overlap, 1);
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "Xef");
    }

    #[test]
    fn skips_unsafe_fixes_unless_opted_in() {
        let (_f, p) = tempfile_with("abcdef");
        let mut d = diag(&p, "T002", 0, 3, "X");
        d.fix.as_mut().unwrap().unsafe_fix = true;

        let report = apply_fixes(std::slice::from_ref(&d), false).unwrap();
        assert_eq!(report.skipped_unsafe, 1);
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "abcdef");

        let report = apply_fixes(std::slice::from_ref(&d), true).unwrap();
        assert_eq!(report.applied, 1);
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "Xdef");
    }

    #[test]
    fn skips_out_of_bounds_fixes() {
        let (_f, p) = tempfile_with("abc");
        let diags = vec![diag(&p, "T001", 10, 20, "X")];
        let report = apply_fixes(&diags, false).unwrap();
        assert_eq!(report.applied, 0);
        assert_eq!(report.skipped_overlap, 1);
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "abc");
    }

    #[test]
    fn errors_on_missing_file() {
        let diags = vec![diag("definitely_missing_file.go", "T001", 0, 1, "X")];
        assert!(apply_fixes(&diags, false).is_err());
    }

    #[test]
    fn diagnostics_without_fix_are_ignored() {
        let (_f, p) = tempfile_with("abcdef");
        let d = Diagnostic {
            file_path: p.clone(),
            line: 1,
            column: 1,
            message: "no fix".to_string(),
            code: "T003".to_string(),
            severity: Severity::Info,
            fix: None,
        };
        let report = apply_fixes(&[d], false).unwrap();
        assert!(report.is_empty());
    }
}
