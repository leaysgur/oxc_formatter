mod format_element;
mod formatter;
mod group_id;
mod options;
mod printer;

use oxc_allocator::Allocator;
use oxc_span::SourceType;

use format_element::{FormatElement, document::Document};
use formatter::{Format, Formatter};
use group_id::GroupId;
use options::*;
use printer::Printer;

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

// ---

pub fn format_source(source_text: &str, source_type: SourceType) -> Result<String, String> {
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
