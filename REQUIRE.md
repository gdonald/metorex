# require_relative Implementation Plan

This document breaks down the implementation of `require_relative` into small, granular tasks.

**Goal**: Implement `require_relative` to enable loading and executing Ruby/Metorex files relative to the current file's location, with proper deduplication to prevent re-loading the same file multiple times.

---

## Phase 1: File Path Tracking Infrastructure

### 1.1 Add Current File Tracking to VM
- [ ] 1.1.1 Add `current_file: Option<PathBuf>` field to `VirtualMachine` struct in [src/vm/core.rs](src/vm/core.rs)
- [ ] 1.1.2 Add `set_current_file()` method to `VirtualMachine` to update current file path
- [ ] 1.1.3 Add `get_current_file()` method to `VirtualMachine` to retrieve current file path
- [ ] 1.1.4 Import `std::path::PathBuf` in [src/vm/core.rs](src/vm/core.rs)

### 1.2 Add Loaded Files Registry
- [ ] 1.2.1 Add `loaded_files: HashSet<PathBuf>` field to `VirtualMachine` struct to track loaded files
- [ ] 1.2.2 Import `std::collections::HashSet` in [src/vm/core.rs](src/vm/core.rs)
- [ ] 1.2.3 Add `mark_file_loaded()` method to record a file as loaded
- [ ] 1.2.4 Add `is_file_loaded()` method to check if a file was already loaded
- [ ] 1.2.5 Initialize `loaded_files` as empty HashSet in `VirtualMachine::new()`

### 1.3 Update Main Entry Point
- [ ] 1.3.1 Update [src/main.rs](src/main.rs) to call `vm.set_current_file()` before executing a program
- [ ] 1.3.2 Convert filename to absolute `PathBuf` using `std::fs::canonicalize()`
- [ ] 1.3.3 Add error handling for invalid file paths in main.rs
- [ ] 1.3.4 Mark the entry point file as loaded in main.rs

---

## Phase 2: File Loading and Parsing Infrastructure

### 2.1 Create File Loader Module
- [ ] 2.1.1 Create new file [src/file_loader.rs](src/file_loader.rs)
- [ ] 2.1.2 Add `pub mod file_loader;` to [src/lib.rs](src/lib.rs)
- [ ] 2.1.3 Import necessary types (`PathBuf`, `MetorexError`, etc.) in file_loader.rs

### 2.2 Implement File Reading Function
- [ ] 2.2.1 Create `load_file_source(path: &Path) -> Result<String, MetorexError>` function
- [ ] 2.2.2 Add file extension validation (.rb, .mx, or no extension)
- [ ] 2.2.3 Try reading with given extension first, fall back to .rb, then .mx
- [ ] 2.2.4 Return appropriate error if file doesn't exist with any extension

### 2.3 Implement Path Resolution Function
- [ ] 2.3.1 Create `resolve_relative_path(base_file: &Path, relative_path: &str) -> Result<PathBuf, MetorexError>` function
- [ ] 2.3.2 Get parent directory of base_file
- [ ] 2.3.3 Join relative_path to parent directory
- [ ] 2.3.4 Canonicalize the resulting path to resolve `..`, `.`, and symlinks
- [ ] 2.3.5 Return error if path resolution fails or goes outside valid bounds

### 2.4 Implement File Parsing Function
- [ ] 2.4.1 Create `parse_file(source: &str, filename: &str) -> Result<Vec<Statement>, MetorexError>` function
- [ ] 2.4.2 Create lexer from source
- [ ] 2.4.3 Tokenize source code
- [ ] 2.4.4 Create parser from tokens
- [ ] 2.4.5 Parse and return AST, converting parse errors to MetorexError

---

## Phase 3: VM File Execution Infrastructure

### 3.1 Add File Execution Method to VM
- [ ] 3.1.1 Add `execute_file(&mut self, path: &Path) -> Result<Object, MetorexError>` method to VirtualMachine
- [ ] 3.1.2 Save the current file path before executing
- [ ] 3.1.3 Check if file is already loaded using `is_file_loaded()`
- [ ] 3.1.4 Return `Object::Nil` early if file was already loaded (deduplication)
- [ ] 3.1.5 Canonicalize the file path to absolute path
- [ ] 3.1.6 Mark file as loaded using `mark_file_loaded()`

### 3.2 Implement File Loading in execute_file
- [ ] 3.2.1 Call `load_file_source()` to read file contents
- [ ] 3.2.2 Call `parse_file()` to get AST
- [ ] 3.2.3 Update current file path using `set_current_file()`
- [ ] 3.2.4 Execute the parsed statements using `execute_program()`
- [ ] 3.2.5 Restore previous current file path after execution
- [ ] 3.2.6 Return the result (or `Object::Nil` if no return value)

### 3.3 Add Error Context
- [ ] 3.3.1 Wrap file loading errors with context about which file failed
- [ ] 3.3.2 Wrap parsing errors with filename information
- [ ] 3.3.3 Wrap execution errors with call stack context

