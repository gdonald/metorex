# Case Expression Implementation Plan

## Overview

Convert the current `case` statement implementation to also work as an expression. This allows `case` to be used in expression contexts like assignments, method arguments, return values, etc.

**Current Status:**
- `case` is implemented only as `Statement::Match`
- It produces values via `ControlFlow::Return` mechanism
- Values are extracted by `execute_program` for top-level cases
- Cannot be used in expression contexts (assignments, arguments, etc.)

**Goal:**
- Support `case` as both a statement and an expression
- Allow usage like: `x = case val when 1 then "one" else "other" end`
- Maintain backward compatibility with existing statement usage

---

## Phase 1: AST Structure Changes

### 1.1 Add Case Expression Variant to AST
- [ ] 1.1.1 Add `Expression::Case` variant to `Expression` enum in [src/ast/node.rs](src/ast/node.rs)
- [ ] 1.1.2 Define `ExprMatchCase` struct for expression-based cases
  - [ ] 1.1.2.1 Include `pattern: MatchPattern`
  - [ ] 1.1.2.2 Include `guard: Option<Expression>`
  - [ ] 1.1.2.3 Include `body: Expression` (NOT `Vec<Statement>`)
  - [ ] 1.1.2.4 Include `position: Position`
- [ ] 1.1.3 Add `Expression::Case` variant with fields:
  - [ ] 1.1.3.1 `expression: Box<Expression>` - value to match against
  - [ ] 1.1.3.2 `cases: Vec<ExprMatchCase>` - when branches
  - [ ] 1.1.3.3 `else_case: Option<Box<Expression>>` - optional else branch
  - [ ] 1.1.3.4 `position: Position`
- [ ] 1.1.4 Keep existing `Statement::Match` for statement form
- [ ] 1.1.5 Document the difference between expression and statement forms

### 1.2 Consider Unified Representation (Alternative Approach)
- [ ] 1.2.1 Evaluate whether to merge `ExprMatchCase` and `MatchCase` structures
- [ ] 1.2.2 Consider using `enum CaseBody { Statements(Vec<Statement>), Expression(Box<Expression>) }`
- [ ] 1.2.3 Document decision and rationale
- [ ] 1.2.4 If unified: implement shared structure
- [ ] 1.2.5 If separate: ensure pattern matching logic can be shared

---

## Phase 2: Parser Changes for Expression Context

### 2.1 Add Case Expression Parsing
- [ ] 2.1.1 Create `parse_case_expression()` method in [src/parser/expressions/primary.rs](src/parser/expressions/primary.rs)
- [ ] 2.1.2 Add `TokenKind::Case` handling to `parse_primary()` method
- [ ] 2.1.3 Parse the value expression after `case` keyword
- [ ] 2.1.4 Skip whitespace and newlines appropriately

### 2.2 Parse When Clauses for Expressions
- [ ] 2.2.1 Loop through `when` keywords (use `TokenKind::When`)
- [ ] 2.2.2 Reuse `parse_case_pattern()` for pattern parsing (already exists in control_flow.rs)
  - [ ] 2.2.2.1 Consider moving pattern parsing to shared parser utility module
  - [ ] 2.2.2.2 Or make `parse_case_pattern()` public within parser module
- [ ] 2.2.3 Parse optional guard clause (if `TokenKind::If` follows pattern)
- [ ] 2.2.4 Handle body parsing with two syntaxes:
  - [ ] 2.2.4.1 Inline syntax: `when pattern then expression`
  - [ ] 2.2.4.2 Block syntax: `when pattern` followed by newline and expression(s)
- [ ] 2.2.5 For block syntax, parse body as single expression or implicit begin-end block
- [ ] 2.2.6 Collect all when cases into `Vec<ExprMatchCase>`

### 2.3 Parse Else Clause
- [ ] 2.3.1 Check for `TokenKind::Else` keyword
- [ ] 2.3.2 Parse else body as expression (with `then` or newline)
- [ ] 2.3.3 Store as `Option<Box<Expression>>`
- [ ] 2.3.4 Handle case where no else clause exists (should return nil if no match)

### 2.4 Parse End Keyword
- [ ] 2.4.1 Expect `TokenKind::End` to close the case expression
- [ ] 2.4.2 Return `Expression::Case` with all collected data

### 2.5 Handle Multi-line vs Single-line Syntax
- [ ] 2.5.1 Support `case x when 1 then "one" when 2 then "two" else "other" end`
- [ ] 2.5.2 Support traditional multi-line format with newlines
- [ ] 2.5.3 Ensure proper whitespace/newline handling in both modes
- [ ] 2.5.4 Write tests for both syntaxes

