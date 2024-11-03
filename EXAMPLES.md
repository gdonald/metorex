# Metorex Examples Implementation Plan

This document tracks the implementation work needed to execute all example files with verified output testing.

**Current Status**: 27/88 examples executing (30.7%)

**Note**: This plan orders implementation from easiest (basic features) to most complex (advanced features). Each phase builds on previous phases.

---

## Phase 1: Core String Features

### 1.1 String Interpolation
- [ ] 1.1.1 Implement string interpolation parsing (`"Hello, #{name}!"`)
- [ ] 1.1.2 Add string interpolation evaluation in VM
- [ ] 1.1.3 Enable `examples/basics/greeting_line.mx` execution
- [ ] 1.1.4 Add output capture and verification to test runner

### 1.2 String Methods
- [ ] 1.2.1 Implement basic string methods (upcase, downcase, trim, reverse)
- [ ] 1.2.2 Add string inspection methods (length, chars, bytes)
- [ ] 1.2.3 Enable string method examples execution

---

## Phase 2: Dictionary/Hash Support

### 2.1 Hash Literal Parsing
- [ ] 2.1.1 Implement hash literal parsing (`{"key" => value}`)
- [ ] 2.1.2 Add hash rocket operator (`=>`) support
- [ ] 2.1.3 Enable `examples/data-structures/simple_dict.mx` parsing

### 2.2 Hash Runtime Operations
- [ ] 2.2.1 Implement hash creation in VM
- [ ] 2.2.2 Add hash indexing operations (`hash["key"]`)
- [ ] 2.2.3 Implement hash methods (keys, values, has_key?, entries)
- [ ] 2.2.4 Enable `examples/data-structures/dict_access.mx` execution
- [ ] 2.2.5 Enable `examples/type-annotations/collection_types.mx` execution

---

## Phase 3: Range and Iterator Support

### 3.1 Range Objects
- [ ] 3.1.1 Implement range literal parsing (`1..10`, `1...10`)
- [ ] 3.1.2 Add range object creation in VM
- [ ] 3.1.3 Implement range methods (each, to_a, include?)

### 3.2 Block Iteration with each
- [ ] 3.2.1 Implement `.each do |var| ... end` parsing
- [ ] 3.2.2 Add block parameter binding in VM
- [ ] 3.2.3 Implement iterator protocol for ranges and arrays
- [ ] 3.2.4 Enable `examples/algorithms/factorial_iterative.mx` execution
- [ ] 3.2.5 Enable `examples/algorithms/average_temperature.mx` execution
- [ ] 3.2.6 Enable `examples/algorithms/primes_under_fifty.mx` execution

---

## Phase 4: Lambda and Closure Support

### 4.1 Lambda Syntax
- [ ] 4.1.1 Implement `lambda do |params| ... end` parsing
- [ ] 4.1.2 Implement arrow syntax `x -> expr` parsing
- [ ] 4.1.3 Implement multi-param lambda `(x, y) -> expr` parsing
- [ ] 4.1.4 Implement zero-param lambda `-> expr` parsing

### 4.2 Lambda Execution
- [ ] 4.2.1 Create Lambda object type in VM
- [ ] 4.2.2 Implement lambda `.call()` method
- [ ] 4.2.3 Add closure variable capture
- [ ] 4.2.4 Enable `examples/functions/closures_nested.mx` execution
- [ ] 4.2.5 Enable `examples/functions/locals_scope.mx` execution
- [ ] 4.2.6 Enable `examples/functions/nonlocal_counter.mx` execution

### 4.3 Advanced Block Support
- [ ] 4.3.1 Implement standalone blocks as expressions
- [ ] 4.3.2 Add block-to-lambda conversion
- [ ] 4.3.3 Implement method block parameters (`&block`)
- [ ] 4.3.4 Enable `examples/parser/blocks.mx` execution
- [ ] 4.3.5 Enable `examples/metaprogramming/blocks_as_objects.mx` execution

---

## Phase 5: Advanced Collection Methods

### 5.1 Higher-Order Functions
- [ ] 5.1.1 Implement `.map` for arrays
- [ ] 5.1.2 Implement `.filter` for arrays
- [ ] 5.1.3 Implement `.reduce` for arrays
- [ ] 5.1.4 Enable `examples/algorithms/filter_even_numbers.mx` execution
- [ ] 5.1.5 Enable `examples/algorithms/character_counter.mx` execution
- [ ] 5.1.6 Enable `examples/algorithms/zip_merger.mx` execution

