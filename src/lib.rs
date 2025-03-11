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

use oxc_allocator::Allocator;
use oxc_span::SourceType;

use arguments::Arguments;
use buffer::{Buffer, VecBuffer};
use context::FormatContext;
use format::Format;
use format_element::{FormatElement, document::Document};
use formatter::Formatter;
use group_id::{GroupId, UniqueGroupIdBuilder};
use options::*;
use printer::Printer;

type PrintError = String; // TODO: diagnostics
pub type PrintResult<T> = Result<T, PrintError>;

#[inline(always)]
pub fn write(output: &mut dyn Buffer, args: Arguments) {
    let mut f = Formatter::new(output);
    f.write_fmt(args);
}

// ---

pub struct FormatState {
    context: FormatContext,
    group_id_builder: UniqueGroupIdBuilder,
}
impl FormatState {
    fn new(context: FormatContext) -> Self {
        Self {
            context,
            group_id_builder: UniqueGroupIdBuilder::default(),
        }
    }

    pub fn into_context(self) -> FormatContext {
        self.context
    }

    /// Returns the context specifying how to format the current CST
    pub fn context(&self) -> &FormatContext {
        &self.context
    }

    /// Returns a mutable reference to the context
    pub fn context_mut(&mut self) -> &mut FormatContext {
        &mut self.context
    }

    /// Creates a new group id that is unique to this document.
    /// The passed debug name is used in the [std::fmt::Debug] of the document if this is a debug build.
    /// The name is unused for production builds and has no meaning on the equality of two group ids.
    pub fn group_id(&self, debug_name: &'static str) -> GroupId {
        self.group_id_builder.group_id(debug_name)
    }
}

impl std::fmt::Debug for FormatState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("FormatState")
            .field("context", &self.context)
            .finish()
    }
}

// ---

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
    Ok(printed.code)
}
