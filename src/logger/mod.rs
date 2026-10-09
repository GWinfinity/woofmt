//! Logging system for woof
//!
//! Supports debug logging via GOLOGGING environment variable:
//! - GOLOGGING=debug - Enable debug logging
//! - GOLOGGING=trace - Enable trace logging (more verbose)
//! - GOLOGGING=perf  - Enable performance logging only

use std::env;
use std::time::{Duration, Instant};

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// No logging
    Off,
    /// Performance metrics only
    Perf,
    /// Debug information
    Debug,
    /// Verbose trace
    Trace,
}

impl LogLevel {
    /// Parse from environment variable
    pub fn from_env() -> Self {
        match env::var("GOLOGGING").as_deref() {
            Ok("trace") => LogLevel::Trace,
            Ok("debug") => LogLevel::Debug,
            Ok("perf") => LogLevel::Perf,
            Ok("1") | Ok("true") | Ok("yes") => LogLevel::Debug,
            _ => LogLevel::Off,
        }
    }

    /// Check if level is enabled
    pub fn is_enabled(&self, required: LogLevel) -> bool {
        let self_val = *self as u8;
        let required_val = required as u8;
        self_val >= required_val
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Global log level (lazy initialized)
static LOG_LEVEL: std::sync::LazyLock<LogLevel> = std::sync::LazyLock::new(LogLevel::from_env);

/// Get current log level
pub fn log_level() -> LogLevel {
    *LOG_LEVEL
}

/// Check if logging is enabled
pub fn is_enabled(level: LogLevel) -> bool {
    log_level().is_enabled(level)
}

/// Log a debug message
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if $crate::logger::is_enabled($crate::logger::LogLevel::Debug) {
            eprintln!("[woofmt:debug] {}", format!($($arg)*));
        }
    };
}

/// Log a trace message
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        if $crate::logger::is_enabled($crate::logger::LogLevel::Trace) {
            eprintln!("[woofmt:trace] {}", format!($($arg)*));
        }
    };
}

/// Log a performance metric
#[macro_export]
macro_rules! log_perf {
    ($($arg:tt)*) => {
        if $crate::logger::is_enabled($crate::logger::LogLevel::Perf) {
            eprintln!("[woofmt:perf] {}", format!($($arg)*));
        }
    };
}

/// Performance timer for measuring operations
pub struct PerfTimer {
    name: String,
    start: Instant,
    level: LogLevel,
}

impl PerfTimer {
    /// Create a new performance timer
    pub fn new<S: Into<String>>(name: S) -> Self {
        let name = name.into();
        let start = Instant::now();

        if is_enabled(LogLevel::Perf) {
            eprintln!("[woofmt:perf] Started: {}", name);
        }

        Self {
            name,
            start,
            level: LogLevel::Perf,
        }
    }

    /// Create a timer with debug level
    pub fn debug<S: Into<String>>(name: S) -> Self {
        let name = name.into();
        let start = Instant::now();

        if is_enabled(LogLevel::Debug) {
            eprintln!("[woofmt:debug] Started: {}", name);
        }

        Self {
            name,
            start,
            level: LogLevel::Debug,
        }
    }

    /// Create a timer with custom level
    pub fn with_level<S: Into<String>>(name: S, level: LogLevel) -> Self {
        let name = name.into();
        let start = Instant::now();

        if is_enabled(level) {
            let level_str = match level {
                LogLevel::Trace => "trace",
                LogLevel::Debug => "debug",
                LogLevel::Perf => "perf",
                LogLevel::Off => "off",
            };
            eprintln!("[woofmt:{}] Started: {}", level_str, name);
        }

        Self { name, start, level }
    }

    /// Get elapsed time without stopping
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Stop the timer and log the result
    pub fn stop(self) -> Duration {
        let elapsed = self.start.elapsed();
        let micros = elapsed.as_micros();

        if is_enabled(self.level) {
            let level_str = match self.level {
                LogLevel::Trace => "trace",
                LogLevel::Debug => "debug",
                LogLevel::Perf => "perf",
                LogLevel::Off => "off",
            };

            if micros < 1000 {
                eprintln!(
                    "[woofmt:{}] Finished: {} - {}µs",
                    level_str, self.name, micros
                );
            } else {
                let millis = elapsed.as_millis();
                eprintln!(
                    "[woofmt:{}] Finished: {} - {}ms",
                    level_str, self.name, millis
                );
            }
        }

        elapsed
    }
}

