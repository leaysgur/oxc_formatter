# oxc_formatter

PoC implementation of formatter for OXC AST, based on `biome(_js)_formatter`.

## Concerns

- How to keep `source_text`?
  - for verbatim, get_lines_before, etc
- How to know parent, ancestors?
  - In Biome, each node seems to know its parent, but not in OXC
  - Should we manage `AstKind` stack somewhere like the current implementation?
  - `oxc_semantic`(will introduce with `oxc_traverse` for preprocessing AST) can be used?
- TBD...

