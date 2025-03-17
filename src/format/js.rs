use oxc_ast::ast::*;

use crate::base_formatter::{Buffer, FormatResult, builders::*};
use crate::{FormatNodeRule, JsFormatter, write};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatProgram;
impl FormatNodeRule<Program<'_>> for FormatProgram {
    fn fmt_fields(&self, node: &Program, f: &mut JsFormatter) -> FormatResult<()> {
        let Program { span, body, .. } = node;
        let _ = body;

        write!(
            f,
            [
                text("/* TODO */"),
                hard_line_break(),
                dynamic_text(span.source_text(f.context().source_text()))
            ]
        )
    }
}