impl Drop for PerfTimer {
    fn drop(&mut self) {
        // Note: This doesn't print on drop to avoid duplicate logging
        // Use .stop() explicitly to log
    }
}

/// Linter performance tracker
pub struct LinterPerfTracker {
    rule_name: String,
    rule_code: String,
    start: Instant,
}

impl LinterPerfTracker {
    /// Start tracking a rule's execution time
    pub fn start<S: Into<String>>(rule_code: S, rule_name: S) -> Self {
        let rule_code = rule_code.into();
        let rule_name = rule_name.into();
        let start = Instant::now();

        if is_enabled(LogLevel::Perf) || is_enabled(LogLevel::Debug) {
            eprintln!("[woofmt:perf] Rule started: [{}] {}", rule_code, rule_name);
        }

        Self {
            rule_name,
            rule_code,
            start,
        }
    }

    /// Stop tracking and log the result
    pub fn stop(self) -> Duration {
        let elapsed = self.start.elapsed();
        let micros = elapsed.as_micros();

        if is_enabled(LogLevel::Perf) || is_enabled(LogLevel::Debug) {
            if micros < 1000 {
                eprintln!(
                    "[woofmt:perf] Rule finished: [{}] {} - {}µs",
                    self.rule_code, self.rule_name, micros
                );
            } else {
                let millis = elapsed.as_millis();
                eprintln!(
                    "[woofmt:perf] Rule finished: [{}] {} - {}ms",
                    self.rule_code, self.rule_name, millis
                );
            }
        }

        elapsed
    }

    /// Get elapsed time without stopping
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// Aggregate performance statistics
#[derive(Debug, Default)]
pub struct PerfStats {
    pub total_rules: usize,
    pub total_time_ms: u64,
    pub rule_times: Vec<RulePerfStat>,
}

#[derive(Debug, Clone)]
pub struct RulePerfStat {
    pub code: String,
    pub name: String,
    pub time_us: u64,
    pub percentage: f64,
}

impl PerfStats {
    /// Print formatted statistics
    pub fn print(&self) {
        if !is_enabled(LogLevel::Perf) && !is_enabled(LogLevel::Debug) {
            return;
        }

        eprintln!("\n[woofmt:perf] === Linter Performance Summary ===");
        eprintln!("[woofmt:perf] Total rules: {}", self.total_rules);
        eprintln!("[woofmt:perf] Total time: {}ms", self.total_time_ms);
        eprintln!("[woofmt:perf]");
        eprintln!("[woofmt:perf] Rule breakdown:");

        // Sort by time descending
        let mut sorted = self.rule_times.clone();
        sorted.sort_by_key(|s| std::cmp::Reverse(s.time_us));

        for stat in sorted.iter().take(10) {
            let time_ms = stat.time_us as f64 / 1000.0;
            eprintln!(
                "[woofmt:perf]   [{:<6}] {:<25} {:>8.2}ms ({:>5.1}%)",
                stat.code, stat.name, time_ms, stat.percentage
            );
        }

        if sorted.len() > 10 {
            eprintln!("[woofmt:perf]   ... and {} more rules", sorted.len() - 10);
        }

        eprintln!("[woofmt:perf] === End Performance Summary ===\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_env() {
        // Note: This test may fail depending on environment
        // It's here for documentation purposes
        let level = LogLevel::from_env();
        println!("Current log level: {:?}", level);
    }

    #[test]
    fn test_perf_timer() {
        if is_enabled(LogLevel::Perf) {
            let timer = PerfTimer::new("test_operation");
            std::thread::sleep(Duration::from_millis(10));
            let elapsed = timer.stop();
            assert!(elapsed >= Duration::from_millis(10));
        }
    }
}
