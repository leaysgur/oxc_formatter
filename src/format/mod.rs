mod js;

use crate::formatter::Formatter;

pub trait Format {
    fn fmt(&self, f: &mut Formatter) {
        // if self.is_suppressed(node, f) {
        //     return write!(f, [format_suppressed_node(node.syntax())]);
        // }

        // self.fmt_leading_comments(f)?;
        self.fmt_node(f);
        // self.fmt_dangling_comments(f)?;
        // self.fmt_trailing_comments(f)
    }

    /// Formats the node without comments. Ignores any suppression comments.
    fn fmt_node(&self, f: &mut Formatter) {
        let _needs_parentheses = self.needs_parentheses();

        // if needs_parentheses {
        //     write!(f, [text("(")])?;
        // }

        self.fmt_fields(f);

        // if needs_parentheses {
        //     write!(f, [text(")")])?;
        // }
    }

    fn fmt_fields(&self, _: &mut Formatter) {
        unreachable!("Should be implemented ny each node")
    }

    fn needs_parentheses(&self) -> bool {
        false
    }
}
