use crate::printer::PrinterOptions;

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
    // /// The style for quotes. Defaults to double.
    // quote_style: QuoteStyle,

    // /// The style for JSX quotes. Defaults to double.
    // jsx_quote_style: QuoteStyle,

    // /// When properties in objects are quoted. Defaults to as-needed.
    // quote_properties: QuoteProperties,

    // /// Print trailing commas wherever possible in multi-line comma-separated syntactic structures. Defaults to "all".
    // trailing_commas: TrailingCommas,

    // /// Whether the formatter prints semicolons for all statements, class members, and type members or only when necessary because of [ASI](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-automatic-semicolon-insertion).
    // semicolons: Semicolons,

    // /// Whether to add non-necessary parentheses to arrow functions. Defaults to "always".
    // arrow_parentheses: ArrowParentheses,

    // /// Whether to insert spaces around brackets in object literals. Defaults to true.
    // bracket_spacing: BracketSpacing,

    // /// Whether to hug the closing bracket of multiline HTML/JSX tags to the end of the last line, rather than being alone on the following line. Defaults to false.
    // bracket_same_line: BracketSameLine,

    // /// Information related to the current file
    // source_type: JsFileSource,

    // /// Attribute position style. By default auto.
    // attribute_position: AttributePosition,

    // /// Whether to enforce collapsing object literals when possible. Defaults to "preserve".
    // object_wrap: ObjectWrap,
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
}
