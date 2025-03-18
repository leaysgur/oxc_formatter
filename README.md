# oxc_formatter

PoC implementation of formatter for OXC AST, based on `biome(_js)_formatter`.

## TODOs and concerns

- Rust's orphan rule constraints don't apply to us?
  - If so, some trait implementations might be simplified
  - But that would mean handling OXC AST nodes directly in the logic instead of intermediate structs, would that cause any problems?
- OXC's AST strictly manages lifetimes `<'a>`, but Biome's AST doesn't seem to do so
  - Even simple things result in "lifetime may not live long enough" errors, so we need to solve this...?
- ---
- How to know parent, ancestors?
  - In Biome, each node seems to know its parent, but not in OXC
  - Should we manage `AstKind` stack somewhere like the current implementation?
  - `oxc_semantic`(will introduce with `oxc_traverse` for preprocessing AST) can be used?
- ---
- How to debug?
  - At least, we may need `IRFormatContext` for `Document` to be restored
- How to track our progress?
  - Create a table and compare to Biome AST files...?
- Update all doc comments and doctests(disabled for now)
- Align `use` declaration usage
- TBD...

