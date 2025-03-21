//! Infrastructure for code formatting
//!
//! This module defines [FormatElement], an IR to format code documents and provides a mean to print
//! such a document to a string. Objects that know how to format themselves implement the [Format] trait.
//!
//! ## Formatting Traits
//!
//! * [Format]: Implemented by objects that can be formatted.
//! * [FormatRule]: Rule that knows how to format an object of another type. Necessary in the situation where
//!     it's necessary to implement [Format] on an object from another crate. This module defines the
//!     [FormatRefWithRule] and [FormatOwnedWithRule] structs to pass an item with its corresponding rule.
//! * [FormatWithRule] implemented by objects that know how to format another type. Useful for implementing
//!     some reusable formatting logic inside of this module if the type itself doesn't implement [Format]
//!
//! ## Formatting Macros
//!
//! This crate defines two macros to construct the IR. These are inspired by Rust's `fmt` macros
//! * [`format!`]: Formats a formatable object
//! * [`format_args!`]: Concatenates a sequence of Format objects.
//! * [`write!`]: Writes a sequence of formatable objects into an output buffer.

#![deny(rustdoc::broken_intra_doc_links)]

mod arguments;
mod buffer;
pub mod builders;
mod context;
pub mod diagnostics;
pub mod format_element;
mod format_extensions;
pub mod formatter;
pub mod group_id;
mod macros;
pub mod printer;
pub mod token;

use std::fmt::Debug;
use std::marker::PhantomData;

pub use arguments::{Argument, Arguments};
pub use buffer::{Buffer, BufferExtensions, VecBuffer};
pub use context::*;
pub use diagnostics::{ActualStart, FormatError, InvalidDocumentError, PrintError};
pub use format_element::FormatElement;
use format_element::document::Document;
use formatter::Formatter;
pub use group_id::GroupId;
use group_id::UniqueGroupIdBuilder;
#[cfg(debug_assertions)]
use printer::Printer;

// TODO: Remove this?
pub type TextSize = u32;

// ---

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Formatted<Context> {
    document: Document,
    context: Context,
}

impl<Context> Formatted<Context> {
    pub fn new(document: Document, context: Context) -> Self {
        Self { document, context }
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn into_document(self) -> Document {
        self.document
    }
}

impl<Context> Formatted<Context>
where
    Context: FormatContext,
{
    pub fn print(&self) -> PrintResult<Printed> {
        let print_options = self.context.options().as_print_options();

        let printed = Printer::new(print_options).print(&self.document)?;

        Ok(printed)
    }
}

pub type PrintResult<T> = Result<T, PrintError>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Printed {
    code: String,
}

impl Printed {
    pub fn new(code: String) -> Self {
        Self { code }
    }

    pub fn as_code(&self) -> &str {
        &self.code
    }

    pub fn into_code(self) -> String {
        self.code
    }
}

/// Public return type of the formatter
pub type FormatResult<F> = Result<F, FormatError>;

// ---

/// Formatting trait for types that can create a formatted representation. The `biome_formatter` equivalent
/// to [std::fmt::Display].
///
/// ## Example
/// Implementing `Format` for a custom struct
///
/// ```
/// use biome_formatter::{format, write, IndentStyle, LineWidth};
/// use biome_formatter::prelude::*;
/// use biome_rowan::TextSize;
///
/// struct Paragraph(String);
///
/// impl Format<SimpleFormatContext> for Paragraph {
///     fn fmt(&self, f: &mut Formatter<SimpleFormatContext>) -> FormatResult<()> {
///         write!(f, [
///             hard_line_break(),
///             dynamic_text(&self.0, TextSize::from(0)),
///             hard_line_break(),
///         ])
///     }
/// }
///
/// # fn main() -> FormatResult<()> {
/// let paragraph = Paragraph(String::from("test"));
/// let formatted = format!(SimpleFormatContext::default(), [paragraph])?;
///
/// assert_eq!("test\n", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
pub trait Format<'ast, Context> {
    /// Formats the object using the given formatter.
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()>;
}

impl<'ast, T, Context> Format<'ast, Context> for &T
where
    T: ?Sized + Format<'ast, Context>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        Format::fmt(&**self, f)
    }
}

impl<'ast, T, Context> Format<'ast, Context> for &mut T
where
    T: ?Sized + Format<'ast, Context>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        Format::fmt(&**self, f)
    }
}

impl<'ast, T, Context> Format<'ast, Context> for Option<T>
where
    T: Format<'ast, Context>,
{
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        match self {
            Some(value) => value.fmt(f),
            None => Ok(()),
        }
    }
}

impl<'ast, Context> Format<'ast, Context> for () {
    #[inline]
    fn fmt(&self, _: &mut Formatter<Context>) -> FormatResult<()> {
        // Intentionally left empty
        Ok(())
    }
}

