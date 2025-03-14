//! This is a generated file. Don't modify it by hand! Run 'cargo codegen formatter' to re-generate the file.

#![expect(clippy::default_constructed_unit_structs)]
use crate::{
    AsFormat, FormatBogusNodeRule, FormatNodeRule, IntoFormat, JsFormatContext, JsFormatter,
};
use biome_formatter::{FormatOwnedWithRule, FormatRefWithRule, FormatResult, FormatRule};

impl FormatRule<biome_js_syntax::JsAccessorModifier>
    for crate::js::auxiliary::accessor_modifier::FormatJsAccessorModifier
{
    type Context = JsFormatContext;
    #[inline(always)]
    fn fmt(
        &self,
        node: &biome_js_syntax::JsAccessorModifier,
        f: &mut JsFormatter,
    ) -> FormatResult<()> {
        FormatNodeRule::<biome_js_syntax::JsAccessorModifier>::fmt(self, node, f)
    }
}

impl AsFormat<JsFormatContext> for biome_js_syntax::JsAccessorModifier {
    type Format<'a> = FormatRefWithRule<
        'a,
        biome_js_syntax::JsAccessorModifier,
        crate::js::auxiliary::accessor_modifier::FormatJsAccessorModifier,
    >;
    fn format(&self) -> Self::Format<'_> {
        FormatRefWithRule::new(
            self,
            crate::js::auxiliary::accessor_modifier::FormatJsAccessorModifier::default(),
        )
    }
}
impl IntoFormat<JsFormatContext> for biome_js_syntax::JsAccessorModifier {
    type Format = FormatOwnedWithRule<
        biome_js_syntax::JsAccessorModifier,
        crate::js::auxiliary::accessor_modifier::FormatJsAccessorModifier,
    >;
    fn into_format(self) -> Self::Format {
        FormatOwnedWithRule::new(
            self,
            crate::js::auxiliary::accessor_modifier::FormatJsAccessorModifier::default(),
        )
    }
}