### 5.2 Matrix and Nested Collections
- [ ] 5.2.1 Add support for nested array operations
- [ ] 5.2.2 Implement transpose and matrix operations
- [ ] 5.2.3 Enable `examples/algorithms/matrix_transpose.mx` execution

---

## Phase 6: OOP Special Methods

### 6.1 super Keyword
- [ ] 6.1.1 Implement `super` keyword parsing
- [ ] 6.1.2 Add parent method lookup in VM
- [ ] 6.1.3 Implement `super()` with argument forwarding
- [ ] 6.1.4 Enable `examples/oop/super.mx` execution
- [ ] 6.1.5 Enable `examples/oop/super_chain.mx` execution
- [ ] 6.1.6 Enable `examples/oop/test_init_param.mx` execution

### 6.2 attr_reader/attr_writer/attr_accessor
- [ ] 6.2.1 Implement `attr_reader` parsing
- [ ] 6.2.2 Implement `attr_writer` parsing
- [ ] 6.2.3 Implement `attr_accessor` parsing
- [ ] 6.2.4 Add automatic getter/setter generation in VM

### 6.3 Special String Methods
- [ ] 6.3.1 Implement `to_s` special method
- [ ] 6.3.2 Implement `inspect` special method (alias for `to_s`)
- [ ] 6.3.3 Implement automatic `to_s` calling in puts
- [ ] 6.3.4 Enable `examples/oop/special_methods.mx` execution
- [ ] 6.3.5 Enable `examples/oop/test_str.mx` execution
- [ ] 6.3.6 Enable `examples/oop/test_repr.mx` execution

### 6.4 Iterator Protocol
- [ ] 6.4.1 Implement `each` special method for custom iterators
- [ ] 6.4.2 Implement `next` method for iterators
- [ ] 6.4.3 Enable `examples/oop/test_iter.mx` execution

### 6.5 Attribute Access Methods
- [ ] 6.5.1 Implement `__getattr__` special method
- [ ] 6.5.2 Implement dynamic attribute lookup
- [ ] 6.5.3 Enable `examples/oop/test_getattr.mx` execution

---

## Phase 7: Control Flow Enhancements

### 7.1 For Loops
- [ ] 7.1.1 Implement `for var in collection` parsing
- [ ] 7.1.2 Add for loop execution in VM
- [ ] 7.1.3 Implement for loop with break/continue
- [ ] 7.1.4 Enable `examples/parser/control_flow.mx` execution (partial)

### 7.2 elsif Support
- [ ] 7.2.1 Implement `elsif` keyword parsing
- [ ] 7.2.2 Add elsif branching in VM
- [ ] 7.2.3 Enable multi-branch conditionals

### 7.3 unless Support
- [ ] 7.3.1 Implement `unless` keyword parsing
- [ ] 7.3.2 Add unless evaluation in VM

---

## Phase 8: Pattern Matching

### 8.1 Basic Match Statements
- [ ] 8.1.1 Implement `match/when` statement parsing
- [ ] 8.1.2 Add literal pattern matching in VM
- [ ] 8.1.3 Implement wildcard pattern (`_`)
- [ ] 8.1.4 Enable `examples/parser/pattern_matching.mx` execution (partial)

### 8.2 Advanced Patterns
- [ ] 8.2.1 Implement guard clauses (`when x if condition`)
- [ ] 8.2.2 Add variable binding in patterns
- [ ] 8.2.3 Implement array destructuring patterns
- [ ] 8.2.4 Implement hash destructuring patterns
- [ ] 8.2.5 Enable `examples/advanced/pattern_matching.mx` execution
- [ ] 8.2.6 Enable `examples/runtime/pattern_matching.mx` execution

### 8.3 Type Patterns
- [ ] 8.3.1 Implement type-based pattern matching
- [ ] 8.3.2 Add pattern matching for custom classes

---

## Phase 9: Exception Handling

