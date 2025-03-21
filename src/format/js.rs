use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::base_formatter::{Buffer, Format, FormatResult, builders::*};
use crate::{JsFormatContext, JsFormatter, write};

impl<'a> Format<JsFormatContext<'a>> for Program<'a> {
    fn fmt_fields(&self, f: &mut JsFormatter<'a, '_>) -> FormatResult<()> {
        let Program { body, .. } = self;

        write!(f, [text("// TODO: Program"), hard_line_break()])?;

        let mut join = f.join_nodes_with_hardline();
        for stmt in body {
            join.entry(&(), &stmt);
        }

        join.finish()
    }
}

impl<'a> Format<JsFormatContext<'a>> for Statement<'a> {
    fn fmt(&self, f: &mut JsFormatter<'a, '_>) -> FormatResult<()> {
        match self {
            Statement::VariableDeclaration(stmt) => stmt.fmt(f),
            _ => write!(
                f,
                [
                    text("// TODO: Statement"),
                    hard_line_break(),
                    dynamic_text(self.span().source_text(f.context().source_text()))
                ]
            ),
        }
    }
}

impl<'a> Format<JsFormatContext<'a>> for VariableDeclaration<'a> {
    fn fmt_fields(&self, f: &mut JsFormatter) -> FormatResult<()> {
        let VariableDeclaration { kind, .. } = self;

        write!(
            f,
            [
                text("// TODO: VariableDeclaration @"),
                text(kind.as_str()),
                hard_line_break(),
            ]
        )?;

        Ok(())
    }
}
