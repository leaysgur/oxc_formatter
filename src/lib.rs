mod base_formatter;
mod format;
mod context;

use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

use crate::base_formatter::format_element::tag::Label;
use crate::base_formatter::formatter::Formatter;
use crate::context::JsFormatContext;
pub use crate::context::JsFormatOptions;

/// 'ast is the lifetime of the source code (input), 'buf is the lifetime of the buffer (output)
type JsFormatter<'ast, 'buf> = Formatter<'buf, JsFormatContext<'ast>>;

/// Formats a JavaScript (and its super languages) file based on its features.
///
/// It returns a [Formatted] result, which the user can use to override a file.
pub fn format_source(
    source_text: &str,
    source_type: SourceType,
    options: JsFormatOptions,
) -> Result<String, String> {
    let allocator = Allocator::new();
    let parser =
        Parser::new(&allocator, source_text, source_type).with_options(ParseOptions::default());
    let parsed = parser.parse();

    if !parsed.errors.is_empty() {
        return Err("TODO: parse error".to_string());
    }

    // TODO: Transform AST node

    let formatted = crate::format!(
        JsFormatContext::new(source_text, options /*comments*/),
        [parsed.program]
    )
    .map_err(|_| "TODO: format error".to_string())?;

    // let context = state.into_context();
    // let comments = context.comments();

    // comments.assert_checked_all_suppressions(&root);
    // comments.assert_formatted_all_comments();

    Ok(formatted
        .print()
        .map_err(|_| "TODO: print error".to_string())?
        .into_code())
}

#[derive(Copy, Clone, Debug)]
enum JsLabels {
    MemberChain,
}

impl Label for JsLabels {
    fn id(&self) -> u64 {
        *self as u64
    }

    fn debug_name(&self) -> &'static str {
        match self {
            JsLabels::MemberChain => "MemberChain",
        }
    }
}