### 2.6 Refactor Pattern Parsing (Shared Logic)
- [ ] 2.6.1 Move `parse_case_pattern()` to shared location or make accessible to expression parser
- [ ] 2.6.2 Ensure both statement and expression parsers use same pattern logic
- [ ] 2.6.3 Test that all pattern types work in expression context:
  - [ ] 2.6.3.1 Literals (int, float, string, bool, nil)
  - [ ] 2.6.3.2 Wildcards (`_`)
  - [ ] 2.6.3.3 Variable bindings (identifiers)
  - [ ] 2.6.3.4 Type patterns (Integer, String, Hash, etc.)
  - [ ] 2.6.3.5 Array destructuring patterns
  - [ ] 2.6.3.6 Object destructuring patterns
  - [ ] 2.6.3.7 Rest patterns (`...rest`)

---

## Phase 3: VM Execution for Case Expressions

### 3.1 Add Case Expression Evaluation
- [ ] 3.1.1 Add `Expression::Case` handling in `evaluate_expression()` in [src/vm/expression.rs](src/vm/expression.rs)
- [ ] 3.1.2 Create `evaluate_case_expression()` method in VM
- [ ] 3.1.3 Place implementation in [src/vm/pattern_matching.rs](src/vm/pattern_matching.rs) (alongside execute_match)

### 3.2 Implement Case Expression Logic
- [ ] 3.2.1 Evaluate the match value expression
- [ ] 3.2.2 Iterate through each `ExprMatchCase` in order
- [ ] 3.2.3 For each case:
  - [ ] 3.2.3.1 Call `match_pattern()` (reuse existing pattern matching logic)
  - [ ] 3.2.3.2 Collect variable bindings from pattern matching
  - [ ] 3.2.3.3 If pattern matches, check guard expression (if present)
  - [ ] 3.2.3.4 Evaluate guard in new scope with pattern bindings
  - [ ] 3.2.3.5 If guard fails (not truthy), continue to next case
  - [ ] 3.2.3.6 If guard passes (or no guard), evaluate body expression
  - [ ] 3.2.3.7 Evaluate body in new scope with pattern bindings
  - [ ] 3.2.3.8 Return the resulting value immediately
- [ ] 3.2.4 If no case matches, evaluate else expression (if present)
- [ ] 3.2.5 If no case matches and no else, return `Object::Nil`
- [ ] 3.2.6 Ensure proper scope management (push/pop scopes correctly)

### 3.3 Refactor Pattern Matching (Shared Logic)
- [ ] 3.3.1 Verify `match_pattern()` can be reused for both statement and expression forms
- [ ] 3.3.2 Verify guard evaluation can be shared
- [ ] 3.3.3 Extract common logic into helper methods if needed
- [ ] 3.3.4 Ensure all pattern types work correctly:
  - [ ] 3.3.4.1 Variable binding creates correct scope entries
  - [ ] 3.3.4.2 Array destructuring with rest patterns
  - [ ] 3.3.4.3 Object destructuring with various key forms
  - [ ] 3.3.4.4 Type checking patterns

### 3.4 Handle Nested Case Expressions
- [ ] 3.4.1 Test case expressions inside other case expressions
- [ ] 3.4.2 Test case expressions inside if/unless expressions (if those exist)
- [ ] 3.4.3 Verify scope handling for nested cases
- [ ] 3.4.4 Verify guard expressions can reference outer scope variables

---

## Phase 4: Testing - Parser Tests

### 4.1 Basic Case Expression Parsing Tests
- [ ] 4.1.1 Test basic case expression with literal patterns
  ```ruby
  x = case val
  when 1
    "one"
  when 2
    "two"
  else
    "other"
  end
  ```
- [ ] 4.1.2 Test inline syntax: `x = case val when 1 then "one" when 2 then "two" else "other" end`
- [ ] 4.1.3 Test case without else clause
- [ ] 4.1.4 Test case with single when clause
- [ ] 4.1.5 Test case with multiple values in when: `when 1, 2, 3 then "small"`

### 4.2 Pattern Type Tests
- [ ] 4.2.1 Test with integer literal patterns
- [ ] 4.2.2 Test with string literal patterns
- [ ] 4.2.3 Test with boolean patterns
- [ ] 4.2.4 Test with nil pattern
- [ ] 4.2.5 Test with wildcard pattern (`_`)
- [ ] 4.2.6 Test with variable binding pattern
- [ ] 4.2.7 Test with type patterns (Integer, String, Array, Hash)
- [ ] 4.2.8 Test with array destructuring: `when [a, b] then ...`
- [ ] 4.2.9 Test with array rest pattern: `when [first, ...rest] then ...`
- [ ] 4.2.10 Test with object destructuring: `when {x: a, y: b} then ...`

