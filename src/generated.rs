// TODO: This should be auto-generated
#![expect(clippy::default_constructed_unit_structs)]

use oxc_ast::ast::*;

use crate::base_formatter::{FormatOwnedWithRule, FormatRefWithRule, FormatResult, FormatRule};
use crate::{AsFormat, FormatNodeRule, IntoFormat, JsFormatContext, JsFormatter};

impl<'ast> FormatRule<'ast, Program<'ast>, JsFormatContext<'ast>> for crate::format::js::FormatProgram {
    #[inline(always)]
    fn fmt(&self, node: &Program, f: &mut JsFormatter) -> FormatResult<()> {
        FormatNodeRule::<Program>::fmt(self, node, f)
    }
}
impl<'ast> AsFormat<'ast, JsFormatContext<'ast>> for Program<'ast> {
    type Format<'a>
        =
        FormatRefWithRule<'a, Program<'a>, crate::format::js::FormatProgram, JsFormatContext<'a>>
    where
        Self: 'a,
        'ast: 'a;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(self, crate::format::js::FormatProgram::default())
    }
}
impl<'ast> IntoFormat<'ast, JsFormatContext<'ast>> for Program<'ast> {
    type Format = FormatOwnedWithRule<
        'ast,
        Program<'ast>,
        crate::format::js::FormatProgram,
        JsFormatContext<'ast>,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(self, crate::format::js::FormatProgram::default())
    }
}
// ---
impl<'ast> AsFormat<'ast, JsFormatContext<'ast>> for Statement<'ast> {
    type Format<'a>
        = FormatRefWithRule<
        'a,
        Statement<'a>,
        crate::format::js::FormatStatement,
        JsFormatContext<'a>,
    >
    where
        Self: 'a,
        'ast: 'a;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(self, crate::format::js::FormatStatement::default())
    }
}
// ---
impl FormatRule<'_, VariableDeclaration<'_>, JsFormatContext<'_>>
    for crate::format::js::FormatVariableDeclaration
{
    #[inline(always)]
    fn fmt(&self, node: &VariableDeclaration, f: &mut JsFormatter) -> FormatResult<()> {
        FormatNodeRule::<VariableDeclaration>::fmt(self, node, f)
    }
}
impl<'ast> AsFormat<'ast, JsFormatContext<'ast>> for VariableDeclaration<'ast> {
    type Format<'a>
        = FormatRefWithRule<
        'a,
        VariableDeclaration<'a>,
        crate::format::js::FormatVariableDeclaration,
        JsFormatContext<'a>
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::format::js::FormatVariableDeclaration::default(),
        )
    }
}