/// Rule that knows how to format an object of type `T`.
///
/// Implementing [Format] on the object itself is preferred over implementing [FormatRule] but
/// this isn't possible inside of a dependent crate for external type.
///
/// For example, the `biome_js_formatter` crate isn't able to implement [Format] on `JsIfStatement`
/// because both the [Format] trait and `JsIfStatement` are external types (Rust's orphan rule).
///
/// That's why the `biome_js_formatter` crate must define a new-type that implements the formatting
/// of `JsIfStatement`.
pub trait FormatRule<'ast, T, C> {
    fn fmt(&self, item: &T, f: &mut Formatter<C>) -> FormatResult<()>;
}

/// Rule that supports customizing how it formats an object of type `T`.
pub trait FormatRuleWithOptions<'ast, T, C>: FormatRule<'ast, T, C> {
    type Options;

    /// Returns a new rule that uses the given options to format an object.
    fn with_options(self, options: Self::Options) -> Self;
}

/// Trait for an object that formats an object with a specified rule.
///
/// Gives access to the underlying item.
///
/// Useful in situation where a type itself doesn't implement [Format] (e.g. because of Rust's orphan rule)
/// but you want to implement some common formatting logic.
///
/// ## Examples
///
/// This can be useful if you want to format a `SyntaxNode` inside biome_formatter.. `SyntaxNode` doesn't implement [Format]
/// itself but the language specific crate implements `AsFormat` and `IntoFormat` for it and the returned [Format]
/// implement [FormatWithRule].
///
/// ```ignore
/// use biome_formatter::prelude::*;
/// use biome_formatter::{format, Formatted, FormatWithRule};
/// use biome_rowan::{Language, SyntaxNode};
/// fn format_node<L: Language, F: FormatWithRule<SimpleFormatContext, Item=SyntaxNode<L>>>(node: F) -> FormatResult<Formatted<SimpleFormatContext>> {
///     let formatted = format!(SimpleFormatContext::default(), [node]);
///     let syntax = node.item();
///     // Do something with syntax
///     formatted;
/// }
/// ```
pub trait FormatWithRule<'ast, Context>: Format<'ast, Context> {
    type Item;

    /// Returns the associated item
    fn item(&self) -> &Self::Item;
}

/// Formats the referenced `item` with the specified rule.
#[derive(Debug, Copy, Clone)]
pub struct FormatRefWithRule<'ast, T, R, C>
where
    R: FormatRule<'ast, T, C>,
{
    item: &'ast T,
    rule: R,
    context: PhantomData<C>,
}

impl<'ast, T, R, C> FormatRefWithRule<'ast, T, R, C>
where
    R: FormatRule<'ast, T, C>,
{
    pub fn new(item: &'ast T, rule: R) -> Self {
        Self {
            item,
            rule,
            context: PhantomData,
        }
    }
}

impl<'ast, T, R, O, C> FormatRefWithRule<'ast, T, R, C>
where
    R: FormatRuleWithOptions<'ast, T, C, Options = O>,
{
    pub fn with_options(mut self, options: O) -> Self {
        self.rule = self.rule.with_options(options);
        self
    }
}

impl<'ast, T, R, C> FormatWithRule<'ast, C> for FormatRefWithRule<'ast, T, R, C>
where
    R: FormatRule<'ast, T, C>,
{
    type Item = T;

    fn item(&self) -> &Self::Item {
        self.item
    }
}

impl<'ast, T, R, C> Format<'ast, C> for FormatRefWithRule<'ast, T, R, C>
where
    R: FormatRule<'ast, T, C>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<C>) -> FormatResult<()> {
        self.rule.fmt(self.item, f)
    }
}

/// Formats the `item` with the specified rule.
#[derive(Debug, Clone)]
pub struct FormatOwnedWithRule<'ast, T, R, C>
where
    R: FormatRule<'ast, T, C>,
{
    item: T,
    rule: R,
    context: PhantomData<(&'ast (), C)>,
}

impl<'ast, T, R, C> FormatOwnedWithRule<'ast, T, R, C>
where
    R: FormatRule<'ast, T, C>,
{
    pub fn new(item: T, rule: R) -> Self {
        Self {
            item,
            rule,
            context: PhantomData,
        }
    }

    pub fn with_item(mut self, item: T) -> Self {
        self.item = item;
        self
    }

    pub fn into_item(self) -> T {
        self.item
    }
}

impl<'ast, T, R, C> Format<'ast, C> for FormatOwnedWithRule<'ast, T, R, C>
where
    R: FormatRule<'ast, T, C>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<C>) -> FormatResult<()> {
        self.rule.fmt(&self.item, f)
    }
}

