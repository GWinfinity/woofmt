use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use std::process;

use woofmt::cli::{Cli, Commands, OutputFormat};
use woofmt::config::Config;

use woofmt::logger::{is_enabled, LogLevel};

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", "Error".red().bold(), e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let mut config = Config::load(cli.config.as_deref())?;

    // Set up thread pool (handle both --threads and --parallel)
    let thread_count = cli.threads.or(cli.parallel);
    if let Some(threads) = thread_count {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .ok();
        // Update config with CLI thread count
        config.global.parallelism.num_threads = threads;
    }

    match &cli.command {
        Some(Commands::Check {
            files,
            fix,
            unsafe_fixes,
            exit_non_zero_on_fix,
        }) => {
            run_check(
                files,
                *fix,
                *unsafe_fixes,
                *exit_non_zero_on_fix,
                &config,
                &cli,
            )?;
        }
        Some(Commands::Format {
            files,
            check,
            stdout,
        }) => {
            run_format(files, *check, *stdout, &config)?;
        }
        Some(Commands::Rules { all }) => {
            run_rules(*all, &config)?;
        }
        Some(Commands::Init { strict }) => {
            run_init(*strict)?;
        }
        None => {
            // Default to check mode with files from CLI (default: cwd)
            let mut files = cli.get_files();
            if files.is_empty() {
                files.push(std::path::PathBuf::from("."));
            }
            run_check(&files, false, false, false, &config, &cli)?;
        }
    }

    Ok(())
}

fn run_check(
    files: &[std::path::PathBuf],
    fix: bool,
    unsafe_fixes: bool,
    exit_non_zero_on_fix: bool,
    config: &Config,
    cli: &Cli,
) -> Result<()> {
    // Use profiled linter when GOLOGGING is enabled
    let use_profiled = is_enabled(LogLevel::Perf) || is_enabled(LogLevel::Debug);

    // Use parallel linter for better performance
    let use_parallel = std::env::var("WOOF_PARALLEL")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(true);

    let all_diagnostics = if use_parallel && !use_profiled && !files.is_empty() {
        // Use high-performance parallel linter for all files at once
        use woofmt::linter::parallel::lint_paths_parallel;

        // Collect all files from all paths
        let mut all_files = Vec::new();
        for file in files {
            if file.is_file() {
                all_files.push(file.clone());
            } else {
                // Directory - collect Go files
                let path_files = woofmt::linter::parallel::collect_go_files(file, config)?;
                all_files.extend(path_files);
            }
        }

        let (diags, _metrics) = lint_paths_parallel(&all_files, config)?;
        diags
    } else {
        let mut combined = Vec::new();
        for file in files {
            let diagnostics = if use_profiled {
                use woofmt::linter::profiled::lint_path_profiled;
                let (diags, _stats) = lint_path_profiled(file, config)?;
                diags
            } else {
                woofmt::lint_path(file, config)?
            };
            combined.extend(diagnostics);
        }
        combined
    };

    // Apply fixes if requested
    if fix && !all_diagnostics.is_empty() {
        let report = woofmt::fixer::apply_fixes(&all_diagnostics, unsafe_fixes)?;
        println!(
            "{}: {}",
            "Fixed".green().bold(),
            woofmt::fixer::format_report(&report)
        );
    }

    // Output diagnostics (skip if quiet mode)
    if !cli.quiet {
        match cli.format {
            OutputFormat::Text => output_text(&all_diagnostics),
            OutputFormat::Json => output_json(&all_diagnostics)?,
            OutputFormat::Github => output_github(&all_diagnostics),
        }

        // Statistics (暂时禁用)
        // if cli.statistics {
        //     let stats = LintStats::from_diagnostics(&all_diagnostics, files.len());
        //     output_stats(&stats);
        // }
    }

    // Exit code
    if !all_diagnostics.is_empty() {
        let has_errors = all_diagnostics
            .iter()
            .any(|d| matches!(d.severity, woofmt::Severity::Error));

        if has_errors || (fix && exit_non_zero_on_fix) {
            process::exit(1);
        }
    }

    Ok(())
}

