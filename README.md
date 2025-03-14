# oxc_formatter

PoC implementation of formatter for OXC AST, based on `biome(_js)_formatter`.

> [!IMPORTANT]
> https://github.com/leaysgur/oxc_formatter/pull/1 is the most active branch for now.
> After some conversation, I realized that fundamental rework is needed...


```
# cargo run
🛠️ Format with options:
FormatOptions { indent_style: Tab, indent_width: IndentWidth(2), line_ending: Lf, line_width: LineWidth(8), quote_style: Double, jsx_quote_style: Double, quote_properties: AsNeeded, trailing_commas: All, semicolons: Always, arrow_parentheses: Always, bracket_spacing: BracketSpacing(true), bracket_same_line: BracketSameLine(false), attribute_position: Auto, expand: Auto }
👀 Original code:
let a, b='Hey';const c =   [2,3,4]   ; call()
✨ Formatted code:
let a, b = "Hey";
const c = [2, 3, 4];
/* TODO: Statement::Xxx */
```

## Concerns

- How to interact with `source_text`?
  - verbatim, get_lines_before, etc
  - `FormatState` or `FormatContext` should keep ref, but it requires many signature changes in many places...
- In Biome, each node seems to know its parent, but not in OXC
  - Should we manage `AstKind` somewhere like the current implementation?
  - Should we use existing things like `oxc_semantic` (along with `oxc_traverse` for preprocessing AST)?
- TBD...

## Notable diffs

- `biome_formatter` and `biome_js_formatter` is separated to support languages other than JS
  - This is needed to avoid Rust's Orphans rule, which may not be necessary for us?
- All of doctests are not updated yet

### /
- arguments.rs
  - Update `-> FormatResult<()>` to `-> ()`
  - ❗️ Remove tests
- buffer.rs
  - Update `-> FormatResult<()>` to `-> ()`
  - Remove `PreambleBuffer`
- builders.rs
  - Update `-> FormatResult<()>` to `-> ()`
  - Remove `SyntaxTokenCowSlice` and `LocatedTokenText` related builders

### format_element/
- tag.rs
  - Remove `TextSize` for `Verbatim`
  - Remove `serde` features for `TagKind`
- document.rs
  - Remove `Display` for `Document`
  - Remove whole `IrFormatContext` related things
  - ❗️ Remove tests
- elements.rs
  - ❗️ Remove `source_position: TextSize` of `DynamicText`
  - ❗️ Remove `LocatedTokenText`
  - Remove `static_assert!` sizes

### printer/
- call_stack.rs
  - Update `InvalidDocumentError` to `String`
- options.rs
  - Remove `From<FormatOptions>` for `PrinterOptions`
- mod.rs
  - Remove `tracing` call
  - Remove `source_position`, `source_markers` and `verbatim_markers` of `PrinterState`
  - ❗️ Ignore `source_position` in `print_text()`
    - This makes `range`, `cursor` related tests fail
  - Update `PrintError` to `String`
  - ❗️ Remove tests