impl<'ast, T, R, O, C> FormatOwnedWithRule<'ast, T, R, C>
where
    R: FormatRuleWithOptions<'ast, T, C, Options = O>,
{
    pub fn with_options(mut self, options: O) -> Self {
        self.rule = self.rule.with_options(options);
        self
    }
}

impl<'ast, T, R, C> FormatWithRule<'ast, C> for FormatOwnedWithRule<'ast, T, R, C>
where
    R: FormatRule<'ast, T, C>,
{
    type Item = T;

    fn item(&self) -> &Self::Item {
        &self.item
    }
}

// ---

/// The `write` function takes a target buffer and an `Arguments` struct that can be precompiled with the `format_args!` macro.
///
/// The arguments will be formatted in-order into the output buffer provided.
///
/// # Examples
///
/// ```
/// use biome_formatter::prelude::*;
/// use biome_formatter::{VecBuffer, format_args, FormatState, write, Formatted};
///
/// # fn main() -> FormatResult<()> {
/// let mut state = FormatState::new(SimpleFormatContext::default());
/// let mut buffer = VecBuffer::new(&mut state);
///
/// write!(&mut buffer, [format_args!(text("Hello World"))])?;
///
/// let formatted = Formatted::new(Document::from(buffer.into_vec()), SimpleFormatContext::default());
///
/// assert_eq!("Hello World", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
///
/// Please note that using [`write!`] might be preferable. Example:
///
/// ```
/// use biome_formatter::prelude::*;
/// use biome_formatter::{VecBuffer, format_args, FormatState, write, Formatted};
///
/// # fn main() -> FormatResult<()> {
/// let mut state = FormatState::new(SimpleFormatContext::default());
/// let mut buffer = VecBuffer::new(&mut state);
///
/// write!(&mut buffer, [text("Hello World")])?;
///
/// let formatted = Formatted::new(Document::from(buffer.into_vec()), SimpleFormatContext::default());
///
/// assert_eq!("Hello World", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
///
#[inline(always)]
pub fn write<Context>(
    output: &mut dyn Buffer<Context = Context>,
    args: Arguments<Context>,
) -> FormatResult<()> {
    let mut f = Formatter::new(output);

    f.write_fmt(args)
}

/// The `format` function takes an [`Arguments`] struct and returns the resulting formatting IR.
///
/// The [`Arguments`] instance can be created with the [`format_args!`].
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use biome_formatter::prelude::*;
/// use biome_formatter::{format, format_args};
///
/// # fn main() -> FormatResult<()> {
/// let formatted = format!(SimpleFormatContext::default(), [&format_args!(text("test"))])?;
/// assert_eq!("test", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
///
/// Please note that using [`format!`] might be preferable. Example:
///
/// ```
/// use biome_formatter::prelude::*;
/// use biome_formatter::{format};
///
/// # fn main() -> FormatResult<()> {
/// let formatted = format!(SimpleFormatContext::default(), [text("test")])?;
/// assert_eq!("test", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
pub fn format<Context>(
    context: Context,
    arguments: Arguments<Context>,
) -> FormatResult<Formatted<Context>>
where
    Context: FormatContext,
{
    let mut state = FormatState::new(context);
    let mut buffer = VecBuffer::with_capacity(arguments.items().len(), &mut state);

    buffer.write_fmt(arguments)?;

    let mut document = Document::from(buffer.into_vec());
    document.propagate_expand();

    Ok(Formatted::new(document, state.into_context()))
}

// ---

/// This structure stores the state that is relevant for the formatting of the whole document.
///
/// This structure is different from [crate::base_formatter::Formatter] in that the formatting infrastructure
/// creates a new [crate::base_formatter::Formatter] for every [crate::base_formatter::write!] call, whereas this structure stays alive
/// for the whole process of formatting a root with [crate::base_formatter::format!].
pub struct FormatState<Context> {
    context: Context,
    group_id_builder: UniqueGroupIdBuilder,
}

impl<Context> std::fmt::Debug for FormatState<Context>
where
    Context: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("FormatState")
            .field("context", &self.context)
            .finish()
    }
}

impl<Context> FormatState<Context> {
    /// Creates a new state with the given language specific context
    pub fn new(context: Context) -> Self {
        Self {
            context,
            group_id_builder: Default::default(),
        }
    }

    pub fn into_context(self) -> Context {
        self.context
    }

    /// Returns the context specifying how to format the current CST
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Returns a mutable reference to the context
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    /// Creates a new group id that is unique to this document. The passed debug name is used in the
    /// [std::fmt::Debug] of the document if this is a debug build.
    /// The name is unused for production builds and has no meaning on the equality of two group ids.
    pub fn group_id(&self, debug_name: &'static str) -> GroupId {
        self.group_id_builder.group_id(debug_name)
    }
}
