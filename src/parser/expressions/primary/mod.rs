// Primary expression parsing.
//
// Implementation is split across files by category:
//   - dispatch:    `parse_primary` — the small `match` over leading tokens
//   - literals:    int/float/string/regex/bool/nil/identifiers, %w[], InterpolatedString
//   - symbols:     `:name` symbol literals (incl. operator symbols, keywords, :"...")
//   - groups:      `( ... )`, `[ ... ]`, `{ ... }` literals and parenthesised assign
//   - blocks:      `lambda { ... }`, `do ... end`, `-> { ... }` stabby lambdas
//   - keywords:    `super`, `defined?`, `yield`, leading `::`
//   - control:     `if` / `unless` / `case` expressions

mod blocks;
mod control;
mod dispatch;
mod groups;
mod keywords;
mod literals;
mod symbols;
