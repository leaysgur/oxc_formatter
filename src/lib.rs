pub mod format_element;
pub mod group_id;
pub mod options;
pub mod prelude;
pub mod printer;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_span::{GetSpan, SourceType};

pub use format_element::{FormatElement, LINE_TERMINATORS, normalize_newlines};
pub use group_id::GroupId;
pub use options::*;
use prelude::Document;
pub use printer::{Printer, PrinterOptions};

type PrintError = String; // TODO: diagnostics
pub type PrintResult<T> = Result<T, PrintError>;

pub struct Printed {
    code: String,
}
impl Printed {
    fn new(code: String) -> Self {
        Self { code }
    }
}

struct Formatter<'a> {
    elements: Vec<FormatElement>,
    pub source_text: &'a str,
}
impl<'a> Formatter<'a> {
    fn new(source_text: &'a str) -> Self {
        Self { elements: vec![], source_text }
    }

    fn write_element(&mut self, element: FormatElement) {
        self.elements.push(element);
    }
}

trait Format {
    fn fmt(&self, f: &mut Formatter) {
        self.fmt_fields(f);
    }

    fn fmt_fields(&self, f: &mut Formatter);
}

impl Format for Program<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        use format_element::tag::Tag;

        for stmt in &self.body {
            match stmt {
                Statement::VariableDeclaration(decl) => {
                    f.write_element(FormatElement::StaticText { text: decl.kind.as_str() });
                    f.write_element(FormatElement::Space);
                    for decl in &decl.declarations {
                        decl.fmt(f);
                    }
                }
                _ => {
                    f.write_element(FormatElement::DynamicText {
                        text: stmt.span().source_text(f.source_text).into(),
                    });
                }
            }
        }
    }
}

impl Format for VariableDeclarator<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        if let Some(name) = self.id.get_identifier_name().as_ref() {
            f.write_element(FormatElement::DynamicText { text: name.as_str().into() });
            f.write_element(FormatElement::Space);
        }
    }
}

// ---

fn format_source(source_text: &str, source_type: SourceType) -> Result<String, String> {
    let allocator = Allocator::new();

    let parser = oxc_parser::Parser::new(&allocator, source_text, source_type);
    let parsed = parser.parse();

    // TODO: Transform AST

    let options = FormatOptions::default();

    let mut f = Formatter::new(parsed.program.source_text);
    parsed.program.fmt(&mut f);

    let mut document = Document::from(f.elements);
    document.propagate_expand();

    let printer = Printer::new(options.as_print_options());
    let printed = printer.print(&document)?;
    Ok(printed.code)
}

// ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_source() {
        let result = format_source("const a=1;const b =   [2,3,4]", oxc_span::SourceType::mjs());
        assert!(result.is_ok());
        println!("✨ Formatted code:");
        println!("{}", result.unwrap());
    }
}
