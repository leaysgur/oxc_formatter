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
mod builders;
pub mod diagnostics;
pub mod format_element;
mod format_extensions;
pub mod formatter;
pub mod group_id;
pub mod prelude;
pub mod printer;
mod source_map;
pub mod token;

use crate::base_formatter::formatter::Formatter;
use crate::base_formatter::group_id::UniqueGroupIdBuilder;
use crate::base_formatter::prelude::TagKind;
use std::fmt;
use std::fmt::{Debug, Display};

pub use crate::base_formatter::diagnostics::{
    ActualStart, FormatError, InvalidDocumentError, PrintError,
};
use crate::base_formatter::format_element::document::Document;
#[cfg(debug_assertions)]
use crate::base_formatter::printer::{Printer, PrinterOptions};
pub use arguments::{Argument, Arguments};
pub use buffer::{
    Buffer, BufferExtensions, BufferSnapshot, Inspect, RemoveSoftLinesBuffer, VecBuffer,
};
pub use builders::BestFitting;
pub use format_element::{FormatElement, LINE_TERMINATORS, normalize_newlines};
pub use group_id::GroupId;
pub use source_map::{TransformSourceMap, TransformSourceMapBuilder};
use std::marker::PhantomData;
use std::num::ParseIntError;
use std::str::FromStr;
use token::string::Quote;

pub type TextSize = u32;

#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq)]
pub enum IndentStyle {
    /// Tab
    #[default]
    Tab,
    /// Space
    Space,
}

impl IndentStyle {
    pub const DEFAULT_SPACES: u8 = 2;

    /// Returns `true` if this is an [IndentStyle::Tab].
    pub const fn is_tab(&self) -> bool {
        matches!(self, IndentStyle::Tab)
    }

    /// Returns `true` if this is an [IndentStyle::Space].
    pub const fn is_space(&self) -> bool {
        matches!(self, IndentStyle::Space)
    }
}

impl FromStr for IndentStyle {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tab" => Ok(Self::Tab),
            "space" => Ok(Self::Space),
            // TODO: replace this error with a diagnostic
            _ => Err("Unsupported value for this option"),
        }
    }
}

impl Display for IndentStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndentStyle::Tab => std::write!(f, "Tab"),
            IndentStyle::Space => std::write!(f, "Space"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Default)]
pub enum LineEnding {
    ///  Line Feed only (\n), common on Linux and macOS as well as inside git repos
    #[default]
    Lf,

    /// Carriage Return + Line Feed characters (\r\n), common on Windows
    Crlf,

    /// Carriage Return character only (\r), used very rarely
    Cr,
}

impl LineEnding {
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
            LineEnding::Cr => "\r",
        }
    }

    /// Returns `true` if this is a [LineEnding::Lf].
    pub const fn is_line_feed(&self) -> bool {
        matches!(self, LineEnding::Lf)
    }

    /// Returns `true` if this is a [LineEnding::Crlf].
    pub const fn is_carriage_return_line_feed(&self) -> bool {
        matches!(self, LineEnding::Crlf)
    }

    /// Returns `true` if this is a [LineEnding::Cr].
    pub const fn is_carriage_return(&self) -> bool {
        matches!(self, LineEnding::Cr)
    }
}

impl FromStr for LineEnding {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lf" => Ok(Self::Lf),
            "crlf" => Ok(Self::Crlf),
            "cr" => Ok(Self::Cr),
            // TODO: replace this error with a diagnostic
            _ => Err("Value not supported for LineEnding"),
        }
    }
}

impl std::fmt::Display for LineEnding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineEnding::Lf => std::write!(f, "LF"),
            LineEnding::Crlf => std::write!(f, "CRLF"),
            LineEnding::Cr => std::write!(f, "CR"),
        }
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct IndentWidth(u8);

impl IndentWidth {
    pub const MIN: u8 = 0;

    pub const MAX: u8 = 24;

