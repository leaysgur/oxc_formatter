use oxc_ast::ast::*;
// use oxc_span::GetSpan;

use crate::buffer::Buffer;
use crate::builders::*;
use crate::format::Format;
use crate::formatter::Formatter;
use crate::write;

impl Format for Program<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        let Program { body, .. } = self;

        for (idx, stmt) in body.iter().enumerate() {
            if idx > 0 {
                write!(f, [hard_line_break()]);
            }

            match stmt {
                Statement::VariableDeclaration(decl) => {
                    write!(f, [text(decl.kind.as_str()), space()]);
                    decl.fmt(f);
                }
                _ => {
                    write!(
                        f,
                        [
                            text("/* TODO */"),
                            // dynamic_text(stmt.span().source_text(f.state().context().source_text)),
                        ]
                    );
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
                let sep = format_with(|f| write!(f, [text(","), space()]));
                write!(f, [sep]);
            }

            decl.fmt(f);
        }

        if f.options().semicolons().is_always() {
            write!(f, [text(";")]);
        }
    }
}

impl Format for VariableDeclarator<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        let VariableDeclarator { id, init, .. } = self;

        if let Some(name) = id.get_identifier_name().as_ref() {
            write!(f, [dynamic_text(name.as_str())]);
        }

        if let Some(init) = init {
            write!(f, [text(" = ")]);
            init.fmt(f);
        }
    }
}

impl Format for Expression<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        match self {
            Expression::NumericLiteral(num) => num.fmt(f),
            Expression::StringLiteral(num) => num.fmt(f),
            Expression::ArrayExpression(arr) => arr.fmt(f),
            _ => {
                write!(
                    f,
                    [
                        text("/* TODO */"),
                        // dynamic_text(self.span().source_text(f.source_text)),
                    ]
                );
            }
        }
    }
}

impl Format for ArrayExpression<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        let ArrayExpression { elements, .. } = self;

        write!(f, [text("[")]);
        for (idx, element) in elements.iter().enumerate() {
            if idx > 0 {
                write!(f, [text(","), space()]);
            }

            match element {
                ArrayExpressionElement::NumericLiteral(num) => num.fmt(f),
                _ => {
                    write!(
                        f,
                        [
                            text("/* TODO */"),
                            // dynamic_text(element.span().source_text(f.source_text)),
                        ]
                    );
                }
            }
        }
        write!(f, [text("]")]);
    }
}

impl Format for NumericLiteral<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        let NumericLiteral { raw, .. } = self;

        let raw = raw.expect("NumericLiteral should have a raw value");

        write!(f, [dynamic_text(raw.to_string().as_str())]);
    }
}

impl Format for StringLiteral<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        let StringLiteral { value, .. } = self;

        let quote = || {
            if f.options().quote_style().is_double() {
                text("\"")
            } else {
                text("'")
            }
        };

        write!(
            f,
            [quote(), dynamic_text(value.to_string().as_str()), quote()]
        );
    }
}
