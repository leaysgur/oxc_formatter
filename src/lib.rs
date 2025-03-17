mod base_formatter;
mod format;
#[rustfmt::skip]
mod generated;
mod context;

use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

use crate::base_formatter::builders::text;
use crate::base_formatter::format_element::tag::Label;
use crate::base_formatter::formatter::Formatter;
use crate::base_formatter::{Buffer, FormatResult};
use crate::context::JsFormatContext;
pub use crate::context::JsFormatOptions;

/// Used to get an object that knows how to format this object.
trait AsFormat<Context> {
    type Format<'a>: base_formatter::Format<Context>
    where
        Self: 'a;

    /// Returns an object that is able to format this object.
    fn format(&self) -> Self::Format<'_>;
}

/// Implement [AsFormat] for references to types that implement [AsFormat].
impl<T, C> AsFormat<C> for &T
where
    T: AsFormat<C>,
{
    type Format<'a>
        = T::Format<'a>
    where
        Self: 'a;

    fn format(&self) -> Self::Format<'_> {
        AsFormat::format(&**self)
    }
}

/// Implement [AsFormat] for [Option] when `T` implements [AsFormat]
///
/// Allows to call format on optional AST fields without having to unwrap the field first.
impl<T, C> AsFormat<C> for Option<T>
where
    T: AsFormat<C>,
{
    type Format<'a>
        = Option<T::Format<'a>>
    where
        Self: 'a;

    fn format(&self) -> Self::Format<'_> {
        self.as_ref().map(|value| value.format())
    }
}

/// Used to convert this object into an object that can be formatted.
///
/// The difference to [AsFormat] is that this trait takes ownership of `self`.
trait IntoFormat<Context> {
    type Format: base_formatter::Format<Context>;

    fn into_format(self) -> Self::Format;
}

/// Implement [IntoFormat] for [Option] when `T` implements [IntoFormat]
///
/// Allows to call format on optional AST fields without having to unwrap the field first.
impl<T, Context> IntoFormat<Context> for Option<T>
where
    T: IntoFormat<Context>,
{
    type Format = Option<T::Format>;

    fn into_format(self) -> Self::Format {
        self.map(IntoFormat::into_format)
    }
}

/// Formatting specific [Iterator] extensions
trait FormattedIterExt {
    /// Converts every item to an object that knows how to format it.
    fn formatted<Context>(self) -> FormattedIter<Self, Self::Item, Context>
    where
        Self: Iterator + Sized,
        Self::Item: IntoFormat<Context>,
    {
        FormattedIter {
            inner: self,
            options: std::marker::PhantomData,
        }
    }
}

impl<I> FormattedIterExt for I where I: std::iter::Iterator {}

struct FormattedIter<Iter, Item, Context>
where
    Iter: Iterator<Item = Item>,
{
    inner: Iter,
    options: std::marker::PhantomData<Context>,
}

impl<Iter, Item, Context> std::iter::Iterator for FormattedIter<Iter, Item, Context>
where
    Iter: Iterator<Item = Item>,
    Item: IntoFormat<Context>,
{
    type Item = Item::Format;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?.into_format())
    }
}

impl<Iter, Item, Context> std::iter::FusedIterator for FormattedIter<Iter, Item, Context>
where
    Iter: std::iter::FusedIterator<Item = Item>,
    Item: IntoFormat<Context>,
{
}

impl<Iter, Item, Context> std::iter::ExactSizeIterator for FormattedIter<Iter, Item, Context>
where
    Iter: Iterator<Item = Item> + std::iter::ExactSizeIterator,
    Item: IntoFormat<Context>,
{
}

type JsFormatter<'buf> = Formatter<'buf, JsFormatContext>;

/// Rule for formatting a JavaScript [AstNode].
trait FormatNodeRule<N> {
    fn fmt(&self, node: &N, f: &mut JsFormatter) -> FormatResult<()> {
        if self.is_suppressed(node, f) {
            // TODO
            // return write!(f, [format_suppressed_node(node.syntax())]);
            return write!(f, [text("TODO: suppressed node")]);
        }

        self.fmt_leading_comments(node, f)?;
        self.fmt_node(node, f)?;
        self.fmt_dangling_comments(node, f)?;
        self.fmt_trailing_comments(node, f)
    }

