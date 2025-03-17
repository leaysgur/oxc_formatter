//! This is a generated file. Don't modify it by hand! Run 'cargo codegen formatter' to re-generate the file.

#![expect(clippy::default_constructed_unit_structs)]

use oxc_ast::ast::*;

use crate::base_formatter::{FormatOwnedWithRule, FormatRefWithRule, FormatResult, FormatRule};
use crate::{AsFormat, FormatNodeRule, IntoFormat, JsFormatContext, JsFormatter};

impl FormatRule<Program<'_>> for crate::format::js::FormatProgram {
    type Context = JsFormatContext;
    #[inline(always)]
    fn fmt(&self, node: &Program, f: &mut JsFormatter) -> FormatResult<()> {
        FormatNodeRule::<Program>::fmt(self, node, f)
    }
}

impl AsFormat<JsFormatContext> for Program<'_> {
    type Format<'a>
        = FormatRefWithRule<'a, Program<'a>, crate::format::js::FormatProgram>
    where
        Self: 'a;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(self, crate::format::js::FormatProgram::default())
    }
}
impl<'a> IntoFormat<JsFormatContext> for Program<'a> {
    type Format = FormatOwnedWithRule<Program<'a>, crate::format::js::FormatProgram>;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(self, crate::format::js::FormatProgram::default())
    }
}