### 4.3 Guard Clause Tests
- [ ] 4.3.1 Test case with guard: `when x if x > 0 then "positive"`
- [ ] 4.3.2 Test multiple guards on different when clauses
- [ ] 4.3.3 Test guard referencing pattern-bound variables
- [ ] 4.3.4 Test guard with complex expressions

### 4.4 Expression Context Tests
- [ ] 4.4.1 Test case in variable assignment
- [ ] 4.4.2 Test case in method call argument: `puts(case x when 1 then "one" end)`
- [ ] 4.4.3 Test case in array literal: `arr = [1, case x when 1 then 2 end, 3]`
- [ ] 4.4.4 Test case in hash literal value
- [ ] 4.4.5 Test case as method return value
- [ ] 4.4.6 Test case inside binary operation
- [ ] 4.4.7 Test nested case expressions

### 4.5 Error Handling Tests
- [ ] 4.5.1 Test missing `end` keyword
- [ ] 4.5.2 Test missing `when` clause
- [ ] 4.5.3 Test invalid pattern syntax
- [ ] 4.5.4 Test `else` before `when` clauses
- [ ] 4.5.5 Test multiple `else` clauses

---

## Phase 5: Testing - Execution Tests

### 5.1 Basic Execution Tests
- [ ] 5.1.1 Test case expression returns correct value for matched pattern
  ```ruby
  result = case 2
  when 1 then "one"
  when 2 then "two"
  else "other"
  end
  # result should be "two"
  ```
- [ ] 5.1.2 Test else clause is evaluated when no pattern matches
- [ ] 5.1.3 Test nil is returned when no match and no else clause
- [ ] 5.1.4 Test first matching pattern wins (order matters)
- [ ] 5.1.5 Test multiple literal patterns in one when clause

### 5.2 Pattern Matching Execution Tests
- [ ] 5.2.1 Test literal pattern matching (int, float, string, bool, nil)
- [ ] 5.2.2 Test wildcard pattern matches any value
- [ ] 5.2.3 Test variable binding captures the value
  ```ruby
  result = case [1, 2]
  when [a, b] then a + b
  end
  # result should be 3
  ```
- [ ] 5.2.4 Test type pattern matching (Integer, String, Array, Hash)
- [ ] 5.2.5 Test array destructuring with exact length match
- [ ] 5.2.6 Test array destructuring with rest pattern
  ```ruby
  result = case [1, 2, 3, 4]
  when [first, ...rest] then rest
  end
  # result should be [2, 3, 4]
  ```
- [ ] 5.2.7 Test object/hash destructuring
  ```ruby
  result = case {"x" => 10, "y" => 20}
  when {x: a, y: b} then a + b
  end
  # result should be 30
  ```

### 5.3 Guard Execution Tests
- [ ] 5.3.1 Test guard prevents match when false
  ```ruby
  result = case 5
  when x if x < 0 then "negative"
  when x if x > 0 then "positive"
  else "zero"
  end
  # result should be "positive"
  ```
- [ ] 5.3.2 Test guard has access to pattern bindings
- [ ] 5.3.3 Test guard with complex boolean expressions
- [ ] 5.3.4 Test multiple patterns with guards

### 5.4 Scope and Binding Tests
- [ ] 5.4.1 Test pattern bindings are available in body
- [ ] 5.4.2 Test pattern bindings are available in guard
- [ ] 5.4.3 Test bindings don't leak outside case expression
- [ ] 5.4.4 Test body can access outer scope variables
- [ ] 5.4.5 Test nested case expressions have separate scopes
- [ ] 5.4.6 Test shadowing of outer variables by pattern bindings

### 5.5 Complex Expression Tests
- [ ] 5.5.1 Test case expression in arithmetic: `x = 1 + (case y when 1 then 2 else 3 end)`
- [ ] 5.5.2 Test case expression in method call
- [ ] 5.5.3 Test case expression in string interpolation
- [ ] 5.5.4 Test nested case expressions
- [ ] 5.5.5 Test case expression with complex match value (method call, binary op, etc.)

### 5.6 Edge Cases
- [ ] 5.6.1 Test empty when body (should return nil)
- [ ] 5.6.2 Test case with no when clauses (only else)
- [ ] 5.6.3 Test matching against nil value
- [ ] 5.6.4 Test matching against boolean values
- [ ] 5.6.5 Test very large number of when clauses (performance)

