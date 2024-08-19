# VM Refactoring Plan

## Overview
The `src/vm.rs` file is currently **2177 lines** and contains too many responsibilities. This document outlines a plan to refactor it into a more modular, maintainable structure by splitting it into multiple focused modules.

## Current Structure Analysis

### Major Components in `src/vm.rs`:
1. **Core VM structures** (lines ~1-170):
   - `Heap` - Memory management stub
   - `CallFrame` - Call stack tracking
   - `GlobalRegistry` - Global object registry
   - `VirtualMachine` - Main VM struct with ~1800 lines of implementation

2. **Statement execution** (lines ~242-728):
   - `execute_statement()` - Statement dispatcher
   - `execute_block()`, `execute_if()`, `execute_while()`, `execute_for()`
   - `execute_class_def()`, `execute_match()`
   - `execute_raise()`, `execute_begin()` - Exception handling
   - Pattern matching logic (`match_pattern()`, `match_array_pattern()`, `match_object_pattern()`)

3. **Expression evaluation** (lines ~980-1390):
   - `evaluate_expression()` - Expression dispatcher
   - `evaluate_interpolated_string()`
   - `evaluate_unary_operation()`, `evaluate_binary_operation()`
   - `evaluate_addition()`, `evaluate_numeric_binary()`, `evaluate_comparison()`
   - `evaluate_array_literal()`, `evaluate_dictionary_literal()`
   - `evaluate_index_operation()`

4. **Method invocation** (lines ~1418-1886):
   - `evaluate_method_call()`
   - `lookup_method()`
   - `invoke_callable()`, `invoke_method()`
   - `execute_block_body()`, `execute_method_body()`
   - `call_native_method()` - Large match for all built-in methods

5. **Helper functions** (lines ~1939-2177):
   - Initialization functions (`initialize_builtin_methods()`, `register_builtin_classes()`, etc.)
   - `ControlFlow` enum
   - Error construction functions (15+ error helper functions)
   - Utility functions (`is_truthy()`, `object_to_dict_key()`, `position_to_location()`)

## Refactoring Plan

### Phase 1: Extract Support Structures
**Priority: HIGH** - **STATUS: COMPLETED**

- [x] 1.1. Create `src/vm/mod.rs` as the main VM module entry point
- [x] 1.2. Create `src/vm/heap.rs`
  - [x] 1.2.1. Move `Heap` struct and implementation
  - [x] 1.2.2. Add proper documentation for future GC integration
  - [x] 1.2.3. Update imports in dependent files

- [x] 1.3. Create `src/vm/call_frame.rs`
  - [x] 1.3.1. Move `CallFrame` struct and implementation
  - [x] 1.3.2. Update imports in dependent files

- [x] 1.4. Create `src/vm/global_registry.rs`
  - [x] 1.4.1. Move `GlobalRegistry` struct and implementation
  - [x] 1.4.2. Update imports in dependent files

- [x] 1.5. Create `src/vm/control_flow.rs`
  - [x] 1.5.1. Move `ControlFlow` enum
  - [x] 1.5.2. Add helper methods for control flow handling (deferred - no helpers needed yet)
  - [x] 1.5.3. Update imports in dependent files

**Results**: Extracted 4 modules (~138 lines total), reduced core VM from 2177 to 2080 lines. All tests pass.

### Phase 2: Extract Error Handling
**Priority: HIGH** - **STATUS: COMPLETED**

- [x] 2.1. Create `src/vm/errors.rs`
  - [x] 2.1.1. Move all error construction functions:
    - `loop_control_error()`
    - `invalid_assignment_target_error()`
    - `undefined_self_error()`
    - `undefined_method_error()`
    - `method_argument_error()`
    - `method_argument_type_error()`
    - `not_callable_error()`
    - `callable_argument_error()`
    - `undefined_variable_error()`
    - `unimplemented_statement_error()`
    - `unary_type_error()`
    - `binary_type_error()`
    - `divide_by_zero_error()`
    - `index_out_of_bounds_error()`
    - `undefined_dictionary_key_error()`
    - `position_to_location()` (helper)
  - [x] 2.1.2. Group related errors together with module organization
  - [x] 2.1.3. Consider using a trait-based approach for extensibility (deferred - not needed yet)
  - [x] 2.1.4. Update imports in dependent files

