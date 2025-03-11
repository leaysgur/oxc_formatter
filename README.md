# oxc_formatter

PoC implementation of formatter for OXC AST, based on `biome(_js)_formatter`.

```
# cargo run
👀 Original code:
let a, b=1;const c =   [2,3,4]   ; call()
✨ Formatted code:
let a, b = 1;
const c = [2, 3, 4];
/* TODO */
```

## Concerns

- How to interact with `source_text`?
  - verbatim, get_lines_before, etc...
- In Biome, each node seems to know its parent, but not in OXC
  - Should we manage `AstKind` somewhere like the current implementation?
  - Should we use existing things like `oxc_semantic` (along with `oxc_traverse` for preprocessing AST)?
- `biome_formatter` and `biome_js_formatter` is separated to support languages other than JS
  - But this introduces a lot of mechanisms to avoid Rust's Orphans rule, which may not be necessary for us?

## Notable diffs

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

