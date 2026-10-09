use crate::config::FormatterConfig;
use anyhow::Result;
use tree_sitter::Node;

/// Go code formatter engine
pub struct Formatter<'a> {
    config: &'a FormatterConfig,
}

impl<'a> Formatter<'a> {
    pub fn new(config: &'a crate::config::Config) -> Self {
        Self {
            config: &config.formatter,
        }
    }

    pub fn format(&self, root: Node, source: &str) -> Result<String> {
        let mut printer = super::printer::Printer::new(self.config);
        self.format_node(root, source, &mut printer)?;
        Ok(printer.finish())
    }

    fn format_node(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        let kind = node.kind();

        match kind {
            "source_file" => self.format_source_file(node, source, printer)?,
            "package_clause" => self.format_package_clause(node, source, printer)?,
            "import_declaration" => self.format_import(node, source, printer)?,
            "import_spec" => self.format_import_spec(node, source, printer)?,
            "function_declaration" => self.format_function(node, source, printer)?,
            "method_declaration" => self.format_method(node, source, printer)?,
            "type_declaration" => self.format_type_decl(node, source, printer)?,
            "var_declaration" => self.format_var_decl(node, source, printer)?,
            "const_declaration" => self.format_const_decl(node, source, printer)?,
            "block" => self.format_block(node, source, printer)?,
            "if_statement" => self.format_if(node, source, printer)?,
            "for_statement" => self.format_for(node, source, printer)?,
            "return_statement" => self.format_return(node, source, printer)?,
            "call_expression" => self.format_call(node, source, printer)?,
            "binary_expression" => self.format_binary(node, source, printer)?,
            "comment" => self.format_comment(node, source, printer)?,
            _ => self.format_generic(node, source, printer)?,
        }

        Ok(())
    }

    fn format_source_file(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        let mut cursor = node.walk();
        let mut first = true;

        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();

                // Add blank lines between top-level declarations
                if !first && is_top_level(child) {
                    printer.blank_line();
                }

                self.format_node(child, source, printer)?;

                if is_top_level(child) {
                    first = false;
                }

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        // Ensure file ends with newline
        printer.ensure_newline();

        Ok(())
    }

    fn format_package_clause(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        self.write_node_text(node, source, printer);
        printer.newline();
        Ok(())
    }

