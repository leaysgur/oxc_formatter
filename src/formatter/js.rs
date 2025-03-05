use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::format_element::FormatElement;
use crate::formatter::{Format, Formatter};

impl Format for Program<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        for stmt in &self.body {
            match stmt {
                Statement::VariableDeclaration(decl) => {
                    f.write_element(FormatElement::StaticText {
                        text: decl.kind.as_str(),
                    });
                    f.write_element(FormatElement::Space);
                    for decl in &decl.declarations {
                        decl.fmt(f);
                    }
                }
                _ => {
                    f.write_element(FormatElement::DynamicText {
                        text: stmt.span().source_text(f.source_text).into(),
                    });
                }
            }
        }
    }
}

impl Format for VariableDeclarator<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        if let Some(name) = self.id.get_identifier_name().as_ref() {
            f.write_element(FormatElement::DynamicText {
                text: name.as_str().into(),
            });
            f.write_element(FormatElement::Space);
        }
    }
}
