use oxc_ast::ast::*;

use crate::base_formatter::{Buffer, FormatResult, builders::*};
use crate::{FormatNodeRule, JsFormatter, write};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatProgram;
impl FormatNodeRule<Program<'_>> for FormatProgram {
    fn fmt_fields(&self, node: &Program, f: &mut JsFormatter) -> FormatResult<()> {
        let Program { body, .. } = node;
        let _ = body;

        write![f, [text("TODO: Program")]]
    }
}
