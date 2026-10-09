use crate::config::FormatterConfig;

/// Code printer that handles indentation and formatting
pub struct Printer<'a> {
    config: &'a FormatterConfig,
    output: String,
    indent_level: usize,
    needs_indent: bool,
    last_char: Option<char>,
}

impl<'a> Printer<'a> {
    pub fn new(config: &'a FormatterConfig) -> Self {
        Self {
            config,
            output: String::new(),
            indent_level: 0,
            needs_indent: false,
            last_char: None,
        }
    }

    pub fn write_str(&mut self, s: &str) {
        self.apply_indent_if_needed();
        self.output.push_str(s);
        if let Some(c) = s.chars().last() {
            self.last_char = Some(c);
        }
    }

    pub fn write_char(&mut self, c: char) {
        self.apply_indent_if_needed();
        self.output.push(c);
        self.last_char = Some(c);
    }

    /// Emit a single line break.
    ///
    /// Collapses consecutive breaks: if the output already ends with `\n`,
    /// this is a no-op. This makes the printer idempotent — re-formatting
    /// formatted output never grows extra blank lines (use [`Self::blank_line`]
    /// when a blank line is explicitly wanted).
    pub fn newline(&mut self) {
        if self.output.ends_with('\n') {
            return;
        }
        self.output.push('\n');
        self.needs_indent = true;
        self.last_char = Some('\n');
    }

    pub fn blank_line(&mut self) {
        // Avoid multiple blank lines
        if !self.output.ends_with("\n\n") && !self.output.is_empty() {
            if !self.output.ends_with('\n') {
                self.output.push('\n');
            }
            self.output.push('\n');
            self.needs_indent = true;
        }
    }

    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub fn indentation(&mut self) {
        self.needs_indent = true;
        self.apply_indent_if_needed();
    }

    fn apply_indent_if_needed(&mut self) {
        if self.needs_indent {
            let indent = if self.config.use_tabs {
                "\t".repeat(self.indent_level)
            } else {
                " ".repeat(self.indent_level * self.config.tab_width)
            };
            self.output.push_str(&indent);
            self.needs_indent = false;
        }
    }

    pub fn ensure_newline(&mut self) {
        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    pub fn finish(self) -> String {
        self.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_config() -> FormatterConfig {
        FormatterConfig {
            use_tabs: false,
            tab_width: 4,
            line_length: 120,
            simplify: false,
        }
    }

    #[test]
    fn test_basic_writing() {
        let config = create_config();
        let mut printer = Printer::new(&config);

        printer.write_str("package main");
        assert_eq!(printer.finish(), "package main");
    }

    #[test]
    fn test_indentation() {
        let config = create_config();
        let mut printer = Printer::new(&config);

        printer.write_str("{");
        printer.newline();
        printer.indent();
        printer.indentation();
        printer.write_str("x := 1");
        printer.finish();
    }

    #[test]
    fn test_tabs() {
        let config = FormatterConfig {
            use_tabs: true,
            tab_width: 4,
            line_length: 120,
            simplify: false,
        };
        let mut printer = Printer::new(&config);

        printer.indent();
        printer.indentation();
        printer.write_str("hello");

        assert_eq!(printer.finish(), "\thello");
    }
}