    fn format_import(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        // Check if it's an import block
        let text = &source[node.start_byte()..node.end_byte()];
        if text.contains('\n') {
            // Multi-line import block
            printer.write_str("import (");
            printer.newline();
            printer.indent();

            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    if child.kind() == "import_spec" {
                        printer.indentation();
                        self.format_import_spec(child, source, printer)?;
                        printer.newline();
                    } else if child.kind() == "import_spec_list" {
                        // tree-sitter-go nests specs inside an import_spec_list
                        // node; iterate its children too.
                        let mut spec_cursor = child.walk();
                        if spec_cursor.goto_first_child() {
                            loop {
                                let spec = spec_cursor.node();
                                if spec.kind() == "import_spec" {
                                    printer.indentation();
                                    self.format_import_spec(spec, source, printer)?;
                                    printer.newline();
                                }
                                if !spec_cursor.goto_next_sibling() {
                                    break;
                                }
                            }
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            printer.dedent();
            printer.write_char(')');
            printer.newline();
        } else {
            // Single import
            self.write_node_text(node, source, printer);
            printer.newline();
        }

        Ok(())
    }

    fn format_import_spec(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        self.write_node_text(node, source, printer);
        Ok(())
    }

    fn format_function(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        // Format function signature
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();

                match child.kind() {
                    "func" => {
                        printer.write_str("func ");
                    }
                    "identifier" => {
                        self.write_node_text(child, source, printer);
                    }
                    "parameter_list" => {
                        self.format_parameter_list(child, source, printer)?;
                    }
                    "type_identifier" | "qualified_type" => {
                        printer.write_str(" ");
                        self.write_node_text(child, source, printer);
                    }
                    "block" => {
                        printer.write_str(" ");
                        self.format_block(child, source, printer)?;
                    }
                    "type_parameter_list" => {
                        self.write_node_text(child, source, printer);
                    }
                    _ => {
                        if child.is_named() {
                            self.format_node(child, source, printer)?;
                        }
                    }
                }

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        Ok(())
    }

    fn format_method(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        // Similar to function but with receiver
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();

                match child.kind() {
                    "func" => {
                        printer.write_str("func ");
                    }
                    "parameter_list" if is_receiver(child, node) => {
                        self.format_parameter_list(child, source, printer)?;
                        printer.write_str(" ");
                    }
                    "identifier" => {
                        self.write_node_text(child, source, printer);
                    }
                    "parameter_list" => {
                        self.format_parameter_list(child, source, printer)?;
                    }
                    // Method result type: needs a space after the parameter list.
                    "type_identifier"
                    | "qualified_type"
                    | "pointer_type"
                    | "slice_type"
                    | "map_type"
                    | "generic_type"
                    | "type_parameter_list" => {
                        printer.write_str(" ");
                        self.write_node_text(child, source, printer);
                    }
                    "block" => {
                        printer.write_str(" ");
                        self.format_block(child, source, printer)?;
                    }
                    _ => {
                        if child.is_named() {
                            self.write_node_text(child, source, printer);
                        }
                    }
                }

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        Ok(())
    }

    fn format_parameter_list(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        self.write_node_text(node, source, printer);
        Ok(())
    }

    fn format_type_decl(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        self.write_node_text(node, source, printer);
        Ok(())
    }

    fn format_var_decl(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        self.write_node_text(node, source, printer);
        Ok(())
    }

    fn format_const_decl(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        self.write_node_text(node, source, printer);
        Ok(())
    }

    fn format_block(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        let text = &source[node.start_byte()..node.end_byte()];
        let is_empty = text.trim() == "{}";

        if is_empty {
            printer.write_str("{}");
            return Ok(());
        }

        printer.write_char('{');
        printer.newline();
        printer.indent();

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            // Skip '{'
            if cursor.node().kind() == "{" && !cursor.goto_next_sibling() {
                printer.dedent();
                printer.write_char('}');
                return Ok(());
            }

            loop {
                let child = cursor.node();

                if child.kind() != "}" {
                    printer.indentation();
                    self.format_node(child, source, printer)?;

                    // Add newline after statements unless it's the last one
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                    if cursor.node().kind() == "}" {
                        break;
                    }
                    if cursor.node().kind() != "comment" {
                        printer.newline();
                    }
                }

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        printer.dedent();
        printer.newline();
        printer.indentation();
        printer.write_char('}');

        Ok(())
    }

    fn format_if(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        printer.write_str("if ");

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            // Skip 'if'
            cursor.goto_next_sibling();

            loop {
                let child = cursor.node();

                match child.kind() {
                    ";" => {
                        printer.write_str("; ");
                    }
                    "block" => {
                        printer.write_char(' ');
                        self.format_block(child, source, printer)?;
                    }
                    "if_statement" => {
                        printer.write_str(" else ");
                        self.format_if(child, source, printer)?;
                    }
                    "else_clause" => {
                        printer.write_str(" else");
                        let mut else_cursor = child.walk();
                        if else_cursor.goto_first_child() {
                            // Skip 'else'
                            while else_cursor.goto_next_sibling() {
                                let else_child = else_cursor.node();
                                if else_child.kind() == "block" {
                                    printer.write_char(' ');
                                    self.format_block(else_child, source, printer)?;
                                } else if else_child.kind() == "if_statement" {
                                    printer.write_char(' ');
                                    self.format_if(else_child, source, printer)?;
                                }
                            }
                        }
                    }
                    _ => {
                        self.format_node(child, source, printer)?;
                    }
                }

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        Ok(())
    }

    fn format_for(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        printer.write_str("for ");

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            // Skip 'for'
            cursor.goto_next_sibling();

            loop {
                let child = cursor.node();

                match child.kind() {
                    "block" => {
                        printer.write_char(' ');
                        self.format_block(child, source, printer)?;
                    }
                    _ => {
                        if child.is_named() {
                            self.format_node(child, source, printer)?;
                        }
                    }
                }

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        Ok(())
    }

    fn format_return(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        printer.write_str("return");

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            // Skip 'return'
            while cursor.goto_next_sibling() {
                let child = cursor.node();
                if child.is_named() {
                    printer.write_char(' ');
                    self.format_node(child, source, printer)?;
                }
            }
        }

        Ok(())
    }

    fn format_call(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        self.write_node_text(node, source, printer);
        Ok(())
    }

    fn format_binary(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            // Left operand
            let left = cursor.node();
            self.format_node(left, source, printer)?;

            // Operator
            if cursor.goto_next_sibling() {
                let op = cursor.node();
                printer.write_char(' ');
                self.write_node_text(op, source, printer);
                printer.write_char(' ');

                // Right operand
                if cursor.goto_next_sibling() {
                    let right = cursor.node();
                    self.format_node(right, source, printer)?;
                }
            }
        }

        Ok(())
    }

    fn format_comment(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        self.write_node_text(node, source, printer);
        Ok(())
    }

    fn format_generic(
        &self,
        node: Node,
        source: &str,
        printer: &mut super::printer::Printer,
    ) -> Result<()> {
        // Default: preserve text for unhandled nodes
        let text = &source[node.start_byte()..node.end_byte()];
        printer.write_str(text);
        Ok(())
    }

    fn write_node_text(&self, node: Node, source: &str, printer: &mut super::printer::Printer) {
        let text = &source[node.start_byte()..node.end_byte()];
        printer.write_str(text);
    }
}

fn is_top_level(node: Node) -> bool {
    matches!(
        node.kind(),
        "function_declaration"
            | "method_declaration"
            | "type_declaration"
            | "var_declaration"
            | "const_declaration"
            | "import_declaration"
    )
}

fn is_receiver(node: Node, parent: Node) -> bool {
    // Check if this parameter_list is the first one in a method declaration
    let mut cursor = parent.walk();
    if cursor.goto_first_child() {
        // Skip 'func'
        if cursor.goto_next_sibling() {
            return cursor.node().id() == node.id();
        }
    }
    false
}
