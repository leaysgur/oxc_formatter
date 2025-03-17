//! This is a generated file. Don't modify it by hand! Run 'cargo codegen formatter' to re-generate the file.

#![expect(clippy::default_constructed_unit_structs)]

use oxc_ast::ast::*;

use crate::base_formatter::{FormatOwnedWithRule, FormatRefWithRule, FormatResult, FormatRule};
use crate::{AsFormat, FormatNodeRule, IntoFormat, JsFormatContext, JsFormatter};

impl FormatRule<Program<'_>, JsFormatContext<'_>> for crate::format::js::FormatProgram {
    #[inline(always)]
    fn fmt(&self, node: &Program, f: &mut JsFormatter) -> FormatResult<()> {
        FormatNodeRule::<Program>::fmt(self, node, f)
    }
}

impl<'ast> AsFormat<JsFormatContext<'ast>> for Program<'ast> {
    type Format<'a>
        =
        FormatRefWithRule<'a, Program<'a>, crate::format::js::FormatProgram, JsFormatContext<'ast>>
    where
        Self: 'a;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(self, crate::format::js::FormatProgram::default())
    }
}
impl<'ast> IntoFormat<JsFormatContext<'ast>> for Program<'ast> {
    type Format =
        FormatOwnedWithRule<Program<'ast>, crate::format::js::FormatProgram, JsFormatContext<'ast>>;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(self, crate::format::js::FormatProgram::default())
    }
}