---

## Phase 4: Native Function Implementation

### 4.1 Implement require_relative Native Function
- [ ] 4.1.1 Add `"require_relative"` case to `call_native_function()` in [src/vm/native_functions.rs](src/vm/native_functions.rs)
- [ ] 4.1.2 Validate that exactly 1 argument is provided
- [ ] 4.1.3 Extract string argument (the relative path)
- [ ] 4.1.4 Return error if argument is not a String

### 4.2 Implement Path Resolution in require_relative
- [ ] 4.2.1 Get current file path using `get_current_file()`
- [ ] 4.2.2 Return error if no current file is set (REPL context)
- [ ] 4.2.3 Call `resolve_relative_path()` to get absolute path
- [ ] 4.2.4 Handle path resolution errors

### 4.3 Implement File Execution in require_relative
- [ ] 4.3.1 Call `execute_file()` with resolved path
- [ ] 4.3.2 Handle execution errors
- [ ] 4.3.3 Return `Object::Boolean(true)` if file was newly loaded
- [ ] 4.3.4 Return `Object::Boolean(false)` if file was already loaded (matching Ruby behavior)

### 4.4 Register require_relative Function
- [ ] 4.4.1 Add `require_relative` to global registry in [src/vm/init.rs](src/vm/init.rs)
- [ ] 4.4.2 Register it as a native function in `register_native_functions()`
- [ ] 4.4.3 Create `Object::NativeFunction` for require_relative

---

## Phase 5: Testing Infrastructure

### 5.1 Create Test Files Directory
- [ ] 5.1.1 Create directory [tests/_examples/require/](tests/_examples/require/)
- [ ] 5.1.2 Create subdirectory [tests/_examples/require/lib/](tests/_examples/require/lib/)

### 5.2 Create Basic Test Files
- [ ] 5.2.1 Create [tests/_examples/require/lib/helper.mx](tests/_examples/require/lib/helper.mx) with a simple function definition
- [ ] 5.2.2 Create [tests/_examples/require/basic.mx](tests/_examples/require/basic.mx) that uses `require_relative "lib/helper"`
- [ ] 5.2.3 Create [tests/_examples/require/lib/greetings.mx](tests/_examples/require/lib/greetings.mx) with greeting functions

### 5.3 Create Deduplication Test Files
- [ ] 5.3.1 Create [tests/_examples/require/lib/counter.mx](tests/_examples/require/lib/counter.mx) that prints on load
- [ ] 5.3.2 Create [tests/_examples/require/deduplication.mx](tests/_examples/require/deduplication.mx) that requires counter.mx twice
- [ ] 5.3.3 Verify counter.mx output only appears once

### 5.4 Create Nested Require Test Files
- [ ] 5.4.1 Create [tests/_examples/require/lib/util_a.mx](tests/_examples/require/lib/util_a.mx)
- [ ] 5.4.2 Create [tests/_examples/require/lib/util_b.mx](tests/_examples/require/lib/util_b.mx) that requires util_a.mx
- [ ] 5.4.3 Create [tests/_examples/require/nested.mx](tests/_examples/require/nested.mx) that requires util_b.mx
- [ ] 5.4.4 Verify all files execute and util_a.mx only loads once

---

## Phase 6: Unit Tests

### 6.1 Create require_relative Test Module
- [ ] 6.1.1 Create [tests/require_relative_tests.rs](tests/require_relative_tests.rs)
- [ ] 6.1.2 Add common test utilities (create temp files, parse, execute)

### 6.2 Test Basic Functionality
- [ ] 6.2.1 Write test: require_relative with simple file in same directory
- [ ] 6.2.2 Write test: require_relative with file in subdirectory
- [ ] 6.2.3 Write test: require_relative with file in parent directory (`../lib/foo`)
- [ ] 6.2.4 Write test: require_relative with .rb extension
- [ ] 6.2.5 Write test: require_relative with .mx extension
- [ ] 6.2.6 Write test: require_relative without extension (auto-detection)

### 6.3 Test Error Handling
- [ ] 6.3.1 Write test: require_relative with non-existent file returns error
- [ ] 6.3.2 Write test: require_relative with invalid path returns error
- [ ] 6.3.3 Write test: require_relative with non-string argument returns error
- [ ] 6.3.4 Write test: require_relative with wrong number of arguments returns error
- [ ] 6.3.5 Write test: require_relative in REPL (no current file) returns helpful error

### 6.4 Test Deduplication
- [ ] 6.4.1 Write test: requiring same file twice only executes once
- [ ] 6.4.2 Write test: requiring same file via different paths (./foo vs foo) only executes once
- [ ] 6.4.3 Write test: verify return value is true on first load, false on subsequent loads

### 6.5 Test Scope and Variables
- [ ] 6.5.1 Write test: variables defined in required file are accessible in requiring file
- [ ] 6.5.2 Write test: functions defined in required file are accessible
- [ ] 6.5.3 Write test: classes defined in required file are accessible
- [ ] 6.5.4 Write test: required files share global scope

