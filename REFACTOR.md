# Object Module Refactoring Plan

## Overview
The `src/object.rs` file (606 lines) contains multiple distinct components that should be separated into a modular structure. This refactoring will improve maintainability, testability, and code organization.

## Current Structure Analysis
The file contains:
- Core `Object` enum (runtime value representation)
- `Instance` struct (class instances)
- `Method` struct (class methods)
- `BlockStatement` struct (closures/lambdas)
- `Exception` struct and `SourceLocation` (error handling)
- `ObjectHash` wrapper (for Set support)
- Display implementation for Object
- Helper methods for Object

---

## Phase 1: Create Object Module Directory Structure

- [ ] 1.1. Create `src/object/` directory to hold the refactored modules
- [ ] 1.2. Create `src/object/mod.rs` as the main module entry point
- [ ] 1.3. Verify directory structure is correct

---

## Phase 2: Extract Core Type Modules

- [ ] 2.1. Create `src/object/types.rs`
  - [ ] 2.1.1. Move the `Object` enum definition
  - [ ] 2.1.2. Move `Object::type_name()` method
  - [ ] 2.1.3. Export publicly from types module

- [ ] 2.2. Create `src/object/instance.rs`
  - [ ] 2.2.1. Move `Instance` struct and all its implementations
  - [ ] 2.2.2. Keep all instance-related methods together
  - [ ] 2.2.3. Import necessary dependencies (Class, Object)

- [ ] 2.3. Create `src/object/method.rs`
  - [ ] 2.3.1. Move `Method` struct and implementations
  - [ ] 2.3.2. Move `Callable` trait implementation for Method
  - [ ] 2.3.3. Ensure binding and receiver methods are included

- [ ] 2.4. Create `src/object/block.rs`
  - [ ] 2.4.1. Move `BlockStatement` struct and implementations
  - [ ] 2.4.2. Move `Callable` trait implementation for BlockStatement
  - [ ] 2.4.3. Keep closure capture logic together

---

## Phase 3: Extract Exception Handling

- [ ] 3.1. Create `src/object/exception.rs`
  - [ ] 3.1.1. Move `Exception` struct and all constructors
  - [ ] 3.1.2. Move `SourceLocation` struct
  - [ ] 3.1.3. Move exception chain logic
  - [ ] 3.1.4. Keep all error-handling related code together

---

## Phase 4: Extract Collection Support

- [ ] 4.1. Create `src/object/hash.rs`
  - [ ] 4.1.1. Move `ObjectHash` wrapper struct
  - [ ] 4.1.2. Move `Object::hash()` method implementation
  - [ ] 4.1.3. Keep all hashing logic centralized

---

## Phase 5: Extract Display and Formatting

- [ ] 5.1. Create `src/object/display.rs`
  - [ ] 5.1.1. Move `fmt::Display` implementation for Object
  - [ ] 5.1.2. Keep all string formatting logic together
  - [ ] 5.1.3. Consider adding additional formatting helpers if needed

---

## Phase 6: Extract Helper Methods

- [ ] 6.1. Create `src/object/constructors.rs`
  - [ ] 6.1.1. Move `Object::string()` factory method
  - [ ] 6.1.2. Move `Object::empty_array()` and `Object::array()` methods
  - [ ] 6.1.3. Move `Object::empty_dict()` and `Object::dict()` methods
  - [ ] 6.1.4. Move `Object::empty_set()` method
  - [ ] 6.1.5. Move `Object::instance()` method
  - [ ] 6.1.6. Move `Object::exception()` method
  - [ ] 6.1.7. Move `Object::ok()` and `Object::err()` methods

- [ ] 6.2. Create `src/object/operations.rs`
  - [ ] 6.2.1. Move `Object::is_truthy()` method
  - [ ] 6.2.2. Move `Object::is_falsy()` method
  - [ ] 6.2.3. Move `Object::equals()` method (deep equality)
  - [ ] 6.2.4. Keep all comparison and boolean logic together

---

## Phase 7: Configure Module Exports

- [ ] 7.1. Set up `src/object/mod.rs`
  - [ ] 7.1.1. Declare all submodules (types, instance, method, block, exception, hash, display, constructors, operations)
  - [ ] 7.1.2. Re-export main types: `Object`, `Instance`, `Method`, `BlockStatement`, `Exception`, `SourceLocation`, `ObjectHash`
  - [ ] 7.1.3. Re-export necessary traits from callable
  - [ ] 7.1.4. Ensure public API remains unchanged for backward compatibility

- [ ] 7.2. Update `src/lib.rs`
  - [ ] 7.2.1. Verify `pub mod object;` declaration still works
  - [ ] 7.2.2. No changes should be needed if re-exports are correct

---

## Phase 8: Update All Imports Throughout Codebase

- [ ] 8.1. Update imports in VM modules
  - [ ] 8.1.1. Update `src/vm/core.rs`
  - [ ] 8.1.2. Update `src/vm/method_invocation.rs`
  - [ ] 8.1.3. Update `src/vm/method_lookup.rs`
  - [ ] 8.1.4. Update `src/vm/class_execution.rs`
  - [ ] 8.1.5. Update all other VM files (operators, expression, statement, etc.)

- [ ] 8.2. Update imports in other core modules
  - [ ] 8.2.1. Update `src/class.rs`
  - [ ] 8.2.2. Update `src/builtin_classes.rs`
  - [ ] 8.2.3. Update `src/environment.rs`
  - [ ] 8.2.4. Update `src/scope.rs`

- [ ] 8.3. Verify imports
  - [ ] 8.3.1. Run `cargo check` to verify no import errors
  - [ ] 8.3.2. Fix any broken imports

---

## Phase 9: Testing and Verification

- [ ] 9.1. Run comprehensive tests
  - [ ] 9.1.1. Run `cargo test` to ensure all tests pass
  - [ ] 9.1.2. Verify no functionality was broken

- [ ] 9.2. Run code quality checks
  - [ ] 9.2.1. Run `cargo clippy` and fix any violations
  - [ ] 9.2.2. Run `cargo fmt` to ensure consistent formatting

- [ ] 9.3. Verify test coverage
  - [ ] 9.3.1. Run `cargo tarpaulin --out Stdout`
  - [ ] 9.3.2. Ensure coverage percentage has not decreased
  - [ ] 9.3.3. If coverage decreased, add missing tests for the refactored code

---

## Expected File Structure After Refactoring

```
src/object/
├── mod.rs              # Module entry point with re-exports
├── types.rs            # Object enum definition
├── instance.rs         # Instance struct and methods
├── method.rs           # Method struct and Callable impl
├── block.rs            # BlockStatement and closure logic
├── exception.rs        # Exception and SourceLocation
├── hash.rs             # ObjectHash wrapper and hashing
├── display.rs          # Display trait implementation
├── constructors.rs     # Factory methods for Object
└── operations.rs       # Comparison and boolean operations
```

---

## Benefits of This Refactoring

1. **Modularity**: Each component has its own file, making it easier to locate and modify specific functionality
2. **Maintainability**: Smaller files are easier to understand and maintain
3. **Testing**: Each module can be tested independently
4. **Scalability**: New object types or operations can be added without bloating a single file
5. **Separation of Concerns**: Related functionality is grouped together logically
6. **Code Navigation**: Developers can quickly find the code they need to work on

---

## Notes

- This refactoring maintains backward compatibility - all public APIs remain accessible via `use crate::object::{Object, Instance, ...}`
- No functionality changes, purely structural reorganization
- The modular structure follows Rust best practices for organizing large modules
- Similar pattern to how `vm/` and `parser/` modules are already structured in the codebase