    /// Return the numeric value for this [IndentWidth]
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl Default for IndentWidth {
    fn default() -> Self {
        Self(2)
    }
}

impl FromStr for IndentWidth {
    type Err = ParseFormatNumberError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = u8::from_str(s).map_err(ParseFormatNumberError::ParseError)?;
        let value = Self::try_from(value).map_err(ParseFormatNumberError::TryFromU8Error)?;
        Ok(value)
    }
}

impl TryFrom<u8> for IndentWidth {
    type Error = IndentWidthFromIntError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(IndentWidthFromIntError(value))
        }
    }
}

impl Display for IndentWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.value();
        f.write_str(&std::format!("{value}"))
    }
}

impl Debug for IndentWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

/// Validated value for the `line_width` formatter options
///
/// The allowed range of values is 1..=320
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct LineWidth(u16);

impl LineWidth {
    /// Minimum allowed value for a valid [LineWidth]
    pub const MIN: u16 = 1;
    /// Maximum allowed value for a valid [LineWidth]
    pub const MAX: u16 = 320;

    /// Return the numeric value for this [LineWidth]
    pub fn value(&self) -> u16 {
        self.0
    }
}

impl Default for LineWidth {
    fn default() -> Self {
        Self(80)
    }
}

impl Display for LineWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.value();
        f.write_str(&std::format!("{value}"))
    }
}

impl Debug for LineWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

/// Error type returned when parsing a [LineWidth] or [IndentWidth] from a string fails
pub enum ParseFormatNumberError {
    /// The string could not be parsed to a number
    ParseError(ParseIntError),
    /// The `u16` value of the string is not a valid [LineWidth]
    TryFromU16Error(LineWidthFromIntError),
    /// The `u8 value of the string is not a valid [IndentWidth]
    TryFromU8Error(IndentWidthFromIntError),
}

impl From<IndentWidthFromIntError> for ParseFormatNumberError {
    fn from(value: IndentWidthFromIntError) -> Self {
        Self::TryFromU8Error(value)
    }
}

impl From<LineWidthFromIntError> for ParseFormatNumberError {
    fn from(value: LineWidthFromIntError) -> Self {
        Self::TryFromU16Error(value)
    }
}

impl From<ParseIntError> for ParseFormatNumberError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseError(value)
    }
}

impl Debug for ParseFormatNumberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Display for ParseFormatNumberError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseFormatNumberError::ParseError(err) => std::fmt::Display::fmt(err, fmt),
            ParseFormatNumberError::TryFromU16Error(err) => std::fmt::Display::fmt(err, fmt),
            ParseFormatNumberError::TryFromU8Error(err) => std::fmt::Display::fmt(err, fmt),
        }
    }
}

impl TryFrom<u16> for LineWidth {
    type Error = LineWidthFromIntError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(LineWidthFromIntError(value))
        }
    }
}

impl FromStr for LineWidth {
    type Err = ParseFormatNumberError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = u16::from_str(s).map_err(ParseFormatNumberError::ParseError)?;
        let value = Self::try_from(value).map_err(ParseFormatNumberError::TryFromU16Error)?;
        Ok(value)
    }
}

/// Error type returned when converting a u16 to a [LineWidth] fails
#[derive(Clone, Copy, Debug)]
pub struct IndentWidthFromIntError(pub u8);

impl std::fmt::Display for IndentWidthFromIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "The indent width should be between {} and {}",
            LineWidth::MIN,
            LineWidth::MAX,
        )
    }
}

/// Error type returned when converting a u16 to a [LineWidth] fails
#[derive(Clone, Copy, Debug)]
pub struct LineWidthFromIntError(pub u16);

impl std::fmt::Display for LineWidthFromIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "The line width should be between {} and {}",
            LineWidth::MIN,
            LineWidth::MAX,
        )
    }
}

impl From<LineWidth> for u16 {
    fn from(value: LineWidth) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum QuoteStyle {
    #[default]
    Double,
    Single,
}

impl QuoteStyle {
    pub fn from_byte(byte: u8) -> Option<QuoteStyle> {
        match byte {
            b'"' => Some(QuoteStyle::Double),
            b'\'' => Some(QuoteStyle::Single),
            _ => None,
        }
    }