**Results**: Extracted errors module (210 lines), reduced core VM from 2080 to 1912 lines. All tests pass.

### Phase 3: Extract Utility Functions
**Priority: MEDIUM** - **STATUS: COMPLETED**

- [x] 3.1. Create `src/vm/utils.rs`
  - [x] 3.1.1. Move `position_to_location()` (from errors.rs)
  - [x] 3.1.2. Move `format_exception()`
  - [x] 3.1.3. Move `is_truthy()`
  - [x] 3.1.4. Move `object_to_dict_key()`
  - [x] 3.1.5. Update imports in dependent files (core.rs and errors.rs)

**Results**: Extracted utils module (41 lines), reduced core VM from 1912 to 1884 lines. Refactored errors.rs to use shared utils. All tests pass.

### Phase 4: Extract Expression Evaluation
**Priority: HIGH** - **STATUS: COMPLETED**

- [x] 4.1. Create `src/vm/expression.rs`
  - [x] 4.1.1. Move core evaluation logic:
    - `evaluate_interpolated_string()`
    - `evaluate_array_literal()`
    - `evaluate_dictionary_literal()`
    - `evaluate_index_operation()`
  - [x] 4.1.2. Update to use `&mut VirtualMachine` as context
  - [x] 4.1.3. Write tests for expression evaluation in isolation (existing tests cover this)
  - [x] 4.1.4. Update imports in dependent files

- [x] 4.2. Create `src/vm/operators.rs`
  - [x] 4.2.1. Move operator evaluation logic:
    - `evaluate_unary_operation()`
    - `evaluate_binary_operation()`
    - `evaluate_addition()`
    - `evaluate_numeric_binary()`
    - `evaluate_comparison()`
  - [x] 4.2.2. Update to use `&mut VirtualMachine` as context
  - [x] 4.2.3. Write tests for operator evaluation (existing tests cover this)
  - [x] 4.2.4. Update imports in dependent files

**Results**: Extracted 2 modules (expression.rs: 125 lines, operators.rs: 207 lines), reduced core VM from 1884 to 1596 lines. All tests pass. Test coverage maintained at 67.03%.

### Phase 5: Extract Statement Execution
**Priority: HIGH**

- [ ] 5.1. Create `src/vm/statement.rs`
  - [ ] 5.1.1. Move statement execution logic:
    - `execute_statement()` - main dispatcher
    - `execute_block()`
    - `execute_statements_internal()`
    - `assign_value()`
  - [ ] 5.1.2. Update to use `&mut VirtualMachine` as context
  - [ ] 5.1.3. Write tests for statement execution
  - [ ] 5.1.4. Update imports in dependent files

- [ ] 5.2. Create `src/vm/control_structures.rs`
  - [ ] 5.2.1. Move control structure execution:
    - `execute_if()`
    - `execute_while()`
    - `execute_for()`
  - [ ] 5.2.2. Update to use `&mut VirtualMachine` as context
  - [ ] 5.2.3. Write tests for control structures
  - [ ] 5.2.4. Update imports in dependent files

- [ ] 5.3. Create `src/vm/class_execution.rs`
  - [ ] 5.3.1. Move class-related execution:
    - `execute_class_def()`
  - [ ] 5.3.2. Update to use `&mut VirtualMachine` as context
  - [ ] 5.3.3. Write tests for class definition execution
  - [ ] 5.3.4. Update imports in dependent files

### Phase 6: Extract Exception Handling
**Priority: HIGH**

- [ ] 6.1. Create `src/vm/exceptions.rs`
  - [ ] 6.1.1. Move exception-related execution:
    - `execute_raise()`
    - `execute_begin()`
    - `exception_matches()`
    - `is_class_or_subclass()`
  - [ ] 6.1.2. Update to use `&mut VirtualMachine` as context
  - [ ] 6.1.3. Write tests for exception handling
  - [ ] 6.1.4. Update imports in dependent files