### 9.1 Basic Exceptions
- [ ] 9.1.1 Implement `begin/rescue/end` parsing
- [ ] 9.1.2 Add exception raising in VM
- [ ] 9.1.3 Implement basic exception catching
- [ ] 9.1.4 Enable `examples/errors/basic_error.mx` execution

### 9.2 Advanced Exception Features
- [ ] 9.2.1 Implement `rescue ExceptionType => var` syntax
- [ ] 9.2.2 Add multiple rescue clauses
- [ ] 9.2.3 Implement `else` clause for rescue
- [ ] 9.2.4 Implement `ensure` clause
- [ ] 9.2.5 Implement `raise` statement
- [ ] 9.2.6 Add exception re-raising
- [ ] 9.2.7 Enable `examples/parser/exceptions.mx` execution
- [ ] 9.2.8 Enable `examples/advanced/exception_handling.mx` execution

### 9.3 Exception Types
- [ ] 9.3.1 Implement standard exception hierarchy
- [ ] 9.3.2 Add custom exception classes
- [ ] 9.3.3 Implement exception chaining

### 9.4 Error Reporting
- [ ] 9.4.1 Implement stack trace generation
- [ ] 9.4.2 Add source location tracking
- [ ] 9.4.3 Implement formatted error messages
- [ ] 9.4.4 Enable `examples/runtime/error_reporting.mx` execution

---

## Phase 10: Introspection and Reflection

### 10.1 Method Introspection
- [ ] 10.1.1 Implement `method(:name)` function
- [ ] 10.1.2 Add method object with `.name` attribute
- [ ] 10.1.3 Enable `examples/introspection/function_name.mx` execution
- [ ] 10.1.4 Enable `examples/introspection/function_module.mx` execution

### 10.2 Code Object Introspection
- [ ] 10.2.1 Implement code object access
- [ ] 10.2.2 Add bytecode/AST inspection
- [ ] 10.2.3 Enable `examples/introspection/code_object.mx` execution

### 10.3 Namespace Introspection
- [ ] 10.3.1 Implement namespace/scope inspection
- [ ] 10.3.2 Add closure variable inspection
- [ ] 10.3.3 Enable `examples/introspection/closure_namespace.mx` execution

### 10.4 Attribute Introspection
- [ ] 10.4.1 Implement attribute listing
- [ ] 10.4.2 Add dynamic attribute access
- [ ] 10.4.3 Enable `examples/introspection/basic_attributes.mx` execution
- [ ] 10.4.4 Enable `examples/introspection/annotations.mx` execution
- [ ] 10.4.5 Enable `examples/introspection/default_parameters.mx` execution

---

## Phase 11: Additional Parser Features

### 11.1 Expression Enhancements
- [ ] 11.1.1 Fix remaining expression parsing issues
- [ ] 11.1.2 Add support for complex nested expressions
- [ ] 11.1.3 Enable `examples/parser/expressions.mx` execution

### 11.2 Statement Completeness
- [ ] 11.2.1 Fix remaining statement parsing issues
- [ ] 11.2.2 Add support for all statement types
- [ ] 11.2.3 Enable `examples/parser/statements.mx` execution

### 11.3 Method Call Enhancements
- [ ] 11.3.1 Fix trailing block support in method calls
- [ ] 11.3.2 Add operator overloading support
- [ ] 11.3.3 Enable `examples/parser/method_calls.mx` execution
- [ ] 11.3.4 Enable `examples/lexer/operators.mx` execution

### 11.4 Class Parsing
- [ ] 11.4.1 Fix remaining class parsing issues
- [ ] 11.4.2 Add module/mixin support parsing
- [ ] 11.4.3 Enable `examples/parser/classes.mx` execution

### 11.5 Function Parsing
- [ ] 11.5.1 Fix remaining function parsing issues
- [ ] 11.5.2 Add keyword argument support
- [ ] 11.5.3 Enable `examples/parser/functions.mx` execution

---

## Phase 12: Lexer Completeness

### 12.1 Identifier Edge Cases
- [ ] 12.1.1 Fix identifier tokenization edge cases
- [ ] 12.1.2 Handle all variable prefix types correctly
- [ ] 12.1.3 Enable `examples/lexer/identifiers.mx` execution

---

## Phase 13: Runtime Features

