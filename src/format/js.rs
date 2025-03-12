use oxc_ast::ast::*;

use crate::buffer::Buffer;
use crate::builders::*;
use crate::format::FormatNode;
use crate::formatter::Formatter;
use crate::write;

// TODO: Split this into multiple files by node?

impl FormatNode for Program<'_> {
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
                    write!(f, [text("/* TODO: Statement::Xxx */")]);
                }
            }
        }
    }
}

impl FormatNode for VariableDeclaration<'_> {
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

impl FormatNode for VariableDeclarator<'_> {
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

impl FormatNode for Expression<'_> {
    fn fmt(&self, f: &mut Formatter) {
        match self {
            Expression::NumericLiteral(num) => num.fmt(f),
            Expression::StringLiteral(num) => num.fmt(f),
            Expression::ArrayExpression(arr) => arr.fmt(f),
            _ => {
                write!(f, [text("/* TODO: Expression::Xxx */")]);
            }
        }
    }
}

impl FormatNode for ArrayExpression<'_> {
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
                    write!(f, [text("/* TODO: ArrayExpressionElement::Xxx */")]);
                }
            }
        }
        write!(f, [text("]")]);
    }
}

impl FormatNode for NumericLiteral<'_> {
    fn fmt_fields(&self, f: &mut Formatter) {
        let NumericLiteral { raw, .. } = self;

        let raw = raw.expect("NumericLiteral should have a raw value");

        write!(f, [dynamic_text(raw.to_string().as_str())]);
    }
}

impl FormatNode for StringLiteral<'_> {
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