    pub fn as_char(&self) -> char {
        match self {
            QuoteStyle::Double => '"',
            QuoteStyle::Single => '\'',
        }
    }

    pub fn as_byte(&self) -> u8 {
        self.as_char() as u8
    }

    /// Returns the quote in HTML entity
    pub fn as_html_entity(&self) -> &str {
        match self {
            QuoteStyle::Double => "&quot;",
            QuoteStyle::Single => "&apos;",
        }
    }

    /// Given the current quote, it returns the other one
    pub fn other(&self) -> Self {
        match self {
            QuoteStyle::Double => QuoteStyle::Single,
            QuoteStyle::Single => QuoteStyle::Double,
        }
    }

    pub const fn is_double(&self) -> bool {
        matches!(self, Self::Double)
    }
}

impl FromStr for QuoteStyle {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "double" => Ok(Self::Double),
            "single" => Ok(Self::Single),
            // TODO: replace this error with a diagnostic
            _ => Err("Value not supported for QuoteStyle"),
        }
    }
}

impl std::fmt::Display for QuoteStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuoteStyle::Double => std::write!(f, "Double Quotes"),
            QuoteStyle::Single => std::write!(f, "Single Quotes"),
        }
    }
}

impl From<QuoteStyle> for Quote {
    fn from(quote: QuoteStyle) -> Self {
        match quote {
            QuoteStyle::Double => Self::Double,
            QuoteStyle::Single => Self::Single,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BracketSpacing(bool);

impl BracketSpacing {
    /// Return the boolean value for this [BracketSpacing]
    pub fn value(&self) -> bool {
        self.0
    }
}

impl Default for BracketSpacing {
    fn default() -> Self {
        Self(true)
    }
}

impl From<bool> for BracketSpacing {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for BracketSpacing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{:?}", self.value())
    }
}

impl FromStr for BracketSpacing {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = bool::from_str(s);

        match value {
            Ok(value) => Ok(Self(value)),
            Err(_) => Err(
                "Value not supported for BracketSpacing. Supported values are 'true' and 'false'.",
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum AttributePosition {
    #[default]
    Auto,
    Multiline,
}

impl std::fmt::Display for AttributePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributePosition::Auto => std::write!(f, "Auto"),
            AttributePosition::Multiline => std::write!(f, "Multiline"),
        }
    }
}

impl FromStr for AttributePosition {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "multiline" => Ok(Self::Multiline),
            "auto" => Ok(Self::Auto),
            _ => Err(
                "Value not supported for attribute_position. Supported values are 'auto' and 'multiline'.",
            ),
        }
    }
}

/// Put the `>` of a multi-line HTML or JSX element at the end of the last line instead of being alone on the next line (does not apply to self closing elements).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct BracketSameLine(bool);

impl BracketSameLine {
    /// Return the boolean value for this [BracketSameLine]
    pub fn value(&self) -> bool {
        self.0
    }
}

impl From<bool> for BracketSameLine {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for BracketSameLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}", self.value())
    }
}

impl FromStr for BracketSameLine {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match bool::from_str(s) {
            Ok(value) => Ok(Self(value)),
            Err(_) => Err(
                "Value not supported for BracketSameLine. Supported values are 'true' and 'false'.",
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Expand {
    /// Objects are expanded when the first property has a leading newline. Arrays are always
    /// expanded if they are shorter than the line width.
    #[default]
    Auto,
    /// Objects and arrays are always expanded.
    Always,
    /// Objects and arrays are never expanded, if they are shorter than the line width.
    Never,
}

impl FromStr for Expand {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            _ => Err(std::format!("unknown expand literal: {}", s)),
        }
    }
}

impl fmt::Display for Expand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expand::Auto => std::write!(f, "Auto"),
            Expand::Always => std::write!(f, "Always"),
            Expand::Never => std::write!(f, "Never"),
        }
    }
}

/// Context object storing data relevant when formatting an object.
pub trait FormatContext {
    type Options: FormatOptions;

