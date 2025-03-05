use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::format_element::{FormatElement, LineMode};
use crate::formatter::{Format, Formatter};

impl Format for Program<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        let Program { body, .. } = self;

        for (idx, stmt) in body.iter().enumerate() {
            if idx > 0 {
                f.write_element(FormatElement::Line(LineMode::Hard));
            }

            match stmt {
                Statement::VariableDeclaration(decl) => {
                    f.write_element(FormatElement::StaticText {
                        text: decl.kind.as_str(),
                    });
                    f.write_element(FormatElement::Space);

                    decl.fmt(f);
                }
                _ => {
                    f.write_element(FormatElement::StaticText {
                        text: "/* TODO: */",
                    });
                    f.write_element(FormatElement::DynamicText {
                        text: stmt.span().source_text(f.source_text).into(),
                    });
                }
            }
        }
    }
}

impl Format for VariableDeclaration<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        let VariableDeclaration { declarations, .. } = self;

        for (idx, decl) in declarations.iter().enumerate() {
            if idx > 0 {
                f.write_element(FormatElement::StaticText { text: "," });
                f.write_element(FormatElement::Space);
            }

            decl.fmt(f);
        }

        f.write_element(FormatElement::StaticText { text: ";" });
    }
}

impl Format for VariableDeclarator<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        let VariableDeclarator { id, init, .. } = self;

        if let Some(name) = id.get_identifier_name().as_ref() {
            f.write_element(FormatElement::DynamicText {
                text: name.as_str().into(),
            });
        }

        if let Some(init) = init {
            f.write_element(FormatElement::StaticText { text: " = " });
            init.fmt(f);
        }
    }
}

impl Format for Expression<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        match self {
            Expression::NumericLiteral(num) => num.fmt(f),
            Expression::ArrayExpression(arr) => arr.fmt(f),
            _ => {
                f.write_element(FormatElement::StaticText {
                    text: "/* TODO: */",
                });
                f.write_element(FormatElement::DynamicText {
                    text: self.span().source_text(f.source_text).into(),
                });
            }
        }
    }
}

impl Format for ArrayExpression<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        let ArrayExpression { elements, .. } = self;

        f.write_element(FormatElement::StaticText { text: "[" });
        for (idx, element) in elements.iter().enumerate() {
            if idx > 0 {
                f.write_element(FormatElement::StaticText { text: "," });
                f.write_element(FormatElement::Space);
            }

            match element {
                ArrayExpressionElement::NumericLiteral(num) => num.fmt(f),
                _ => {
                    f.write_element(FormatElement::StaticText {
                        text: "/* TODO: */",
                    });
                    f.write_element(FormatElement::DynamicText {
                        text: element.span().source_text(f.source_text).into(),
                    });
                }
            }
        }
        f.write_element(FormatElement::StaticText { text: "]" });
    }
}

impl Format for NumericLiteral<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        let NumericLiteral { value, .. } = self;

        f.write_element(FormatElement::DynamicText {
            text: value.to_string().into(),
        });
    }
}
