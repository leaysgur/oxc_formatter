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
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

use arguments::Arguments;
use buffer::{Buffer, VecBuffer};
use context::FormatContext;
use format::FormatNode;
use format_element::document::Document;
use formatter::Formatter;
pub use options::FormatOptions;
use printer::Printer;
use state::FormatState;

// ---

type PrintError = String; // TODO: diagnostics
type PrintResult<T> = Result<T, PrintError>;
type FormatError = String; // TODO: diagnostics
type FormatResult<T> = Result<T, FormatError>;

// ---

#[inline(always)]
fn write_with_formatter(output: &mut dyn Buffer, args: Arguments) {
    let mut f = Formatter::new(output);
    f.write_fmt(args);
}

// ---

pub fn format_source(
    source_text: &str,
    source_type: SourceType,
    options: FormatOptions,
) -> FormatResult<String> {
    let allocator = Allocator::new();

    // TEXT -> AST
    let parser =
        Parser::new(&allocator, source_text, source_type).with_options(ParseOptions::default());
    let parsed = parser.parse();
    let program = parsed.program;
    let source_text = program.source_text;

    // TODO: Transform AST

    let mut state = FormatState::new(FormatContext::new(options));
    let mut buffer = VecBuffer::new(&mut state);

    // AST -> IR
    program.fmt(&mut Formatter::new(&mut buffer));

    let mut document = Document::from(buffer.into_vec());
    document.propagate_expand();

    // IR -> TEXT
    let printer = Printer::new(
        source_text,
        state.into_context().options().as_print_options(),
    );
    let printed = printer.print(&document)?;

    Ok(printed)
}