    /// Returns the formatting options
    fn options(&self) -> &Self::Options;

    /// Returns [None] if the CST has not been pre-processed.
    ///
    /// Returns [Some] if the CST has been pre-processed to simplify formatting.
    /// The source map can be used to map positions of the formatted nodes back to their original
    /// source locations or to resolve the source text.
    fn source_map(&self) -> Option<&TransformSourceMap>;
}

/// Options customizing how the source code should be formatted.
///
/// **Note**: This trait should **only** contain the essential abstractions required for the printing phase.
/// For example, do not add a `fn bracket_spacing(&self) -> BracketSpacing` method here,
/// as the [BracketSpacing] option is not needed during the printing phase
/// and enforcing its implementation for all structs using this trait is unnecessary.
pub trait FormatOptions {
    /// The indent style.
    fn indent_style(&self) -> IndentStyle;

    /// The indent width.
    fn indent_width(&self) -> IndentWidth;

    /// What's the max width of a line. Defaults to 80.
    fn line_width(&self) -> LineWidth;

    /// The type of line ending.
    fn line_ending(&self) -> LineEnding;

    /// Derives the print options from the these format options
    fn as_print_options(&self) -> PrinterOptions;
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct SimpleFormatContext {
    options: SimpleFormatOptions,
}

impl SimpleFormatContext {
    pub fn new(options: SimpleFormatOptions) -> Self {
        Self { options }
    }
}

impl FormatContext for SimpleFormatContext {
    type Options = SimpleFormatOptions;

    fn options(&self) -> &Self::Options {
        &self.options
    }

    fn source_map(&self) -> Option<&TransformSourceMap> {
        None
    }
}

#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
pub struct SimpleFormatOptions {
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_width: LineWidth,
    pub line_ending: LineEnding,
}

impl FormatOptions for SimpleFormatOptions {
    fn indent_style(&self) -> IndentStyle {
        self.indent_style
    }

    fn indent_width(&self) -> IndentWidth {
        self.indent_width
    }

    fn line_width(&self) -> LineWidth {
        self.line_width
    }

    fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    fn as_print_options(&self) -> PrinterOptions {
        PrinterOptions::default()
            .with_indent_style(self.indent_style)
            .with_indent_width(self.indent_width)
            .with_print_width(self.line_width.into())
            .with_line_ending(self.line_ending)
    }
}

impl Display for SimpleFormatOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Formatted<Context> {
    document: Document,
    context: Context,
}

impl<Context> Formatted<Context> {
    pub fn new(document: Document, context: Context) -> Self {
        Self { document, context }
    }

    /// Returns the context used during formatting.
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Returns the formatted document.
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Consumes `self` and returns the formatted document.
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

        let printed = match self.context.source_map() {
            Some(source_map) => source_map.map_printed(printed),
            None => printed,
        };

        Ok(printed)
    }

    pub fn print_with_indent(&self, indent: u16) -> PrintResult<Printed> {
        let print_options = self.context.options().as_print_options();
        let printed = Printer::new(print_options).print_with_indent(&self.document, indent)?;

        let printed = match self.context.source_map() {
            Some(source_map) => source_map.map_printed(printed),
            None => printed,
        };

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

    /// Construct an empty formatter result
    pub fn new_empty() -> Self {
        Self {
            code: String::new(),
        }
    }

    /// Access the resulting code, borrowing the result
    pub fn as_code(&self) -> &str {
        &self.code
    }

    /// Access the resulting code, consuming the result
    pub fn into_code(self) -> String {
        self.code
    }
}

/// Public return type of the formatter
pub type FormatResult<F> = Result<F, FormatError>;

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
pub trait Format<Context> {
    /// Formats the object using the given formatter.
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()>;
}

impl<T, Context> Format<Context> for &T
where
    T: ?Sized + Format<Context>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        Format::fmt(&**self, f)
    }
}

impl<T, Context> Format<Context> for &mut T
where
    T: ?Sized + Format<Context>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        Format::fmt(&**self, f)
    }
}

