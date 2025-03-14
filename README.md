# oxc_formatter

PoC implementation of formatter for OXC AST, based on `biome(_js)_formatter`.

## Concerns

- How to keep `source_text`?
  - verbatim, get_lines_before, etc
- How to know parent, ancestors?
  - In Biome, each node seems to know its parent, but not in OXC
  - Should we manage `AstKind` stack somewhere like the current implementation?
  - `oxc_semantic`(will introduce with `oxc_traverse` for preprocessing AST) can be used?
- TBD...

## Migration logs

First,

- Copy biome_formatter -> base_formatter
- Copy biome_js_formatter -> .
  - Remove AST formatting parts

Then, apply following changes:

- base_formatter/mod.rs
  - Remove cfg_attr for `serde`
  - Remove derive `Deseraizable, Merge`
  - Remove `format_(range|subtree)` related
  - Export `TextSize` as `u32`
  - Remove `PrintedTokens` related
  - Remove `tracing` call
  - Remove `SourceMarker`
  - Remove `CstFormatContext`
  - Remove `FormatToken`
- base_formatter/diagnostics.rs
  - Remove cfg_attr for `serde`
  - Remove `impl Diagnostic`
  - Remove `FormatError::RangeError`
  - Remove tests
- base_formatter/macros.rs
  - Move to ./macros.rs
- base_formatter/builders.rs
  - Remove `syntax_token_cow_slice` related
  - Remove `located_token_text` related
  - Remove `source_position` of `DynamicText`
- base_formatter/format_element.rs
  - Remove `size_assert`
  - Remove `LocatedTokenText`
  - Remove `source_position` of `DynamicText`
- base_formatter/format_element/document.rs
  - Remove `IRFormatContext` related
  - Remove tests
- base_formatter/printer/mod.rs
  - Remove `tracing` call
  - Remove `(verbatim|source)_markers`
  - Update `Printed { code }` only
- base_formatter/buffer.rs
  - Remove `PreambleBuffer` related
- base_formatter/token/number.rs
  - Remove mod
- base_formatter/printed_tokens.rs
  - Remove mod
- base_formatter/comments.rs
  - Remove mod
- base_formatter/comments/*.rs
  - Remove mod
- base_formatter/trivia.rs
  - Remove mod
- base_formatter/verbatim.rs
  - Remove mod
- base_formatter/separated.rs
  - Remove mod
- base_formatter/sourcemap.rs
  - Remove mod