fn run_format(
    files: &[std::path::PathBuf],
    check: bool,
    stdout: bool,
    config: &Config,
) -> Result<()> {
    let mut all_unchanged = true;

    for file in files {
        if stdout && file.is_file() {
            // Read and format to stdout
            let source = std::fs::read_to_string(file)?;
            let formatted = woofmt::format_to_string(&source, config)?;
            print!("{}", formatted);
        } else {
            let result = woofmt::format_path(file, check, config)?;

            if check && !result.unchanged {
                if all_unchanged {
                    eprintln!(
                        "{}: The following files need formatting:",
                        "Error".red().bold()
                    );
                }
                eprintln!("  - {}", file.display());
                all_unchanged = false;
            }
        }
    }

    if check && !all_unchanged {
        process::exit(1);
    }

    Ok(())
}

fn run_rules(all: bool, config: &Config) -> Result<()> {
    let rules = woofmt::rules::get_all_rules();

    println!("{}", "Available Rules:".bold());
    println!();

    for rule in rules {
        let enabled = if all {
            true
        } else {
            config.is_rule_enabled(rule.code())
        };

        let status = if enabled {
            "✓".green()
        } else {
            "✗".dimmed()
        };

        println!(
            "{} {} {} - {} {}",
            status,
            rule.code().cyan().bold(),
            rule.name().white(),
            rule.description(),
            if enabled {
                "".to_string()
            } else {
                "[disabled]".dimmed().to_string()
            }
        );
    }

    Ok(())
}

fn run_init(strict: bool) -> Result<()> {
    let config_path = "woof.toml";

    if std::path::Path::new(config_path).exists() {
        anyhow::bail!("Configuration file already exists: {}", config_path);
    }

    let content = Config::generate_default(strict);
    std::fs::write(config_path, content)?;

    println!("{} Created {}", "✓".green().bold(), config_path.green());

    Ok(())
}

fn output_text(diagnostics: &[woofmt::Diagnostic]) {
    for diag in diagnostics {
        let severity_color = match diag.severity {
            woofmt::Severity::Error => "error".red().bold(),
            woofmt::Severity::Warning => "warning".yellow().bold(),
            woofmt::Severity::Info => "info".blue().bold(),
        };

        println!(
            "{}:{}:{}: {} [{}] {}",
            diag.file_path,
            diag.line,
            diag.column,
            severity_color,
            diag.code.cyan(),
            diag.message
        );

        if let Some(fix) = &diag.fix {
            println!(
                "  {} {}",
                "-->".dimmed(),
                format!("Suggested fix: {}", fix.description).dimmed()
            );
        }
    }
}

fn output_json(diagnostics: &[woofmt::Diagnostic]) -> Result<()> {
    let json: Vec<_> = diagnostics
        .iter()
        .map(|d| {
            serde_json::json!({
                "file": d.file_path,
                "line": d.line,
                "column": d.column,
                "message": d.message,
                "code": d.code,
                "severity": format!("{}", d.severity),
                "fix": d.fix.as_ref().map(|f| {
                    serde_json::json!({
                        "description": f.description,
                        "replacement": f.replacement,
                    })
                }),
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

fn output_github(diagnostics: &[woofmt::Diagnostic]) {
    // GitHub Actions annotation format
    for diag in diagnostics {
        let level = match diag.severity {
            woofmt::Severity::Error => "error",
            woofmt::Severity::Warning => "warning",
            woofmt::Severity::Info => "notice",
        };

        println!(
            "::{} file={},line={},col={}::[{}] {}",
            level, diag.file_path, diag.line, diag.column, diag.code, diag.message
        );
    }
}

fn _output_stats(_stats: &()) {
    // 统计功能暂时禁用
}