impl<T, Context> Format<Context> for Option<T>
where
    T: Format<Context>,
{
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        match self {
            Some(value) => value.fmt(f),
            None => Ok(()),
        }
    }
}

impl<T, Context> Format<Context> for SyntaxResult<T>
where
    T: Format<Context>,
{
    fn fmt(&self, f: &mut Formatter<Context>) -> FormatResult<()> {
        match self {
            Ok(value) => value.fmt(f),
            Err(err) => Err(err.into()),
        }
    }
}

impl<Context> Format<Context> for () {
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
pub trait FormatRule<T> {
    type Context;

    fn fmt(&self, item: &T, f: &mut Formatter<Self::Context>) -> FormatResult<()>;
}

/// Default implementation for formatting a token
pub struct FormatToken<C> {
    context: PhantomData<C>,
}

impl<C> Default for FormatToken<C> {
    fn default() -> Self {
        Self {
            context: PhantomData,
        }
    }
}

impl<C> FormatRule<SyntaxToken<C::Language>> for FormatToken<C>
where
    C: CstFormatContext,
    C::Language: 'static,
{
    type Context = C;

    fn fmt(
        &self,
        token: &SyntaxToken<C::Language>,
        f: &mut Formatter<Self::Context>,
    ) -> FormatResult<()> {
        f.state_mut().track_token(token);

        crate::write!(
            f,
            [
                format_skipped_token_trivia(token),
                format_trimmed_token(token),
            ]
        )
    }
}

/// Rule that supports customizing how it formats an object of type `T`.
pub trait FormatRuleWithOptions<T>: FormatRule<T> {
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
pub trait FormatWithRule<Context>: Format<Context> {
    type Item;

    /// Returns the associated item
    fn item(&self) -> &Self::Item;
}

/// Formats the referenced `item` with the specified rule.
#[derive(Debug, Copy, Clone)]
pub struct FormatRefWithRule<'a, T, R>
where
    R: FormatRule<T>,
{
    item: &'a T,
    rule: R,
}

impl<'a, T, R> FormatRefWithRule<'a, T, R>
where
    R: FormatRule<T>,
{
    pub fn new(item: &'a T, rule: R) -> Self {
        Self { item, rule }
    }
}

impl<T, R, O> FormatRefWithRule<'_, T, R>
where
    R: FormatRuleWithOptions<T, Options = O>,
{
    pub fn with_options(mut self, options: O) -> Self {
        self.rule = self.rule.with_options(options);
        self
    }
}

impl<T, R> FormatWithRule<R::Context> for FormatRefWithRule<'_, T, R>
where
    R: FormatRule<T>,
{
    type Item = T;

    fn item(&self) -> &Self::Item {
        self.item
    }
}

impl<T, R> Format<R::Context> for FormatRefWithRule<'_, T, R>
where
    R: FormatRule<T>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<R::Context>) -> FormatResult<()> {
        self.rule.fmt(self.item, f)
    }
}

/// Formats the `item` with the specified rule.
#[derive(Debug, Clone)]
pub struct FormatOwnedWithRule<T, R>
where
    R: FormatRule<T>,
{
    item: T,
    rule: R,
}

impl<T, R> FormatOwnedWithRule<T, R>
where
    R: FormatRule<T>,
{
    pub fn new(item: T, rule: R) -> Self {
        Self { item, rule }
    }

    pub fn with_item(mut self, item: T) -> Self {
        self.item = item;
        self
    }

    pub fn into_item(self) -> T {
        self.item
    }
}

impl<T, R> Format<R::Context> for FormatOwnedWithRule<T, R>
where
    R: FormatRule<T>,
{
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<R::Context>) -> FormatResult<()> {
        self.rule.fmt(&self.item, f)
    }
}

impl<T, R, O> FormatOwnedWithRule<T, R>
where
    R: FormatRuleWithOptions<T, Options = O>,
{
    pub fn with_options(mut self, options: O) -> Self {
        self.rule = self.rule.with_options(options);
        self
    }
}