### Phase 7: Extract Pattern Matching
**Priority: MEDIUM**

- [ ] 7.1. Create `src/vm/pattern_matching.rs`
  - [ ] 7.1.1. Move pattern matching logic:
    - `execute_match()`
    - `match_pattern()`
    - `match_array_pattern()`
    - `match_object_pattern()`
  - [ ] 7.1.2. Update to use `&mut VirtualMachine` as context
  - [ ] 7.1.3. Write tests for pattern matching
  - [ ] 7.1.4. Update imports in dependent files

### Phase 8: Extract Method Invocation
**Priority: CRITICAL** (largest section)

- [ ] 8.1. Create `src/vm/method_lookup.rs`
  - [ ] 8.1.1. Move method lookup and dispatch:
    - `lookup_method()`
    - `evaluate_method_call()`
  - [ ] 8.1.2. Update to use `&mut VirtualMachine` as context
  - [ ] 8.1.3. Write tests for method lookup
  - [ ] 8.1.4. Update imports in dependent files

- [ ] 8.2. Create `src/vm/method_invocation.rs`
  - [ ] 8.2.1. Move method invocation logic:
    - `invoke_method()`
    - `execute_method_body()`
    - `invoke_callable()`
    - `execute_block_body()`
  - [ ] 8.2.2. Update to use `&mut VirtualMachine` as context
  - [ ] 8.2.3. Write tests for method invocation
  - [ ] 8.2.4. Update imports in dependent files

- [ ] 8.3. Consider further splitting `src/vm/native_methods.rs`
  - [ ] 8.3.1. Move `call_native_method()` - this is a 200+ line match statement
  - [ ] 8.3.2. Consider splitting by class (object_methods, string_methods, array_methods, etc.)
  - [ ] 8.3.3. Update to use `&mut VirtualMachine` as context
  - [ ] 8.3.4. Write tests for native method calls
  - [ ] 8.3.5. Update imports in dependent files

### Phase 9: Refactor Core VM
**Priority: MEDIUM**

- [ ] 9.1. Update `src/vm/core.rs` (or keep as `src/vm/mod.rs`)
  - [ ] 9.1.1. Keep only core `VirtualMachine` struct
  - [ ] 9.1.2. Keep initialization logic (`new()`, `Default` impl)
  - [ ] 9.1.3. Keep accessor methods
  - [ ] 9.1.4. Keep `execute_program()` as the main entry point
  - [ ] 9.1.5. Delegate to extracted modules for specific operations
  - [ ] 9.1.6. Update all imports to use new module structure

- [ ] 9.2. Create `src/vm/init.rs`
  - [ ] 9.2.1. Move initialization helpers:
    - `initialize_builtin_methods()`
    - `register_builtin_classes()`
    - `register_singletons()`
    - `seed_environment_with_globals()`
  - [ ] 9.2.2. Update imports in dependent files

### Phase 10: Documentation and Testing
**Priority: HIGH**

- [ ] 10.1. Add module-level documentation
  - [ ] 10.1.1. Document `src/vm/mod.rs` with overview of VM architecture
  - [ ] 10.1.2. Add doc comments to each submodule explaining its purpose
  - [ ] 10.1.3. Add examples where appropriate

- [ ] 10.2. Ensure test coverage
  - [ ] 10.2.1. Run `cargo tarpaulin --out Stdout` to measure coverage
  - [ ] 10.2.2. Add unit tests for each extracted module
  - [ ] 10.2.3. Ensure no regression in existing integration tests
  - [ ] 10.2.4. Achieve 100% test coverage target

- [ ] 10.3. Update project structure documentation
  - [ ] 10.3.1. Update README.md with new VM architecture
  - [ ] 10.3.2. Add architecture diagram if helpful
  - [ ] 10.3.3. Document public API surface

### Phase 11: Cleanup and Optimization
**Priority: LOW**

- [ ] 11.1. Review and optimize module boundaries
  - [ ] 11.1.1. Ensure minimal coupling between modules
  - [ ] 11.1.2. Review public vs private APIs
  - [ ] 11.1.3. Consider using traits for extensibility points

