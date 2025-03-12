mod js;

// use oxc_ast::AstKind;
use oxc_span::GetSpan;

use crate::buffer::Buffer;
use crate::builders::text;
use crate::formatter::Formatter;
use crate::write;

pub trait Format {
    fn fmt(&self, f: &mut Formatter);
}

pub trait FormatNode
where
    Self: GetSpan,
{
    fn fmt(&self, f: &mut Formatter) {
        let _span = self.span();

        // if self.is_suppressed(node, f) {
        //     return write!(f, [format_suppressed_node(node.syntax())]);
        // }

        // self.fmt_leading_comments(f);
        self.fmt_node(f);
        // self.fmt_dangling_comments(f);
        // self.fmt_trailing_comments(f);
    }

    /// Formats the node without comments. Ignores any suppression comments.
    fn fmt_node(&self, f: &mut Formatter) {
        if self.needs_parentheses() {
            write!(f, [text("(")]);
            self.fmt_fields(f);
            write!(f, [text(")")]);
        } else {
            self.fmt_fields(f);
        }
    }

    fn fmt_fields(&self, _: &mut Formatter) {
        unreachable!("Should be implemented by the node");
    }

    fn needs_parentheses(&self) -> bool {
        false
    }
}
