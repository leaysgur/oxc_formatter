mod arguments;
mod buffer;
mod builders;
mod context;
mod format;
mod format_element;
mod formatter;
mod group_id;
mod macros;
mod options;
mod printer;
mod state;

use oxc_allocator::Allocator;
use oxc_span::SourceType;

use arguments::Arguments;
use buffer::{Buffer, VecBuffer};
use context::FormatContext;
use format::Format;
use format_element::document::Document;
use formatter::Formatter;
use options::FormatOptions;
use printer::Printer;
use state::FormatState;

// ---

type PrintError = String; // TODO: diagnostics
type PrintResult<T> = Result<T, PrintError>;
type FormatError = String; // TODO: diagnostics
type FormatResult<T> = Result<T, FormatError>;

// ---

#[inline(always)]
fn write(output: &mut dyn Buffer, args: Arguments) {
    let mut f = Formatter::new(output);
    f.write_fmt(args);
}

// ---

pub fn format_source(source_text: &str, source_type: SourceType) -> FormatResult<String> {
    let allocator = Allocator::new();

    let parser = oxc_parser::Parser::new(&allocator, source_text, source_type);
    let parsed = parser.parse();

    // TODO: Transform AST

    let options = FormatOptions::default();

    let context = FormatContext::new(options);
    let mut state = FormatState::new(context);
    let mut buffer = VecBuffer::new(&mut state);

    let mut f = Formatter::new(&mut buffer);
    parsed.program.fmt(&mut f);

    let mut document = Document::from(buffer.into_vec());
    document.propagate_expand();

    let context = state.into_context();
    let printer = Printer::new(
        parsed.program.source_text,
        context.options().as_print_options(),
    );

    let printed = printer.print(&document)?;
    Ok(printed)
}