- [ ] 11.2. Performance validation
  - [ ] 11.2.1. Benchmark before and after refactoring
  - [ ] 11.2.2. Ensure no performance regressions
  - [ ] 11.2.3. Profile if needed

- [ ] 11.3. Code quality checks
  - [ ] 11.3.1. Run `cargo clippy` and fix all warnings
  - [ ] 11.3.2. Run `cargo fmt` to ensure consistent formatting
  - [ ] 11.3.3. Run `scripts/misplaced_tests.sh` to verify test placement

## Proposed Final Structure

```
src/
├── vm/
│   ├── mod.rs                    # Main VM struct, entry point, re-exports (~200-300 lines)
│   ├── heap.rs                   # Heap management (~50 lines)
│   ├── call_frame.rs             # Call stack frames (~50 lines)
│   ├── global_registry.rs        # Global object registry (~80 lines)
│   ├── control_flow.rs           # ControlFlow enum and helpers (~50 lines)
│   ├── init.rs                   # VM initialization functions (~80 lines)
│   ├── utils.rs                  # Utility functions (~50 lines)
│   ├── errors.rs                 # Error construction functions (~200 lines)
│   ├── expression.rs             # Expression evaluation dispatcher (~200 lines)
│   ├── operators.rs              # Operator evaluation (~200 lines)
│   ├── statement.rs              # Statement execution dispatcher (~150 lines)
│   ├── control_structures.rs     # If/while/for execution (~200 lines)
│   ├── class_execution.rs        # Class definition execution (~150 lines)
│   ├── exceptions.rs             # Exception handling (raise/begin/rescue) (~200 lines)
│   ├── pattern_matching.rs       # Pattern matching logic (~250 lines)
│   ├── method_lookup.rs          # Method lookup and dispatch (~150 lines)
│   ├── method_invocation.rs      # Method invocation logic (~200 lines)
│   └── native_methods.rs         # Native method implementations (~250-300 lines)
│       └── (consider further splitting by class in the future)
```

## Benefits of This Refactoring

1. **Improved Maintainability**: Each module has a single, clear responsibility
2. **Better Testability**: Smaller modules are easier to unit test in isolation
3. **Easier Navigation**: Developers can quickly find relevant code
4. **Reduced Cognitive Load**: Each file is ~50-300 lines instead of 2177
5. **Better Documentation**: Module-level docs provide clear architectural overview
6. **Parallel Development**: Multiple developers can work on different modules simultaneously
7. **Future Extensibility**: Clear boundaries make it easier to add features

## Migration Strategy

1. **One phase at a time**: Complete each phase before moving to the next
2. **Test after each phase**: Ensure `cargo test` passes after each extraction
3. **Maintain 100% coverage**: Add tests as needed to maintain coverage target
4. **Run quality checks**: `cargo clippy && cargo fmt` after each phase
5. **Update documentation**: Keep docs in sync with code changes

## Dependencies Between Phases

- Phase 1-3 (Support structures, errors, utils) can be done first as they have no dependencies on other phases
- Phase 4-5 (Expression and Statement) depend on Phase 1-3 being complete
- Phase 6-8 (Exceptions, Pattern matching, Method invocation) depend on Phase 4-5
- Phase 9 (Core VM refactor) depends on all extraction phases (1-8)
- Phase 10-11 can be done iteratively throughout or at the end

## Timeline Estimate

- **Phase 1-3**: 1-2 days (straightforward extraction)
- **Phase 4-5**: 2-3 days (core logic, needs careful testing)
- **Phase 6-8**: 3-4 days (complex logic, extensive testing needed)
- **Phase 9**: 1-2 days (integration and cleanup)
- **Phase 10-11**: 2-3 days (documentation and polish)

**Total Estimate**: 9-14 days for complete refactoring with thorough testing

## Notes

- All existing tests must continue to pass at each checkpoint
- Code coverage must remain at 100% throughout the refactoring
- No functional changes should be made during refactoring - only structural changes
- Consider using feature flags if certain phases need to be done incrementally