---

## Phase 6: Integration and Examples

### 6.1 Create Example Files
- [ ] 6.1.1 Create `tests/_examples/control-flow/case_expression_basic.mx`
  - [ ] 6.1.1.1 Show basic case expression with assignment
  - [ ] 6.1.1.2 Show inline vs multi-line syntax
  - [ ] 6.1.1.3 Show use in method arguments
- [ ] 6.1.2 Create `tests/_examples/control-flow/case_expression_patterns.mx`
  - [ ] 6.1.2.1 Demonstrate all pattern types
  - [ ] 6.1.2.2 Show array destructuring examples
  - [ ] 6.1.2.3 Show object destructuring examples
- [ ] 6.1.3 Create `tests/_examples/control-flow/case_expression_guards.mx`
  - [ ] 6.1.3.1 Show guard clause usage
  - [ ] 6.1.3.2 Show guards with pattern bindings
- [ ] 6.1.4 Create `tests/_examples/control-flow/case_expression_nested.mx`
  - [ ] 6.1.4.1 Show nested case expressions
  - [ ] 6.1.4.2 Show complex expression contexts

### 6.2 Add Examples to Test Runner
- [ ] 6.2.1 Add case_expression_basic.mx to examples_runner.rs
- [ ] 6.2.2 Add case_expression_patterns.mx to examples_runner.rs
- [ ] 6.2.3 Add case_expression_guards.mx to examples_runner.rs
- [ ] 6.2.4 Add case_expression_nested.mx to examples_runner.rs
- [ ] 6.2.5 Define expected output for each example
- [ ] 6.2.6 Verify all examples pass

### 6.3 Update Existing Examples
- [ ] 6.3.1 Review existing case statement examples
- [ ] 6.3.2 Add expression usage to existing examples where appropriate
- [ ] 6.3.3 Ensure backward compatibility with statement form

---

## Phase 7: Documentation and Code Quality

### 7.1 Code Documentation
- [ ] 7.1.1 Add doc comments to `Expression::Case` variant
- [ ] 7.1.2 Add doc comments to `ExprMatchCase` struct
- [ ] 7.1.3 Add doc comments to `parse_case_expression()` method
- [ ] 7.1.4 Add doc comments to `evaluate_case_expression()` method
- [ ] 7.1.5 Document differences between statement and expression forms
- [ ] 7.1.6 Add examples to doc comments

### 7.2 Code Quality
- [ ] 7.2.1 Run `cargo fmt` to format all code
- [ ] 7.2.2 Run `cargo clippy` and fix all warnings
- [ ] 7.2.3 Ensure no clippy violations in new code
- [ ] 7.2.4 Review code for potential optimizations
- [ ] 7.2.5 Check for code duplication between statement and expression implementations
- [ ] 7.2.6 Refactor shared logic into helper functions

### 7.3 Test Coverage
- [ ] 7.3.1 Run `cargo tarpaulin --out Stdout` to check coverage
- [ ] 7.3.2 Ensure all new code paths are covered by tests
- [ ] 7.3.3 Aim for 100% coverage on new code
- [ ] 7.3.4 Add tests for any uncovered branches
- [ ] 7.3.5 Verify overall project coverage hasn't decreased

### 7.4 Final Verification
- [ ] 7.4.1 Run `cargo test` and ensure all tests pass
- [ ] 7.4.2 Run `scripts/misplaced_tests.sh` to verify no tests in implementation files
- [ ] 7.4.3 Run examples manually to verify correct behavior
- [ ] 7.4.4 Test with the original Ruby file that triggered this work: `test/runner.rb`
- [ ] 7.4.5 Verify no regressions in existing functionality

---

## Phase 8: Optional Enhancements

### 8.1 Statement-to-Expression Conversion (Optional)
- [ ] 8.1.1 Consider converting `Statement::If` to `Expression::If`
- [ ] 8.1.2 Consider converting `Statement::Unless` to `Expression::Unless`
- [ ] 8.1.3 Evaluate whether all control structures should be expressions
- [ ] 8.1.4 Document decision and rationale
- [ ] 8.1.5 If implementing: follow similar pattern as case expression

### 8.2 Performance Optimization (Optional)
- [ ] 8.2.1 Profile case expression performance
- [ ] 8.2.2 Compare with statement form performance
- [ ] 8.2.3 Optimize pattern matching if needed
- [ ] 8.2.4 Consider caching or memoization for repeated patterns
- [ ] 8.2.5 Benchmark against Ruby's case expression performance