    /// Formats the node without comments. Ignores any suppression comments.
    fn fmt_node(&self, node: &N, f: &mut JsFormatter) -> FormatResult<()> {
        let needs_parentheses = self.needs_parentheses(node);

        if needs_parentheses {
            write!(f, [text("(")])?;
        }

        self.fmt_fields(node, f)?;

        if needs_parentheses {
            write!(f, [text(")")])?;
        }

        Ok(())
    }

    /// Formats the node's fields.
    fn fmt_fields(&self, item: &N, f: &mut JsFormatter) -> FormatResult<()>;

    /// Returns whether the node requires parens.
    fn needs_parentheses(&self, item: &N) -> bool {
        let _ = item;
        false
    }

    /// Returns `true` if the node has a suppression comment and should use the same formatting as in the source document.
    fn is_suppressed(&self, _node: &N, _f: &JsFormatter) -> bool {
        // f.context().comments().is_suppressed(node.syntax())
        false // TODO
    }

    /// Formats the [leading comments](base_formatter::comments#leading-comments) of the node.
    ///
    /// You may want to override this method if you want to manually handle the formatting of comments
    /// inside of the `fmt_fields` method or customize the formatting of the leading comments.
    fn fmt_leading_comments(&self, _node: &N, _f: &mut JsFormatter) -> FormatResult<()> {
        // format_leading_comments(node.syntax()).fmt(f)
        Ok((/* TODO */))
    }

    /// Formats the [dangling comments](base_formatter::comments#dangling-comments) of the node.
    ///
    /// You should override this method if the node handled by this rule can have dangling comments because the
    /// default implementation formats the dangling comments at the end of the node, which isn't ideal but ensures that
    /// no comments are dropped.
    ///
    /// A node can have dangling comments if all its children are tokens or if all node childrens are optional.
    fn fmt_dangling_comments(&self, _node: &N, _f: &mut JsFormatter) -> FormatResult<()> {
        // format_dangling_comments(node.syntax())
        //     .with_soft_block_indent()
        //     .fmt(f)
        Ok((/* TODO */))
    }

    /// Formats the [trailing comments](base_formatter::comments#trailing-comments) of the node.
    ///
    /// You may want to override this method if you want to manually handle the formatting of comments
    /// inside of the `fmt_fields` method or customize the formatting of the trailing comments.
    fn fmt_trailing_comments(&self, _node: &N, _f: &mut JsFormatter) -> FormatResult<()> {
        // format_trailing_comments(node.syntax()).fmt(f)
        Ok((/* TODO */))
    }
}

/// Formats a JavaScript (and its super languages) file based on its features.
///
/// It returns a [Formatted] result, which the user can use to override a file.
pub fn format_source(
    source_text: &str,
    source_type: SourceType,
    options: JsFormatOptions,
) -> Result<String, String> {
    let allocator = Allocator::new();
    let parser =
        Parser::new(&allocator, source_text, source_type).with_options(ParseOptions::default());
    let parsed = parser.parse();

    if !parsed.errors.is_empty() {
        return Err("TODO: parse error".to_string());
    }

    // TODO: Transform AST node

    let context = JsFormatContext::new(options /*comments*/);
    let formatted = crate::format!(context, [parsed.program.format()])
        .map_err(|_| "TODO: format error".to_string())?;

    // let context = state.into_context();
    // let comments = context.comments();

    // comments.assert_checked_all_suppressions(&root);
    // comments.assert_formatted_all_comments();

    Ok(formatted
        .print()
        .map_err(|_| "TODO: print error".to_string())?
        .into_code())
}

#[derive(Copy, Clone, Debug)]
enum JsLabels {
    MemberChain,
}

impl Label for JsLabels {
    fn id(&self) -> u64 {
        *self as u64
    }

    fn debug_name(&self) -> &'static str {
        match self {
            JsLabels::MemberChain => "MemberChain",
        }
    }
}
