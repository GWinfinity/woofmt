//! Correctness regression tests for the formatter and auto-fix engine.
//!
//! These tests run against the corpus in `testdata/gofmt_corpus/` and enforce
//! two properties that must hold before any "gofmt-compatible" claim:
//!
//! 1. **Parse validity**: every corpus file must parse cleanly.
//! 2. **Idempotency**: `format(format(x)) == format(x)`. A formatter that is
//!    not idempotent will make `fmt --check` oscillate in CI.
//!
//! For byte-level gofmt equivalence, see `scripts/gofmt_compat.sh` (CI-only,
//! requires the Go toolchain).

use std::path::PathBuf;

fn corpus_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("testdata");
    p.push("gofmt_corpus");
    p
}

fn corpus_files() -> Vec<PathBuf> {
    let dir = corpus_dir();
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read corpus dir {}: {}", dir.display(), e))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "go").unwrap_or(false))
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "gofmt corpus is empty — add Go files to testdata/gofmt_corpus/"
    );
    files
}

#[test]
fn corpus_files_parse_and_format_idempotently() {
    let config = woofmt::config::Config::default();

    for file in corpus_files() {
        let source = std::fs::read_to_string(&file)
            .unwrap_or_else(|e| panic!("cannot read {}: {}", file.display(), e));

        let once = woofmt::format_to_string(&source, &config)
            .unwrap_or_else(|e| panic!("{} failed to format: {}", file.display(), e));
        let twice = woofmt::format_to_string(&once, &config).unwrap_or_else(|e| {
            panic!(
                "{} failed to re-format its own output (formatter is not stable): {}",
                file.display(),
                e
            )
        });

        assert_eq!(
            once,
            twice,
            "{}: formatter is NOT idempotent (format(format(x)) != format(x)). \
             CI `fmt --check` will oscillate on this file.",
            file.display()
        );
    }
}

#[test]
fn fixer_end_to_end_trailing_whitespace() {
    let config = woofmt::config::Config::default();

    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("ws.go");
    std::fs::write(&file, "package main\n\nfunc main() {}   \n").unwrap();

    let diags = woofmt::lint_path(&file, &config).unwrap();
    assert!(
        diags.iter().any(|d| d.code == "E115"),
        "E115 should flag trailing whitespace"
    );

    // Apply safe fixes via the fixer engine.
    let report = woofmt::fixer::apply_fixes(&diags, false).unwrap();
    assert!(
        report.applied > 0 && report.files_modified == 1,
        "expected trailing-whitespace fix to apply, got {report:?}"
    );

    let fixed = std::fs::read_to_string(&file).unwrap();
    assert_eq!(
        fixed, "package main\n\nfunc main() {}\n",
        "trailing whitespace must be removed without touching other lines"
    );
}

#[test]
fn fixer_never_applies_overlapping_edits() {
    use woofmt::{Diagnostic, Fix, Severity};

    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("overlap.go");
    std::fs::write(&file, "abcdef").unwrap();

    let mk = |start: usize, end: usize| Diagnostic {
        file_path: file.to_string_lossy().to_string(),
        line: 1,
        column: 1,
        message: "test".into(),
        code: "T000".into(),
        severity: Severity::Warning,
        fix: Some(Fix::safe("t", "X", start, end)),
    };

    let diags = vec![mk(0, 4), mk(2, 6)];
    let report = woofmt::fixer::apply_fixes(&diags, false).unwrap();

    assert_eq!(report.applied, 1, "only the first fix may win");
    assert_eq!(report.skipped_overlap, 1);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "Xef");
}
