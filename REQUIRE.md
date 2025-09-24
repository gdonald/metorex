# require_relative Implementation Plan

This document breaks down the implementation of `require_relative` into small, granular tasks.

**Goal**: Implement `require_relative` to enable loading and executing Ruby/Metorex files relative to the current file's location, with proper deduplication to prevent re-loading the same file multiple times.

**Testing Philosophy**: Tests should be written incrementally alongside implementation, not saved for the end. Each phase includes test tasks that should be completed as you implement the corresponding functionality. This ensures continuous verification of correctness and maintains 100% code coverage throughout development.

---

## Phase 1: File Path Tracking Infrastructure

### 1.1 Add Current File Tracking to VM
- [x] 1.1.1 Add `current_file: Option<PathBuf>` field to `VirtualMachine` struct in [src/vm/core.rs](src/vm/core.rs)
- [x] 1.1.2 Add `set_current_file()` method to `VirtualMachine` to update current file path
- [x] 1.1.3 Add `get_current_file()` method to `VirtualMachine` to retrieve current file path
- [x] 1.1.4 Import `std::path::PathBuf` in [src/vm/core.rs](src/vm/core.rs)
- [x] 1.1.5 Write tests for `set_current_file()` and `get_current_file()` methods

### 1.2 Add Loaded Files Registry
- [x] 1.2.1 Add `loaded_files: HashSet<PathBuf>` field to `VirtualMachine` struct to track loaded files
- [x] 1.2.2 Import `std::collections::HashSet` in [src/vm/core.rs](src/vm/core.rs)
- [x] 1.2.3 Add `mark_file_loaded()` method to record a file as loaded
- [x] 1.2.4 Add `is_file_loaded()` method to check if a file was already loaded
- [x] 1.2.5 Initialize `loaded_files` as empty HashSet in `VirtualMachine::new()`
- [x] 1.2.6 Write tests for `mark_file_loaded()` and `is_file_loaded()` methods

### 1.3 Update Main Entry Point
- [x] 1.3.1 Update [src/main.rs](src/main.rs) to call `vm.set_current_file()` before executing a program
- [x] 1.3.2 Convert filename to absolute `PathBuf` using `std::fs::canonicalize()`
- [x] 1.3.3 Add error handling for invalid file paths in main.rs
- [x] 1.3.4 Mark the entry point file as loaded in main.rs
- [x] 1.3.5 Write integration test for main.rs file path tracking

---

## Phase 2: File Loading and Parsing Infrastructure

### 2.1 Create File Loader Module
- [x] 2.1.1 Create new file [src/file_loader.rs](src/file_loader.rs)
- [x] 2.1.2 Add `pub mod file_loader;` to [src/lib.rs](src/lib.rs)
- [x] 2.1.3 Import necessary types (`PathBuf`, `MetorexError`, etc.) in file_loader.rs

### 2.2 Implement File Reading Function
- [x] 2.2.1 Create `load_file_source(path: &Path) -> Result<String, MetorexError>` function
- [x] 2.2.2 Add file extension validation (.rb, .mx, or no extension)
- [x] 2.2.3 Try reading with given extension first, fall back to .rb, then .mx
- [x] 2.2.4 Return appropriate error if file doesn't exist with any extension
- [x] 2.2.5 Write tests for `load_file_source()` with various file extensions
- [x] 2.2.6 Write tests for `load_file_source()` error cases (non-existent files)

### 2.3 Implement Path Resolution Function
- [x] 2.3.1 Create `resolve_relative_path(base_file: &Path, relative_path: &str) -> Result<PathBuf, MetorexError>` function
- [x] 2.3.2 Get parent directory of base_file
- [x] 2.3.3 Join relative_path to parent directory
- [x] 2.3.4 Canonicalize the resulting path to resolve `..`, `.`, and symlinks
- [x] 2.3.5 Return error if path resolution fails or goes outside valid bounds
- [x] 2.3.6 Write tests for `resolve_relative_path()` with same directory
- [x] 2.3.7 Write tests for `resolve_relative_path()` with subdirectories
- [x] 2.3.8 Write tests for `resolve_relative_path()` with parent directories (`../`)

### 2.4 Implement File Parsing Function
- [x] 2.4.1 Create `parse_file(source: &str, filename: &str) -> Result<Vec<Statement>, MetorexError>` function
- [x] 2.4.2 Create lexer from source
- [x] 2.4.3 Tokenize source code
- [x] 2.4.4 Create parser from tokens
- [x] 2.4.5 Parse and return AST, converting parse errors to MetorexError
- [x] 2.4.6 Write tests for `parse_file()` with valid source code
- [x] 2.4.7 Write tests for `parse_file()` with syntax errors

