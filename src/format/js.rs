use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::base_formatter::{Buffer, Format, FormatResult, FormatRule, builders::*};
use crate::{AsFormat, FormatNodeRule, JsFormatContext, JsFormatter, write};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatProgram;
impl FormatNodeRule<Program<'_>> for FormatProgram {
    fn fmt_fields(&self, node: &Program, f: &mut JsFormatter) -> FormatResult<()> {
        let Program { body, .. } = node;

        write!(f, [text("// TODO: Program"), hard_line_break()])?;

        for stmt in body {
            write!(f, [stmt.format()])?;
        }
        Ok(())
        // let mut join = f.join_nodes_with_hardline();
        // for stmt in body {
        //     join.entry(&(), &stmt.format());
        // }

        // join.finish()
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatStatement;
impl FormatRule<'_, Statement<'_>, JsFormatContext<'_>> for FormatStatement {
    fn fmt(&self, node: &Statement, f: &mut JsFormatter) -> FormatResult<()> {
        match node {
            Statement::VariableDeclaration(stmt) => stmt.format().fmt(f),
            _ => write!(
                f,
                [
                    text("// TODO: Statement"),
                    hard_line_break(),
                    dynamic_text(node.span().source_text(f.context().source_text()))
                ]
            ),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatVariableDeclaration;
impl FormatNodeRule<VariableDeclaration<'_>> for FormatVariableDeclaration {
    fn fmt_fields(&self, node: &VariableDeclaration, f: &mut JsFormatter) -> FormatResult<()> {
        let VariableDeclaration { kind, .. } = node;

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