### 8.3 Advanced Pattern Features (Optional)
- [ ] 8.3.1 Consider adding regex pattern matching: `when /pattern/ then ...`
- [ ] 8.3.2 Consider range patterns: `when 1..10 then ...`
- [ ] 8.3.3 Consider pin operator for variable matching: `when ^x then ...`
- [ ] 8.3.4 Consider nested destructuring patterns
- [ ] 8.3.5 Document any advanced features added

---

## Implementation Notes

### Key Design Decisions

1. **Separate vs Unified AST Nodes:**
   - **Option A:** Keep `Statement::Match` and add separate `Expression::Case`
     - Pros: Clear separation, easier to maintain different semantics
     - Cons: Code duplication, two implementations to maintain
   - **Option B:** Unify under one representation with flexible body type
     - Pros: Less duplication, shared logic
     - Cons: More complex type, harder to enforce correctness

2. **Body Representation:**
   - Expression form uses single `Expression` per case
   - Statement form uses `Vec<Statement>` per case
   - Consider: Should expression form support implicit `begin...end` for multi-statement bodies?

3. **Return Value Semantics:**
   - Expression form: return value of matched case body
   - Statement form: Currently returns last expression via `ControlFlow::Return`
   - Both should behave the same way from user perspective

4. **Nil Handling:**
   - When no pattern matches and no else: return `Object::Nil`
   - This matches Ruby behavior

5. **Guard Scope:**
   - Guards have access to pattern bindings
   - Guards evaluated in new scope (pushed before, popped after)
   - Same for both statement and expression forms

### Potential Challenges

1. **Parser Ambiguity:**
   - Need to distinguish when `case` starts an expression vs statement
   - Context determines parsing mode (statement vs expression)

2. **Scope Management:**
   - Ensure bindings don't leak
   - Nested cases need separate scopes
   - Guards need temporary scopes

3. **Error Messages:**
   - Provide clear errors for malformed case expressions
   - Distinguish parse errors from runtime errors

4. **Backward Compatibility:**
   - All existing case statements must continue to work
   - Tests for existing functionality must pass

5. **Performance:**
   - Pattern matching can be expensive
   - Multiple guard evaluations
   - Consider optimization for common patterns

### Success Criteria

- [ ] All existing case statement tests pass
- [ ] New case expression tests pass
- [ ] Can parse and execute: `x = case val when pattern then expr end`
- [ ] Can use case in all expression contexts
- [ ] 100% test coverage maintained
- [ ] No clippy violations
- [ ] Documentation complete
- [ ] Examples work correctly
- [ ] The original `test/runner.rb` file parses and begins executing

---

## Estimated Complexity

- **Phase 1 (AST):** Low - straightforward structure addition
- **Phase 2 (Parser):** Medium - need to handle syntax variations carefully
- **Phase 3 (VM):** Medium - can reuse pattern matching logic, but scope management is tricky
- **Phase 4 (Parser Tests):** Medium - comprehensive test suite needed
- **Phase 5 (Execution Tests):** High - many edge cases to cover
- **Phase 6 (Integration):** Low - create examples and integrate
- **Phase 7 (Quality):** Medium - ensure coverage and quality standards
- **Phase 8 (Optional):** Variable - depends on what's chosen

**Total Estimated Effort:** 2-3 days of focused development work

---

## References

### Relevant Files

- [src/ast/node.rs](src/ast/node.rs) - AST definitions
- [src/parser/expressions/primary.rs](src/parser/expressions/primary.rs) - Primary expression parsing
- [src/parser/expressions/mod.rs](src/parser/expressions/mod.rs) - Expression parsing entry points
- [src/parser/statements/control_flow.rs](src/parser/statements/control_flow.rs) - Case statement parsing
- [src/vm/pattern_matching.rs](src/vm/pattern_matching.rs) - Pattern matching execution
- [src/vm/expression.rs](src/vm/expression.rs) - Expression evaluation
- [src/vm/core.rs](src/vm/core.rs) - Main VM execution loop
- [tests/control_flow/case_parsing_tests.rs](tests/control_flow/case_parsing_tests.rs) - Existing parser tests
- [tests/control_flow/case_execution_tests.rs](tests/control_flow/case_execution_tests.rs) - Existing execution tests

### Ruby Case Expression Behavior

In Ruby, `case` is an expression that returns the value of the matched branch:

```ruby
result = case x
when 1 then "one"
when 2 then "two"
else "other"
end
```

Key behaviors to match:
- Returns value of matched when body
- Returns value of else if no match
- Returns `nil` if no match and no else
- Can be used anywhere an expression is valid
- Supports both inline (`then`) and block syntax