---

## Phase 3: VM File Execution Infrastructure

### 3.1 Add File Execution Method to VM
- [ ] 3.1.1 Add `execute_file(&mut self, path: &Path) -> Result<Object, MetorexError>` method to VirtualMachine
- [ ] 3.1.2 Save the current file path before executing
- [ ] 3.1.3 Check if file is already loaded using `is_file_loaded()`
- [ ] 3.1.4 Return `Object::Nil` early if file was already loaded (deduplication)
- [ ] 3.1.5 Canonicalize the file path to absolute path
- [ ] 3.1.6 Mark file as loaded using `mark_file_loaded()`
- [ ] 3.1.7 Write test for `execute_file()` with simple file execution

### 3.2 Implement File Loading in execute_file
- [ ] 3.2.1 Call `load_file_source()` to read file contents
- [ ] 3.2.2 Call `parse_file()` to get AST
- [ ] 3.2.3 Update current file path using `set_current_file()`
- [ ] 3.2.4 Execute the parsed statements using `execute_program()`
- [ ] 3.2.5 Restore previous current file path after execution
- [ ] 3.2.6 Return the result (or `Object::Nil` if no return value)
- [ ] 3.2.7 Write test for `execute_file()` deduplication (same file twice)
- [ ] 3.2.8 Write test for `execute_file()` restoring previous file path

### 3.3 Add Error Context
- [ ] 3.3.1 Wrap file loading errors with context about which file failed
- [ ] 3.3.2 Wrap parsing errors with filename information
- [ ] 3.3.3 Wrap execution errors with call stack context
- [ ] 3.3.4 Write tests for error handling in `execute_file()`

---

## Phase 4: Native Function Implementation

### 4.1 Implement require_relative Native Function
- [ ] 4.1.1 Add `"require_relative"` case to `call_native_function()` in [src/vm/native_functions.rs](src/vm/native_functions.rs)
- [ ] 4.1.2 Validate that exactly 1 argument is provided
- [ ] 4.1.3 Extract string argument (the relative path)
- [ ] 4.1.4 Return error if argument is not a String
- [ ] 4.1.5 Write test for require_relative with wrong number of arguments
- [ ] 4.1.6 Write test for require_relative with non-string argument

### 4.2 Implement Path Resolution in require_relative
- [ ] 4.2.1 Get current file path using `get_current_file()`
- [ ] 4.2.2 Return error if no current file is set (REPL context)
- [ ] 4.2.3 Call `resolve_relative_path()` to get absolute path
- [ ] 4.2.4 Handle path resolution errors
- [ ] 4.2.5 Write test for require_relative in REPL context (no current file)
- [ ] 4.2.6 Write test for require_relative with invalid path

### 4.3 Implement File Execution in require_relative
- [ ] 4.3.1 Call `execute_file()` with resolved path
- [ ] 4.3.2 Handle execution errors
- [ ] 4.3.3 Return `Object::Boolean(true)` if file was newly loaded
- [ ] 4.3.4 Return `Object::Boolean(false)` if file was already loaded (matching Ruby behavior)
- [ ] 4.3.5 Write test for require_relative with simple file in same directory
- [ ] 4.3.6 Write test for require_relative with file in subdirectory
- [ ] 4.3.7 Write test for require_relative with file in parent directory (`../`)
- [ ] 4.3.8 Write test for require_relative return values (true/false)

### 4.4 Register require_relative Function
- [ ] 4.4.1 Add `require_relative` to global registry in [src/vm/init.rs](src/vm/init.rs)
- [ ] 4.4.2 Register it as a native function in `register_native_functions()`
- [ ] 4.4.3 Create `Object::NativeFunction` for require_relative
- [ ] 4.4.4 Write test verifying require_relative is registered globally

---

## Phase 5: Advanced Testing

### 5.1 Test File Extension Auto-Detection
- [ ] 5.1.1 Create test files with .rb extension
- [ ] 5.1.2 Create test files with .mx extension
- [ ] 5.1.3 Create test files without extension
- [ ] 5.1.4 Write tests for require_relative with .rb extension
- [ ] 5.1.5 Write tests for require_relative with .mx extension
- [ ] 5.1.6 Write tests for require_relative without extension (auto-detection)

### 5.2 Test Scope and Variable Sharing
- [ ] 5.2.1 Create test file that defines variables
- [ ] 5.2.2 Write test: variables defined in required file are accessible in requiring file
- [ ] 5.2.3 Create test file that defines functions
- [ ] 5.2.4 Write test: functions defined in required file are accessible
- [ ] 5.2.5 Create test file that defines classes
- [ ] 5.2.6 Write test: classes defined in required file are accessible
- [ ] 5.2.7 Write test: required files share global scope