### 6.6 Test Nested Requires
- [ ] 6.6.1 Write test: file A requires B which requires C - all execute in correct order
- [ ] 6.6.2 Write test: circular require detection (A requires B, B requires A)
- [ ] 6.6.3 Write test: diamond dependency (A requires B and C, both require D - D loads once)

---

## Phase 7: Integration Tests

### 7.1 Add to examples_runner.rs
- [ ] 7.1.1 Add test case for [tests/_examples/require/basic.mx](tests/_examples/require/basic.mx)
- [ ] 7.1.2 Add test case for [tests/_examples/require/deduplication.mx](tests/_examples/require/deduplication.mx)
- [ ] 7.1.3 Add test case for [tests/_examples/require/nested.mx](tests/_examples/require/nested.mx)
- [ ] 7.1.4 Add expected output verification for each test

### 7.2 Test with Real Ruby Files
- [ ] 7.2.1 Create simple Ruby test file that uses require_relative
- [ ] 7.2.2 Verify Metorex can execute it successfully
- [ ] 7.2.3 Document any compatibility limitations

---

## Phase 8: Documentation and Polish

### 8.1 Code Documentation
- [ ] 8.1.1 Add rustdoc comments to all new public functions
- [ ] 8.1.2 Add module-level documentation to file_loader.rs
- [ ] 8.1.3 Document the file loading strategy and deduplication approach

### 8.2 Error Messages
- [ ] 8.2.1 Review all error messages for clarity and helpfulness
- [ ] 8.2.2 Ensure file paths in errors use absolute paths for clarity
- [ ] 8.2.3 Add suggestions in error messages (e.g., "did you mean file.rb?")

### 8.3 Update README
- [ ] 8.3.1 Add require_relative to the feature list in README.md
- [ ] 8.3.2 Add example usage of require_relative
- [ ] 8.3.3 Document any differences from Ruby's require_relative

---

## Phase 9: Verification and Cleanup

### 9.1 Run Full Test Suite
- [ ] 9.1.1 Run `cargo test` and ensure all tests pass
- [ ] 9.1.2 Run `cargo clippy` and fix any warnings
- [ ] 9.1.3 Run `cargo fmt` to format code

### 9.2 Verify Code Coverage
- [ ] 9.2.1 Run `cargo tarpaulin --out Stdout` to check coverage
- [ ] 9.2.2 Ensure require_relative implementation has 100% test coverage
- [ ] 9.2.3 Add any missing tests to reach 100% coverage

### 9.3 Verify misplaced_tests.sh
- [ ] 9.3.1 Run `scripts/misplaced_tests.sh` to ensure no tests in implementation files
- [ ] 9.3.2 Move any misplaced tests to [tests/](tests/) directory

### 9.4 Manual Testing
- [ ] 9.4.1 Test with CRuby's test/runner.rb to see how far it gets
- [ ] 9.4.2 Document what works and what additional features are needed
- [ ] 9.4.3 Create a list of next features needed for CRuby compatibility

---

## Success Criteria

After completing all tasks:
- ✅ `require_relative` works with relative paths (./foo, ../lib/bar, subdir/baz)
- ✅ Files are only loaded and executed once (deduplication works)
- ✅ File extension auto-detection works (.rb, .mx, or no extension)
- ✅ Nested requires work correctly (A requires B requires C)
- ✅ Circular dependencies are handled gracefully
- ✅ Error messages are clear and helpful
- ✅ All tests pass (`cargo test`)
- ✅ Code coverage remains at 100%
- ✅ No clippy warnings
- ✅ Code is properly formatted
- ✅ Documentation is complete

---

## Future Enhancements (Out of Scope)

These are related features that may be needed later but are not part of this initial implementation:

- [ ] Implement `require` (non-relative, uses load path)
- [ ] Implement `$LOAD_PATH` / `$:` global variable
- [ ] Implement `load` (always re-loads file)
- [ ] Implement `autoload` (lazy loading)
- [ ] Add support for loading compiled bytecode
- [ ] Add support for loading C extensions (.so, .dylib, .dll)
- [ ] Add `__FILE__` and `__LINE__` magic constants
- [ ] Add proper `Kernel` module and move require_relative there

---

## Notes

- **Ruby Compatibility**: This implementation aims to match Ruby's `require_relative` behavior as closely as possible
- **File Extensions**: Following Ruby convention, try .rb first, then fall back to .mx (Metorex extension)
- **Path Canonicalization**: Using `fs::canonicalize()` ensures absolute paths and handles symlinks correctly
- **Deduplication**: Files are tracked by their canonical absolute path to prevent duplicate loading
- **Scope Sharing**: Required files execute in the same global scope, matching Ruby's behavior
- **REPL Context**: `require_relative` will fail in REPL mode since there's no "current file" - this matches Ruby
