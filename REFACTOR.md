# Parser Module Refactoring Plan

## Current State
- Single file: `src/parser/mod.rs` (1057 lines)
- Contains all parser logic including token management, error handling, statement parsing, and expression parsing
- Two test files: `tests/parser_tests.rs` and `tests/parser_error_recovery_tests.rs`

## Goals
- Split the large parser module into smaller, focused modules
- Improve code organization and maintainability
- Keep the public API unchanged to avoid breaking existing code
- Maintain 100% test coverage throughout the refactoring

## Phase 1: Core Infrastructure

- [x] 1.1. Create `src/parser/token_stream.rs`
  - [x] 1.1.1. Move token navigation methods: `peek()`, `peek_ahead()`, `previous()`, `is_at_end()`, `advance()`
  - [x] 1.1.2. Move token matching methods: `check()`, `match_kind()`, `match_token()`, `expect()`
  - [x] 1.1.3. Move whitespace handling: `skip_newlines()`, `skip_comments()`, `skip_whitespace()`
  - [x] 1.1.4. Extract a `TokenStream` struct to encapsulate token navigation state

- [x] 1.2. Create `src/parser/error.rs`
  - [x] 1.2.1. Move error handling methods: `error_at_current()`, `error_at_previous()`, `report_error()`, `synchronize()`
  - [x] 1.2.2. Move position conversion: `position_to_location()`
  - [x] 1.2.3. Extract error recovery logic into dedicated functions

## Phase 2: Statement Parsing

- [ ] 2.1. Create `src/parser/statements/mod.rs`
  - [ ] 2.1.1. Move `parse_statement()` as the main entry point
  - [ ] 2.1.2. Re-export statement parsing functions

- [ ] 2.2. Create `src/parser/statements/class.rs`
  - [ ] 2.2.1. Move `parse_class_def()` function
  - [ ] 2.2.2. Include any class-specific helper functions

- [ ] 2.3. Create `src/parser/statements/function.rs`
  - [ ] 2.3.1. Move `parse_function_def()` function
  - [ ] 2.3.2. Move `parse_parameters()` function
  - [ ] 2.3.3. Include parameter parsing helpers

- [ ] 2.4. Create `src/parser/statements/control_flow.rs`
  - [ ] 2.4.1. Move `parse_if_statement()` function
  - [ ] 2.4.2. Move `parse_while_statement()` function

- [ ] 2.5. Create `src/parser/statements/exception.rs`
  - [ ] 2.5.1. Move `parse_begin_statement()` function
  - [ ] 2.5.2. Move `parse_rescue_clause()` function
  - [ ] 2.5.3. Move `parse_raise_statement()` function

## Phase 3: Expression Parsing

- [ ] 3.1. Create `src/parser/expressions/mod.rs`
  - [ ] 3.1.1. Move `parse_expression()` as the main entry point
  - [ ] 3.1.2. Move `parse_assignment()` function
  - [ ] 3.1.3. Re-export expression parsing functions

- [ ] 3.2. Create `src/parser/expressions/binary.rs`
  - [ ] 3.2.1. Move `parse_equality()` function
  - [ ] 3.2.2. Move `parse_comparison()` function
  - [ ] 3.2.3. Move `parse_term()` function (addition/subtraction)
  - [ ] 3.2.4. Move `parse_factor()` function (multiplication/division/modulo)

- [ ] 3.3. Create `src/parser/expressions/unary.rs`
  - [ ] 3.3.1. Move `parse_unary()` function

- [ ] 3.4. Create `src/parser/expressions/call.rs`
  - [ ] 3.4.1. Move `parse_call()` function
  - [ ] 3.4.2. Move `finish_call()` function
  - [ ] 3.4.3. Move `parse_arguments()` function

- [ ] 3.5. Create `src/parser/expressions/primary.rs`
  - [ ] 3.5.1. Move `parse_primary()` function
  - [ ] 3.5.2. Handle all literal parsing (int, float, string, bool, nil, arrays, dictionaries)
  - [ ] 3.5.3. Handle interpolated string parsing

## Phase 4: Main Parser Module

- [ ] 4.1. Refactor `src/parser/mod.rs`
  - [ ] 4.1.1. Keep only the main `Parser` struct definition
  - [ ] 4.1.2. Keep the `new()` constructor
  - [ ] 4.1.3. Keep the main `parse()` method
  - [ ] 4.1.4. Delegate to sub-modules for specific parsing tasks
  - [ ] 4.1.5. Re-export public types from sub-modules

- [ ] 4.2. Update module declarations
  - [ ] 4.2.1. Add `mod token_stream;`
  - [ ] 4.2.2. Add `mod error;`
  - [ ] 4.2.3. Add `mod statements;`
  - [ ] 4.2.4. Add `mod expressions;`
  - [ ] 4.2.5. Add appropriate `pub use` statements for public API

## Phase 5: Testing & Validation

- [ ] 5.1. Run all tests after each phase
  - [ ] 5.1.1. Execute `cargo test` after completing each phase
  - [ ] 5.1.2. Ensure no test failures introduced

- [ ] 5.2. Verify test coverage
  - [ ] 5.2.1. Run `cargo tarpaulin --out Stdout`
  - [ ] 5.2.2. Ensure coverage remains at 100%

- [ ] 5.3. Run code quality tools
  - [ ] 5.3.1. Execute `cargo clippy` and fix any warnings
  - [ ] 5.3.2. Execute `cargo fmt` to format code

- [ ] 5.4. Verify no misplaced tests
  - [ ] 5.4.1. Run `scripts/misplaced_tests.sh`
  - [ ] 5.4.2. Ensure tests remain in `tests/` directory

## Implementation Notes

- Each phase should be completed and tested before moving to the next
- The public API of the `Parser` struct must remain unchanged
- All methods should maintain their current signatures where exposed
- Internal helper methods can be refactored as needed
- Consider using traits to define common parsing behavior
- Maintain error handling consistency across all modules