impl<T, R> FormatWithRule<R::Context> for FormatOwnedWithRule<T, R>
where
    R: FormatRule<T>,
{
    type Item = T;

    fn item(&self) -> &Self::Item {
        &self.item
    }
}

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

/// Entry point for formatting a [SyntaxNode] for a specific language.
pub trait FormatLanguage {
    type SyntaxLanguage: Language;

    /// The type of the formatting context
    type Context: CstFormatContext<Language = Self::SyntaxLanguage>;

    /// The rule type that can format a [SyntaxNode] of this language
    type FormatRule: FormatRule<SyntaxNode<Self::SyntaxLanguage>, Context = Self::Context> + Default;

    /// Performs an optional pre-processing of the tree. This can be useful to remove nodes
    /// that otherwise complicate formatting.
    ///
    /// Return [None] if the tree shouldn't be processed. Return [Some] with the transformed
    /// tree and the source map otherwise.
    fn transform(
        &self,
        _root: &SyntaxNode<Self::SyntaxLanguage>,
    ) -> Option<(SyntaxNode<Self::SyntaxLanguage>, TransformSourceMap)> {
        None
    }

    /// This is used to select appropriate "root nodes" for the
    /// range formatting process: for instance in JavaScript the function returns
    /// true for statement and declaration nodes, to ensure the entire statement
    /// gets formatted instead of the smallest sub-expression that fits the range
    fn is_range_formatting_node(&self, _node: &SyntaxNode<Self::SyntaxLanguage>) -> bool {
        true
    }

    /// Returns the formatting options
    fn options(&self) -> &<Self::Context as FormatContext>::Options;

    /// Creates the [FormatContext] with the given `source map` and `comments`
    fn create_context(
        self,
        root: &SyntaxNode<Self::SyntaxLanguage>,
        source_map: Option<TransformSourceMap>,
    ) -> Self::Context;
}

/// Formats a syntax node file based on its features.
///
/// It returns a [Formatted] result, which the user can use to override a file.
pub fn format_node<L: FormatLanguage>(
    root: &SyntaxNode<L::SyntaxLanguage>,
    language: L,
) -> FormatResult<Formatted<L::Context>> {
    let (root, source_map) = match language.transform(&root.clone()) {
        Some((transformed, source_map)) => {
            // we don't need to insert the node back if it has the same offset
            if &transformed == root {
                (transformed, Some(source_map))
            } else {
                match root
                    .ancestors()
                    // ancestors() always returns self as the first element of the iterator.
                    .skip(1)
                    .last()
                {
                    // current root node is the topmost node we don't need to insert the transformed node back
                    None => (transformed, Some(source_map)),
                    Some(top_root) => {
                        // we have to return transformed node back into subtree
                        let transformed_range = transformed.text_range_with_trivia();
                        let root_range = root.text_range_with_trivia();

                        let transformed_root = top_root
                            .replace_child(root.clone().into(), transformed.into())
                            // SAFETY: Calling `unwrap` is safe because we know that `root` is part of the `top_root` subtree.
                            .unwrap();
                        let transformed = transformed_root.covering_element(TextRange::new(
                            root_range.start(),
                            root_range.start() + transformed_range.len(),
                        ));

                        let node = match transformed {
                            NodeOrToken::Node(node) => node,
                            NodeOrToken::Token(token) => {
                                // if we get a token we need to get the parent node
                                token.parent().unwrap_or(transformed_root)
                            }
                        };

                        (node, Some(source_map))
                    }
                }
            }
        }
        None => (root.clone(), None),
    };

    let context = language.create_context(&root, source_map);
    let format_node = FormatRefWithRule::new(&root, L::FormatRule::default());

    let mut state = FormatState::new(context);
    let mut buffer = VecBuffer::new(&mut state);

    crate::write!(buffer, [format_node])?;

    let mut document = Document::from(buffer.into_vec());
    document.propagate_expand();

    let context = state.into_context();
    // let comments = context.comments();

    // comments.assert_checked_all_suppressions(&root);
    // comments.assert_formatted_all_comments();

    Ok(Formatted::new(document, context))
}

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