### 5.3 Test Nested Requires
- [ ] 5.3.1 Create test files for nested requires (A requires B which requires C)
- [ ] 5.3.2 Write test: file A requires B which requires C - all execute in correct order
- [ ] 5.3.3 Create test files for circular requires (A requires B, B requires A)
- [ ] 5.3.4 Write test: circular require is handled gracefully (no infinite loop)
- [ ] 5.3.5 Create test files for diamond dependency (A requires B and C, both require D)
- [ ] 5.3.6 Write test: diamond dependency - D loads only once

---

## Phase 6: Integration Tests and Examples

### 6.1 Create Example Files
- [ ] 6.1.1 Create directory [tests/_examples/require/](tests/_examples/require/)
- [ ] 6.1.2 Create subdirectory [tests/_examples/require/lib/](tests/_examples/require/lib/)
- [ ] 6.1.3 Create [tests/_examples/require/lib/helper.mx](tests/_examples/require/lib/helper.mx) with a simple function
- [ ] 6.1.4 Create [tests/_examples/require/basic.mx](tests/_examples/require/basic.mx) using `require_relative "lib/helper"`
- [ ] 6.1.5 Add test case in examples_runner.rs for basic.mx with expected output

### 6.2 Create Deduplication Example
- [ ] 6.2.1 Create [tests/_examples/require/lib/counter.mx](tests/_examples/require/lib/counter.mx) that prints on load
- [ ] 6.2.2 Create [tests/_examples/require/deduplication.mx](tests/_examples/require/deduplication.mx) requiring counter.mx twice
- [ ] 6.2.3 Add test case in examples_runner.rs for deduplication.mx verifying counter loads once

### 6.3 Create Nested Require Example
- [ ] 6.3.1 Create [tests/_examples/require/lib/util_a.mx](tests/_examples/require/lib/util_a.mx)
- [ ] 6.3.2 Create [tests/_examples/require/lib/util_b.mx](tests/_examples/require/lib/util_b.mx) that requires util_a.mx
- [ ] 6.3.3 Create [tests/_examples/require/nested.mx](tests/_examples/require/nested.mx) that requires util_b.mx
- [ ] 6.3.4 Add test case in examples_runner.rs for nested.mx with expected output

### 6.4 Test with Ruby Files
- [ ] 6.4.1 Create simple Ruby test file (.rb) that uses require_relative
- [ ] 6.4.2 Add test case in examples_runner.rs to verify Metorex can execute it
- [ ] 6.4.3 Document any compatibility limitations discovered

---

## Phase 7: Documentation and Polish

### 7.1 Code Documentation
- [ ] 7.1.1 Add rustdoc comments to all new public functions in file_loader.rs
- [ ] 7.1.2 Add rustdoc comments to new VM methods
- [ ] 7.1.3 Add module-level documentation to file_loader.rs
- [ ] 7.1.4 Document the file loading strategy and deduplication approach

### 7.2 Error Messages
- [ ] 7.2.1 Review all error messages for clarity and helpfulness
- [ ] 7.2.2 Ensure file paths in errors use absolute paths for clarity
- [ ] 7.2.3 Add suggestions in error messages (e.g., "did you mean file.rb?")
- [ ] 7.2.4 Write tests verifying error message quality

### 7.3 Update README
- [ ] 7.3.1 Add require_relative to the feature list in README.md
- [ ] 7.3.2 Add example usage of require_relative
- [ ] 7.3.3 Document any differences from Ruby's require_relative

---

## Phase 8: Final Verification

### 8.1 Run Full Test Suite
- [ ] 8.1.1 Run `cargo test` and ensure all tests pass
- [ ] 8.1.2 Run `cargo clippy` and fix any warnings
- [ ] 8.1.3 Run `cargo fmt` to format code

### 8.2 Verify Code Coverage
- [ ] 8.2.1 Run `cargo tarpaulin --out Stdout` to check coverage
- [ ] 8.2.2 Ensure require_relative implementation has 100% test coverage
- [ ] 8.2.3 Add any missing tests to reach 100% coverage

### 8.3 Verify Test Organization
- [ ] 8.3.1 Run `scripts/misplaced_tests.sh` to ensure no tests in implementation files
- [ ] 8.3.2 Move any misplaced tests to [tests/](tests/) directory
- [ ] 8.3.3 Verify all tests are in examples_runner.rs or dedicated test files

### 8.4 Manual Testing and Compatibility
- [ ] 8.4.1 Test with CRuby's test/runner.rb to see how far it gets
- [ ] 8.4.2 Document what works and what additional features are needed
- [ ] 8.4.3 Create a list of next features needed for CRuby compatibility

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
