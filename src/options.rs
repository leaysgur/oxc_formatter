use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;

use crate::printer::PrinterOptions;

#[derive(Debug, Default, Clone)]
pub struct FormatOptions {
    /// The indent style.
    indent_style: IndentStyle,
    /// The indent width.
    indent_width: IndentWidth,
    /// The type of line ending.
    line_ending: LineEnding,
    /// What's the max width of a line. Defaults to 80.
    line_width: LineWidth,
    /// The style for quotes. Defaults to double.
    quote_style: QuoteStyle,
    /// The style for JSX quotes. Defaults to double.
    jsx_quote_style: QuoteStyle,
    /// When properties in objects are quoted. Defaults to as-needed.
    quote_properties: QuoteProperties,
    /// Print trailing commas wherever possible in multi-line comma-separated syntactic structures. Defaults to "all".
    trailing_commas: TrailingCommas,
    /// Whether the formatter prints semicolons for all statements, class members, and type members or only when necessary because of [ASI](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-automatic-semicolon-insertion).
    semicolons: Semicolons,
    /// Whether to add non-necessary parentheses to arrow functions. Defaults to "always".
    arrow_parentheses: ArrowParentheses,
    /// Whether to insert spaces around brackets in object literals. Defaults to true.
    bracket_spacing: BracketSpacing,
    /// Whether to hug the closing bracket of multiline HTML/JSX tags to the end of the last line, rather than being alone on the following line. Defaults to false.
    bracket_same_line: BracketSameLine,
    /// Attribute position style. By default auto.
    attribute_position: AttributePosition,
    /// Whether to expand object and array literals to multiple lines. Defaults to "auto".
    expand: Expand,
}

impl FormatOptions {
    pub fn as_print_options(&self) -> PrinterOptions {
        PrinterOptions {
            indent_style: self.indent_style,
            indent_width: self.indent_width,
            line_ending: self.line_ending,
            print_width: self.line_width.into(),
        }
    }

    pub fn arrow_parentheses(&self) -> ArrowParentheses {
        self.arrow_parentheses
    }

    pub fn bracket_spacing(&self) -> BracketSpacing {
        self.bracket_spacing
    }

    pub fn bracket_same_line(&self) -> BracketSameLine {
        self.bracket_same_line
    }

    pub fn quote_style(&self) -> QuoteStyle {
        self.quote_style
    }

    pub fn jsx_quote_style(&self) -> QuoteStyle {
        self.jsx_quote_style
    }

    pub fn quote_properties(&self) -> QuoteProperties {
        self.quote_properties
    }

    pub fn trailing_commas(&self) -> TrailingCommas {
        self.trailing_commas
    }

    pub fn semicolons(&self) -> Semicolons {
        self.semicolons
    }

    pub fn tab_width(&self) -> u8 {
        self.indent_width.value()
    }

    pub fn attribute_position(&self) -> AttributePosition {
        self.attribute_position
    }

    pub fn expand(&self) -> Expand {
        self.expand
    }
}

// ---

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum IndentStyle {
    #[default]
    Tab,
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LineWidth(u16);
impl Default for LineWidth {
    fn default() -> Self {
        Self(8) // TODO: Revert to 80
    }
}
impl From<LineWidth> for u16 {
    fn from(value: LineWidth) -> Self {
        value.0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PrintWidth(u32);

impl PrintWidth {
    pub fn new(width: u32) -> Self {
        Self(width)
    }
}

impl Default for PrintWidth {
    fn default() -> Self {
        LineWidth::default().into()
    }
}
impl From<LineWidth> for PrintWidth {
    fn from(width: LineWidth) -> Self {
        Self(u32::from(u16::from(width)))
    }
}

impl From<PrintWidth> for usize {
    fn from(width: PrintWidth) -> Self {
        width.0 as usize
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
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

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Quote {
    Double,
    Single,
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

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum QuoteProperties {
    #[default]
    AsNeeded,
    Preserve,
}

impl FromStr for QuoteProperties {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "as-needed" => Ok(Self::AsNeeded),
            "preserve" => Ok(Self::Preserve),
            // TODO: replace this error with a diagnostic
            _ => Err("Value not supported for QuoteProperties"),
        }
    }
}

impl fmt::Display for QuoteProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuoteProperties::AsNeeded => write!(f, "As needed"),
            QuoteProperties::Preserve => write!(f, "Preserve"),
        }
    }
}

#[derive(Clone, Copy, Default, Debug, Eq, Hash, PartialEq)]
pub enum TrailingCommas {
    /// Trailing commas wherever possible (including function parameters and calls).
    #[default]
    All,
    /// Trailing commas where valid in ES5 (objects, arrays, etc.). No trailing commas in type parameters in TypeScript.
    Es5,
    /// No trailing commas.
    None,
}

impl TrailingCommas {
    pub const fn is_es5(&self) -> bool {
        matches!(self, TrailingCommas::Es5)
    }
    pub const fn is_all(&self) -> bool {
        matches!(self, TrailingCommas::All)
    }
    pub const fn is_none(&self) -> bool {
        matches!(self, TrailingCommas::None)
    }
}

impl FromStr for TrailingCommas {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "es5" => Ok(Self::Es5),
            "all" => Ok(Self::All),
            "none" => Ok(Self::None),
            // TODO: replace this error with a diagnostic
            _ => Err("Value not supported for TrailingCommas"),
        }
    }
}

impl fmt::Display for TrailingCommas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrailingCommas::Es5 => std::write!(f, "ES5"),
            TrailingCommas::All => std::write!(f, "All"),
            TrailingCommas::None => std::write!(f, "None"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Semicolons {
    #[default]
    Always,
    AsNeeded,
}

impl Semicolons {
    pub const fn is_as_needed(&self) -> bool {
        matches!(self, Self::AsNeeded)
    }

    pub const fn is_always(&self) -> bool {
        matches!(self, Self::Always)
    }
}

impl FromStr for Semicolons {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "as-needed" => Ok(Self::AsNeeded),
            "always" => Ok(Self::Always),
            _ => Err(
                "Value not supported for Semicolons. Supported values are 'as-needed' and 'always'.",
            ),
        }
    }
}

impl fmt::Display for Semicolons {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Semicolons::AsNeeded => write!(f, "As needed"),
            Semicolons::Always => write!(f, "Always"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ArrowParentheses {
    #[default]
    Always,
    AsNeeded,
}

impl ArrowParentheses {
    pub const fn is_as_needed(&self) -> bool {
        matches!(self, Self::AsNeeded)
    }

    pub const fn is_always(&self) -> bool {
        matches!(self, Self::Always)
    }
}

// Required by [Bpaf]
impl FromStr for ArrowParentheses {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "as-needed" => Ok(Self::AsNeeded),
            "always" => Ok(Self::Always),
            _ => Err(
                "Value not supported for Arrow parentheses. Supported values are 'as-needed' and 'always'.",
            ),
        }
    }
}

impl fmt::Display for ArrowParentheses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrowParentheses::AsNeeded => write!(f, "As needed"),
            ArrowParentheses::Always => write!(f, "Always"),
        }
    }
}
