# oxc_formatter

PoC implementation of formatter for OXC AST, based on `biome(_js)_formatter`.

## TODOs and concerns

- How to know parent, ancestors?
  - In Biome, each node seems to know its parent, but not in OXC
  - Should we manage `AstKind` stack somewhere like the current implementation?
  - `oxc_semantic`(will introduce with `oxc_traverse` for preprocessing AST) can be used?
- Rust's orphans rule is not applied for us? If so, some traits can be simplified?
  - Will handle AST nodes instead of formatted objects, but could that lead to ownership issues?
- How to debug?
  - At least, may need `IRFormatContext` for `Document` to be restored
- How to track progress?
  - Compare to Biome nodes...?
- Update all doc comments and doctests(disabled for now)
- Align `use` declaration usage
- TBD...