### 13.1 Built-in Classes
- [ ] 13.1.1 Implement type() function
- [ ] 13.1.2 Add class hierarchy inspection
- [ ] 13.1.3 Enable `examples/runtime/builtin_classes.mx` execution
- [ ] 13.1.4 Enable `examples/runtime/types.mx` execution

### 13.2 Instance Management
- [ ] 13.2.1 Implement instance variable inspection
- [ ] 13.2.2 Add instance copying/cloning
- [ ] 13.2.3 Enable `examples/runtime/instances.mx` execution

### 13.3 Variable Scoping
- [ ] 13.3.1 Fix variable scope edge cases
- [ ] 13.3.2 Add nonlocal/global keyword support
- [ ] 13.3.3 Enable `examples/runtime/variable_scope.mx` execution

---

## Phase 14: Advanced OOP

### 14.1 Metaprogramming
- [ ] 14.1.1 Implement `define_method`
- [ ] 14.1.2 Add `method_missing`
- [ ] 14.1.3 Implement class instance variables
- [ ] 14.1.4 Enable `examples/advanced/dynamic_method_definition.mx` execution

### 14.2 Implicit Block Capture
- [ ] 14.2.1 Implement Ruby-style block capture
- [ ] 14.2.2 Add yield keyword
- [ ] 14.2.3 Enable `examples/advanced/implicit_block_capture.mx` execution

---

## Phase 15: Advanced Features

### 15.1 Traits/Interfaces
- [ ] 15.1.1 Implement trait definition syntax
- [ ] 15.1.2 Add trait implementation checking
- [ ] 15.1.3 Implement trait method requirements
- [ ] 15.1.4 Enable `examples/advanced/traits.mx` execution

### 15.2 DSL Support
- [ ] 15.2.1 Implement method chaining optimizations
- [ ] 15.2.2 Add builder pattern support
- [ ] 15.2.3 Enable `examples/advanced/dsl_example.mx` execution

### 15.3 Serialization
- [ ] 15.3.1 Verify JSON serialization works
- [ ] 15.3.2 Add additional format support
- [ ] 15.3.3 Confirm `examples/advanced/serialization.mx` execution

---

## Phase 16: Concurrency (Future)

### 16.1 Basic Concurrency
- [ ] 16.1.1 Implement thread creation
- [ ] 16.1.2 Add mutex/lock support
- [ ] 16.1.3 Implement channels
- [ ] 16.1.4 Enable `examples/advanced/concurrency.mx` execution

---

## Phase 17: Networking (Future)

### 17.1 HTTP Support
- [ ] 17.1.1 Implement HTTP client
- [ ] 17.1.2 Implement HTTP server
- [ ] 17.1.3 Add WebSocket support
- [ ] 17.1.4 Enable `examples/advanced/networking.mx` execution

---

## Phase 18: Builtins

### 18.1 Type Introspection
- [ ] 18.1.1 Implement type checking functions
- [ ] 18.1.2 Add runtime type inspection
- [ ] 18.1.3 Enable `examples/builtins/type_introspection.mx` execution

---

## Testing Strategy

For each enabled example:

1. **Parse Verification** (currently done)
   - Verify file parses without syntax errors

2. **Execution Verification** (to be added)
   - Execute the example file
   - Capture stdout/stderr output
   - Compare against expected output file

3. **Expected Output Files**
   - Create `examples/{category}/{file}.expected` files
   - Contains expected stdout for each example
   - Update test runner to compare actual vs expected

4. **Test Runner Updates**
   - Replace `parse_example()` with `execute_and_verify_example()`
   - Add output capture functionality
   - Implement output comparison with clear diffs
   - Report execution errors vs parse errors separately

---

## Implementation Priority Notes

- **Quick Wins**: Phases 1-2 (string interpolation, hashes) unlock many examples
- **High Impact**: Phase 4 (lambdas/closures) enables algorithm examples
- **Critical Path**: Phase 6 (super keyword) required for OOP examples
- **Complex**: Phases 9-10 (exceptions, introspection) are sophisticated features
- **Future Work**: Phases 16-17 (concurrency, networking) require external dependencies

---

## Success Metrics

- **Phase 1-2**: ~35 examples executable (40%)
- **Phase 1-5**: ~50 examples executable (57%)
- **Phase 1-9**: ~70 examples executable (80%)
- **All Phases**: 88 examples executable (100%)
