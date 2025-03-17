use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::base_formatter::{Buffer, FormatResult, builders::*};
use crate::{FormatNodeRule, JsFormatter, write};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatProgram;
impl FormatNodeRule<Program<'_>> for FormatProgram {
    fn fmt_fields(&self, node: &Program, f: &mut JsFormatter) -> FormatResult<()> {
        let Program { body, .. } = node;

        let mut join = f.join_nodes_with_hardline();

        for stmt in body {
            // join.entry(&(), &stmt.format());
            join.entry(
                &(),
                &format_with(|f: &mut JsFormatter| {
                    write!(
                        f,
                        [
                            text("// TODO: Statement"),
                            hard_line_break(),
                            dynamic_text(stmt.span().source_text(f.context().source_text()))
                        ]
                    )
                }),
            );
        }

        join.finish()
    }
}
