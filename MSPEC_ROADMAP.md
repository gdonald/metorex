# Ruby Spec Suite (mspec) Compatibility

Goal: Get the official Ruby spec suite (ruby/spec) running against Metorex via mspec, progressively passing more specs.

## The `ruby/` directory

The `ruby/` directory contains two git submodules:

- **`ruby/mspec/`** — The [mspec](https://github.com/ruby/mspec) test runner. Provides the `mspec-run` binary (`ruby/mspec/bin/mspec-run`), the `describe`/`it`/`before`/`after` DSL, matchers (`should`, `should_not`, `equal`, `raise_error`, etc.), guards (`ruby_version_is`, `platform_is`, etc.), and helpers (`MSpec.protect`, `ExceptionState`, `MSpecScript`, etc.). The main library entry point is `ruby/mspec/lib/mspec.rb` which requires the sub-libraries under `ruby/mspec/lib/mspec/`.
- **`ruby/spec/`** — The [ruby/spec](https://github.com/ruby/spec) test suite. Contains the actual spec files organized by category: `core/` (built-in classes like `TrueClass`, `NilClass`, `String`, `Array`, etc.), `language/` (Ruby language semantics), `library/` (stdlib), and others. Each spec file uses the mspec DSL to define expectations.

To run specs against Metorex: `cargo run -- ruby/mspec/bin/mspec-run -t ./target/debug/metorex spec/core/true`

## Phase 1: CLI Harness

- [x] 1.1. Accept `-v` flag and output Ruby-compatible version string (ruby 4.0.2)
- [x] 1.2. Accept `-e "code"` flag to evaluate inline code
- [x] 1.3. Accept and ignore `--disable=gems` flag
- [x] 1.4. Accept multiple file arguments (trailing args for script ARGV)
- [x] 1.5. Wire up script arguments as ARGV in the VM
- [x] 1.6. Accept common Ruby flags without crashing (`-r`, `-I`, `-w`, `-W`, `-d`) — currently ignored, real implementations in Phase 2

## Phase 2: Parse and Execute mspec-run

- [x] 2.1. Lex special global variables (`$:`, `$0`, `$;`, `$,`, etc.)
- [x] 2.2. Initialize special global variables (`$:`, `$LOAD_PATH`, `$stderr`, `$0`, `$DEBUG`, etc.)
- [x] 2.3. Support `__FILE__` and `__LINE__` magic constants
- [x] 2.4. Implement `File.expand_path` with one and two-argument forms
- [x] 2.5. Implement `require` (non-relative, uses `$LOAD_PATH`)
- [x] 2.6. Support paren-less method calls on dotted expressions (`$:.unshift expr`)
- [x] 2.7. Wire up `-r` flag to actually require libraries
- [x] 2.8. Wire up `-I` flag to prepend to `$LOAD_PATH`

## Phase 3: mspec Library Compatibility

### Parser / Syntax — Completed

- [x] 3.1. Multiple assignment / destructuring (`a, b, c = expr`)
- [x] 3.2. `<=>` spaceship operator (lexer, parser, VM, and as method name)
- [x] 3.3. `if` / `unless` postfix modifiers (`x = 99 unless x`)
- [x] 3.4. Default parameter values (`def foo(a, b = 5)`)
- [x] 3.5. Semicolons as statement separators (`class Foo; end`)
- [x] 3.6. `class << self` singleton class syntax
- [x] 3.7. `lambda { |x| ... }` brace block syntax
- [x] 3.8. `raise ExceptionClass, message` two-argument form
- [x] 3.9. `?x` character literals and `?` ternary operator
- [x] 3.10. `<<` shovel operator (array append)
- [x] 3.11. Ternary operator (`condition ? true_expr : false_expr`)
- [x] 3.12. Paren-less calls with multiple comma-separated args (`add short, long, arg`)
- [x] 3.13. Assignment in conditions (`unless option = match?(opt)`)
- [x] 3.14. Empty bracket call (`obj[]`) and multi-arg bracket (`str[0, n]`)
- [x] 3.15. Multiple return values (`return a, b, c`)
- [x] 3.16. Method-level rescue (`def method ... rescue Error => e ... end`)
- [x] 3.17. `or` / `and` / `not` keyword operators
- [x] 3.18. `next` keyword (alias for continue)
- [x] 3.19. Postfix modifiers on `break` / `next` (`next unless condition`)
- [x] 3.20. `&expr` block-to-proc conversion in call arguments
- [x] 3.21. `def obj.method_name` singleton method definition (parsed, not fully wired)
- [x] 3.22. Symbol arguments in paren-less calls (`foo :hello, "world"`)

### Parser / Syntax — Remaining

- [x] 3.23. Paren-less dotted method calls with symbol args (`MSpec.register :start, obj` — ambiguous with ternary `:`)
- [x] 3.24. String `%` formatting operator (`"format" % args`)
- [x] 3.25. Regex literals (`/pattern/`)
- [x] 3.26. `yield` keyword
- [x] 3.27. Splat operators (`*args`, `**kwargs`)
- [x] 3.28. `defined?` keyword

### Runtime / Stdlib — Completed

- [x] 3.29. `Comparable`, `Enumerable`, `Kernel` module stubs
- [x] 3.30. Constant assignment in module body (`VERSION = ...`)
- [x] 3.31. Scope resolution on modules (`MSpec::VERSION`)

### Runtime / Stdlib — Remaining

- [x] 3.32. `String#to_i` method
- [x] 3.33. `String#dup` method
- [x] 3.34. `Array#inject` / `Array#reduce` with initial value
- [x] 3.35. `Array#shift`, `Array#dup`
- [x] 3.36. `Integer()` / `String()` / `Array()` kernel methods
- [x] 3.37. `exit` function
- [x] 3.38. `load` function
- [x] 3.39. Proc/Lambda `[]` call syntax

### Current blocker

Phase 3 complete. Phase 4 in progress.

Parser fixes during Phase 4 work:
- Fixed paren-less dotted method calls with multiple args (`Foo.bar "hello", 30, "world"`)
- Fixed ternary `:` disambiguation with paren-less symbol args and method call args
- Added `||=` and `&&=` compound assignment operators
- Added `:@ivar` / `:@@cvar` / `:keyword` symbol syntax
- Added keywords as dotted method names (`.extend`, `.include`, `.module`, etc.)
- Fixed scope resolution in rescue clauses (`rescue Errno::ENOENT`)
- Fixed paren-less arg parsing consuming newlines (breaking subsequent `if` statements)
- Added `return X if/unless condition` postfix modifiers on return statements
- Added symbol literals in `case/when` patterns (`:major`, `:minor`, etc.)
- Added `^` XOR operator (boolean and integer)
- Fixed `yield if condition` — yield without args now stops before `if`/`unless` keywords
- Added multi-line `or`/`and` — continuation past newlines after logical operators
- Added keyword names in `attr_reader`/`attr_writer`/`attr_accessor` (`:include`, `:exclude`, etc.)
- Added `return unless/if` with no value (postfix modifier on bare return)
- Added assignment-in-condition for postfix modifiers (`return unless x = expr`)
- Added symbol from string literal (`:"string"`) and interpolated symbols (`:"@#{expr}"`)
- Extended ternary disambiguation for `:` + String (not just `:` + Ident)
- Fixed chained ternary with `:` + InstanceVar/any token — broadened ternary colon disambiguation guard to reject paren-less Colon args without a following Comma regardless of what follows the Colon
- Parallel assignment with bracket accessors (`ary[i], ary[r] = ary[r], ary[i]`) with correct RHS-first evaluation
- `%r(pattern)` regex literal syntax
- `===` triple-equals operator (case equality)
- `def name=(val)` setter method definition
- `-> { body }` and `-> (params) { body }` stabby lambda in expression context (args, assignments)
- Chained assignment (`@a = @b = value`) at statement level
- Assignment in `elsif` conditions (`elsif x = expr`)
- `=~` and `!~` regex match operators with runtime implementation
- `%[...]` / `%Q[...]` string literal syntax
- Postfix `if`/`unless` on multiple assignment statements
- Module body: general statements, instance variable assignments, `class << self` with attr_*
- Bare identifier to method dispatch on `self` (class/module body method calls)
- Module method lookup in `lookup_method`
- Setter methods on Class/Module objects (`Config.debug = true`)
- Class/Module reopening (Ruby semantics: `module M; end` reopens existing M)
- Global registration of classes/modules (persist across file scopes)
- Instance variable read/write on Class/Module (stored as class vars)
- `Object.const_defined?` method
- Backtick command strings (`` `command` `` → String token)
- Operator symbols (`:[]`, `:[]=`, `:+`, `:-`, etc.)
- `Range#begin`, `Range#end`, `Range#first`, `Range#last`, `Range#exclude_end?`
- `&block` argument extraction from positional args in method invocation
- `String#[]` with integer index and range index
- Assignment as last expression returns value (fixes `||=` in methods)
- Recursion guard: bare identifier → method dispatch skips current method to prevent infinite recursion
- **`def self.method` vs `def method` separation** — class methods stored with `__class__` prefix, looked up preferentially when receiver is Class/Module. Fixes infinite recursion where instance `config` and class `config` shadowed each other.
- 64MB thread stack for deep mspec execution
- `evaluate_expression` depth guard (1000 limit) prevents stack overflow
- Symbol keys in hash index assignment (`config[:formatter] = nil`)
- `instance_variable_set` native method on Instance/Class/Module
- `instance_variable_get` for Class/Module receivers (reads from class vars)
- Bare identifier → method dispatch fallback in `Expression::Call` (try `self.method(args)` when identifier fails)
- Paren-less call with `:InterpolatedString` arg (e.g., `instance_variable_get :"@#{sym}"`)
- **mspec runner `MSpecRun.main` begins executing.** `MSpecScript.new` → `check_version!` → `ruby_version_is` guard called. Next blocker: `MSpec.mode?` called with 0 args from guard dispatch (bare identifier dispatch auto-invokes 0-arg methods, but guard code calls `mode?` expecting 1 arg).
- Bare `new` inside `def self.method` now properly instantiates the class (was returning the Class object). Fixed in `Expression::Identifier` evaluation.
- `Expression::Call` with a bare identifier callee now dispatches to `self.method(args)` first (instead of auto-invoking the identifier with zero args, which discarded the supplied arguments and returned a value that was then "called").
- `&block` parameter binds to `nil` when method is called without a block (was leaving the local variable undefined).
- `String#ljust`, `String#rjust` (1-arg and 2-arg with pad string).
- `String#start_with?`, `String#end_with?` (multi-arg form).
- mspec progresses through `script.load_default` → `script.options`. Next blocker: line 256 in options.rb (`Undefined method 'empty?' for type 'Nil'`) — likely `&block` chain or arg parsing.

## Phase 4: Passing Specs

### Primitives and Singletons

- [x] 4.1. core/true — 9 files, 13 examples, 28 expectations, 0 failures, 0 errors
- [x] 4.2. core/nil — 18 files, 27 examples, 50 expectations, 0 failures, 0 errors
- [x] 4.3. core/false — 9 files, 13 examples, 29 expectations, 0 failures, 0 errors

### Core Classes and Modules

- [x] 4.4. core/comparable — 7 files, 54 examples, 103 expectations, 0 failures, 0 errors
- [ ] 4.5. core/main
  - [x] 4.5.1. core/main/define_method_spec — 1 file, 3 examples, 3 expectations, 0 failures, 0 errors
  - [x] 4.5.2. core/main/include_spec — 1 file, 2 examples, 2 expectations, 0 failures, 0 errors
  - [x] 4.5.3. core/main/private_spec — 1 file, 5 examples, 7 expectations, 0 failures, 0 errors
  - [ ] 4.5.4. core/main/public_spec
  - [ ] 4.5.5. core/main/ruby2_keywords_spec
  - [x] 4.5.6. core/main/to_s_spec — 1 file, 1 example, 1 expectation, 0 failures, 0 errors
  - [ ] 4.5.7. core/main/using_spec
- [ ] 4.6. core/class
  - [x] 4.6.1. core/class/allocate_spec — 1 file, 5 examples, 6 expectations, 0 failures, 0 errors
  - [x] 4.6.2. core/class/attached_object_spec — 1 file, 4 examples, 6 expectations, 0 failures, 0 errors
  - [x] 4.6.3. core/class/dup_spec — 1 file, 6 examples, 8 expectations, 0 failures, 0 errors
  - [x] 4.6.4. core/class/inherited_spec — 1 file, 9 examples, 16 expectations, 0 failures, 0 errors
  - [x] 4.6.5. core/class/initialize_spec — 1 file, 4 examples, 5 expectations, 0 failures, 0 errors
  - [x] 4.6.6. core/class/new_spec — 1 file, 15 examples, 29 expectations, 0 failures, 0 errors
  - [x] 4.6.7. core/class/subclasses_spec — 1 file, 8 examples, 16008 expectations, 0 failures, 0 errors
  - [x] 4.6.8. core/class/superclass_spec — 1 file, 3 examples, 8 expectations, 0 failures, 0 errors
- [ ] 4.7. core/module
  - [x] 4.7.1. core/module/alias_method_spec — 1 file, 23 examples, 39 expectations, 0 failures, 0 errors
  - [x] 4.7.2. core/module/ancestors_spec — 1 file, 9 examples, 14 expectations, 0 failures, 0 errors
  - [x] 4.7.3. core/module/append_features_spec — 1 file, 6 examples, 8 expectations, 0 failures, 0 errors
  - [x] 4.7.4. core/module/attr_accessor_spec — 1 file, 11 examples, 27 expectations, 0 failures, 0 errors
  - [x] 4.7.5. core/module/attr_reader_spec — 1 file, 9 examples, 20 expectations, 0 failures, 0 errors
  - [x] 4.7.6. core/module/attr_spec — 1 file, 13 examples, 45 expectations, 0 failures, 0 errors
  - [x] 4.7.7. core/module/attr_writer_spec — 1 file, 10 examples, 21 expectations, 0 failures, 0 errors
  - [x] 4.7.8. core/module/autoload_relative_spec — 1 file, 0 examples, 0 expectations, 0 failures, 0 errors (all examples guarded under ruby_version_is "4.1")
  - [x] 4.7.9. core/module/autoload_spec — 1 file, 79 examples, 192 expectations, 0 failures, 0 errors
  - [x] 4.7.10. core/module/case_compare_spec — 1 file, 3 examples, 15 expectations, 0 failures, 0 errors
  - [x] 4.7.11. core/module/class_eval_spec — 1 file, 20 examples, 33 expectations, 0 failures, 0 errors
  - [x] 4.7.12. core/module/class_exec_spec — 1 file, 5 examples, 6 expectations, 0 failures, 0 errors
  - [x] 4.7.13. core/module/class_variable_defined_spec — 1 file, 9 examples, 17 expectations, 0 failures, 0 errors
  - [x] 4.7.14. core/module/class_variable_get_spec — 1 file, 12 examples, 19 expectations, 0 failures, 0 errors
  - [x] 4.7.15. core/module/class_variable_set_spec — 1 file, 7 examples, 15 expectations, 0 failures, 0 errors
  - [x] 4.7.16. core/module/class_variables_spec — 1 file, 5 examples, 9 expectations, 0 failures, 0 errors
  - [x] 4.7.17. core/module/comparison_spec — 1 file, 5 examples, 8 expectations, 0 failures, 0 errors
  - [x] 4.7.18. core/module/const_added_spec — 1 file, 14 examples, 15 expectations, 0 failures, 0 errors
  - [x] 4.7.19. core/module/const_defined_spec — 1 file, 31 examples, 51 expectations, 0 failures, 0 errors
  - [x] 4.7.20. core/module/const_get_spec — 1 file, 45 examples, 70 expectations, 0 failures, 0 errors
  - [x] 4.7.21. core/module/const_missing_spec — 1 file, 5 examples, 5 expectations, 0 failures, 0 errors
  - [x] 4.7.22. core/module/const_set_spec — 1 file, 15 examples, 36 expectations, 0 failures, 0 errors
  - [x] 4.7.23. core/module/const_source_location_spec — 1 file, 42 examples, 65 expectations, 0 failures, 0 errors
  - [x] 4.7.24. core/module/constants_spec — 1 file, 11 examples, 30 expectations, 0 failures, 0 errors
  - [x] 4.7.25. core/module/define_method_spec — 1 file, 88 examples, 102 expectations, 0 failures, 0 errors
  - [ ] 4.7.26. core/module/define_singleton_method_spec
  - [ ] 4.7.27. core/module/deprecate_constant_spec
  - [ ] 4.7.28. core/module/eql_spec
  - [ ] 4.7.29. core/module/equal_spec
  - [ ] 4.7.30. core/module/equal_value_spec
  - [ ] 4.7.31. core/module/extend_object_spec
  - [ ] 4.7.32. core/module/extended_spec
  - [ ] 4.7.33. core/module/freeze_spec
  - [ ] 4.7.34. core/module/gt_spec
  - [ ] 4.7.35. core/module/gte_spec
  - [ ] 4.7.36. core/module/include_spec
  - [ ] 4.7.37. core/module/included_modules_spec
  - [ ] 4.7.38. core/module/included_spec
  - [ ] 4.7.39. core/module/initialize_copy_spec
  - [ ] 4.7.40. core/module/initialize_spec
  - [ ] 4.7.41. core/module/instance_method_spec
  - [ ] 4.7.42. core/module/instance_methods_spec
  - [ ] 4.7.43. core/module/lt_spec
  - [ ] 4.7.44. core/module/lte_spec
  - [ ] 4.7.45. core/module/method_added_spec
  - [ ] 4.7.46. core/module/method_defined_spec
  - [ ] 4.7.47. core/module/method_removed_spec
  - [ ] 4.7.48. core/module/method_undefined_spec
  - [ ] 4.7.49. core/module/module_eval_spec
  - [ ] 4.7.50. core/module/module_exec_spec
  - [ ] 4.7.51. core/module/module_function_spec
  - [ ] 4.7.52. core/module/name_spec
  - [ ] 4.7.53. core/module/nesting_spec
  - [ ] 4.7.54. core/module/new_spec
  - [ ] 4.7.55. core/module/prepend_features_spec
  - [ ] 4.7.56. core/module/prepend_spec
  - [ ] 4.7.57. core/module/prepended_spec
  - [ ] 4.7.58. core/module/private_class_method_spec
  - [ ] 4.7.59. core/module/private_constant_spec
  - [ ] 4.7.60. core/module/private_instance_methods_spec
  - [ ] 4.7.61. core/module/private_method_defined_spec
  - [ ] 4.7.62. core/module/private_spec
  - [ ] 4.7.63. core/module/protected_instance_methods_spec
  - [ ] 4.7.64. core/module/protected_method_defined_spec
  - [ ] 4.7.65. core/module/protected_spec
  - [ ] 4.7.66. core/module/public_class_method_spec
  - [ ] 4.7.67. core/module/public_constant_spec
  - [ ] 4.7.68. core/module/public_instance_method_spec
  - [ ] 4.7.69. core/module/public_instance_methods_spec
  - [ ] 4.7.70. core/module/public_method_defined_spec
  - [ ] 4.7.71. core/module/public_spec
  - [ ] 4.7.72. core/module/refine_spec
  - [ ] 4.7.73. core/module/refinements_spec
  - [ ] 4.7.74. core/module/remove_class_variable_spec
  - [ ] 4.7.75. core/module/remove_const_spec
  - [ ] 4.7.76. core/module/remove_method_spec
  - [ ] 4.7.77. core/module/ruby2_keywords_spec
  - [ ] 4.7.78. core/module/set_temporary_name_spec
  - [ ] 4.7.79. core/module/singleton_class_spec
  - [ ] 4.7.80. core/module/to_s_spec
  - [ ] 4.7.81. core/module/undef_method_spec
  - [ ] 4.7.82. core/module/undefined_instance_methods_spec
  - [ ] 4.7.83. core/module/used_refinements_spec
  - [ ] 4.7.84. core/module/using_spec
- [ ] 4.8. core/kernel
  - [ ] 4.8.1. core/kernel/Array_spec
  - [ ] 4.8.2. core/kernel/Complex_spec
  - [ ] 4.8.3. core/kernel/Float_spec
  - [ ] 4.8.4. core/kernel/Hash_spec
  - [ ] 4.8.5. core/kernel/Integer_spec
  - [ ] 4.8.6. core/kernel/Rational_spec
  - [ ] 4.8.7. core/kernel/String_spec
  - [ ] 4.8.8. core/kernel/__callee___spec
  - [ ] 4.8.9. core/kernel/__dir___spec
  - [ ] 4.8.10. core/kernel/__method___spec
  - [ ] 4.8.11. core/kernel/abort_spec
  - [ ] 4.8.12. core/kernel/at_exit_spec
  - [ ] 4.8.13. core/kernel/autoload_relative_spec
  - [ ] 4.8.14. core/kernel/autoload_spec
  - [ ] 4.8.15. core/kernel/backtick_spec
  - [ ] 4.8.16. core/kernel/binding_spec
  - [ ] 4.8.17. core/kernel/block_given_spec
  - [ ] 4.8.18. core/kernel/caller_locations_spec
  - [ ] 4.8.19. core/kernel/caller_spec
  - [ ] 4.8.20. core/kernel/case_compare_spec
  - [ ] 4.8.21. core/kernel/catch_spec
  - [ ] 4.8.22. core/kernel/chomp_spec
  - [ ] 4.8.23. core/kernel/chop_spec
  - [ ] 4.8.24. core/kernel/class_spec
  - [ ] 4.8.25. core/kernel/clone_spec
  - [ ] 4.8.26. core/kernel/comparison_spec
  - [ ] 4.8.27. core/kernel/define_singleton_method_spec
  - [ ] 4.8.28. core/kernel/display_spec
  - [ ] 4.8.29. core/kernel/dup_spec
  - [ ] 4.8.30. core/kernel/enum_for_spec
  - [ ] 4.8.31. core/kernel/eql_spec
  - [ ] 4.8.32. core/kernel/equal_value_spec
  - [ ] 4.8.33. core/kernel/eval_spec
  - [ ] 4.8.34. core/kernel/exec_spec
  - [ ] 4.8.35. core/kernel/exit_spec
  - [ ] 4.8.36. core/kernel/extend_spec
  - [ ] 4.8.37. core/kernel/fail_spec
  - [ ] 4.8.38. core/kernel/fork_spec
  - [ ] 4.8.39. core/kernel/format_spec
  - [ ] 4.8.40. core/kernel/freeze_spec
  - [ ] 4.8.41. core/kernel/frozen_spec
  - [ ] 4.8.42. core/kernel/gets_spec
  - [ ] 4.8.43. core/kernel/global_variables_spec
  - [ ] 4.8.44. core/kernel/gsub_spec
  - [ ] 4.8.45. core/kernel/initialize_clone_spec
  - [ ] 4.8.46. core/kernel/initialize_copy_spec
  - [ ] 4.8.47. core/kernel/initialize_dup_spec
  - [ ] 4.8.48. core/kernel/inspect_spec
  - [ ] 4.8.49. core/kernel/instance_of_spec
  - [ ] 4.8.50. core/kernel/instance_variable_defined_spec
  - [ ] 4.8.51. core/kernel/instance_variable_get_spec
  - [ ] 4.8.52. core/kernel/instance_variable_set_spec
  - [ ] 4.8.53. core/kernel/instance_variables_spec
  - [ ] 4.8.54. core/kernel/is_a_spec
  - [ ] 4.8.55. core/kernel/itself_spec
  - [ ] 4.8.56. core/kernel/kind_of_spec
  - [ ] 4.8.57. core/kernel/lambda_spec
  - [ ] 4.8.58. core/kernel/load_spec
  - [ ] 4.8.59. core/kernel/local_variables_spec
  - [ ] 4.8.60. core/kernel/loop_spec
  - [x] 4.8.61. core/kernel/match_spec — 1 file, 1 example, 1 expectation, 0 failures, 0 errors
  - [ ] 4.8.62. core/kernel/method_spec
  - [ ] 4.8.63. core/kernel/methods_spec
  - [ ] 4.8.64. core/kernel/nil_spec
  - [ ] 4.8.65. core/kernel/not_match_spec
  - [ ] 4.8.66. core/kernel/object_id_spec
  - [ ] 4.8.67. core/kernel/open_spec
  - [ ] 4.8.68. core/kernel/p_spec
  - [ ] 4.8.69. core/kernel/pp_spec
  - [ ] 4.8.70. core/kernel/print_spec
  - [ ] 4.8.71. core/kernel/printf_spec
  - [ ] 4.8.72. core/kernel/private_methods_spec
  - [ ] 4.8.73. core/kernel/proc_spec
  - [ ] 4.8.74. core/kernel/protected_methods_spec
  - [ ] 4.8.75. core/kernel/public_method_spec
  - [ ] 4.8.76. core/kernel/public_methods_spec
  - [ ] 4.8.77. core/kernel/public_send_spec
  - [ ] 4.8.78. core/kernel/putc_spec
  - [ ] 4.8.79. core/kernel/puts_spec
  - [ ] 4.8.80. core/kernel/raise_spec
  - [ ] 4.8.81. core/kernel/rand_spec
  - [ ] 4.8.82. core/kernel/readline_spec
  - [ ] 4.8.83. core/kernel/readlines_spec
  - [ ] 4.8.84. core/kernel/remove_instance_variable_spec
  - [ ] 4.8.85. core/kernel/require_relative_spec
  - [ ] 4.8.86. core/kernel/require_spec
  - [ ] 4.8.87. core/kernel/respond_to_missing_spec
  - [ ] 4.8.88. core/kernel/respond_to_spec
  - [ ] 4.8.89. core/kernel/select_spec
  - [ ] 4.8.90. core/kernel/send_spec
  - [ ] 4.8.91. core/kernel/set_trace_func_spec
  - [ ] 4.8.92. core/kernel/singleton_class_spec
  - [ ] 4.8.93. core/kernel/singleton_method_spec
  - [ ] 4.8.94. core/kernel/singleton_methods_spec
  - [ ] 4.8.95. core/kernel/sleep_spec
  - [ ] 4.8.96. core/kernel/spawn_spec
  - [ ] 4.8.97. core/kernel/sprintf_spec
  - [ ] 4.8.98. core/kernel/srand_spec
  - [ ] 4.8.99. core/kernel/sub_spec
  - [ ] 4.8.100. core/kernel/syscall_spec
  - [ ] 4.8.101. core/kernel/system_spec
  - [ ] 4.8.102. core/kernel/taint_spec
  - [ ] 4.8.103. core/kernel/tainted_spec
  - [ ] 4.8.104. core/kernel/tap_spec
  - [ ] 4.8.105. core/kernel/test_spec
  - [ ] 4.8.106. core/kernel/then_spec
  - [ ] 4.8.107. core/kernel/throw_spec
  - [ ] 4.8.108. core/kernel/to_enum_spec
  - [ ] 4.8.109. core/kernel/to_s_spec
  - [ ] 4.8.110. core/kernel/trace_var_spec
  - [ ] 4.8.111. core/kernel/trap_spec
  - [ ] 4.8.112. core/kernel/trust_spec
  - [ ] 4.8.113. core/kernel/untaint_spec
  - [ ] 4.8.114. core/kernel/untrace_var_spec
  - [ ] 4.8.115. core/kernel/untrust_spec
  - [ ] 4.8.116. core/kernel/untrusted_spec
  - [ ] 4.8.117. core/kernel/warn_spec
  - [ ] 4.8.118. core/kernel/yield_self_spec
- [ ] 4.9. core/basicobject
  - [ ] 4.9.1. core/basicobject/__id__spec
  - [ ] 4.9.2. core/basicobject/__send___spec
  - [ ] 4.9.3. core/basicobject/basicobject_spec
  - [ ] 4.9.4. core/basicobject/equal_spec
  - [ ] 4.9.5. core/basicobject/equal_value_spec
  - [ ] 4.9.6. core/basicobject/initialize_spec
  - [ ] 4.9.7. core/basicobject/instance_eval_spec
  - [ ] 4.9.8. core/basicobject/instance_exec_spec
  - [ ] 4.9.9. core/basicobject/method_missing_spec
  - [ ] 4.9.10. core/basicobject/not_equal_spec
  - [ ] 4.9.11. core/basicobject/not_spec
  - [ ] 4.9.12. core/basicobject/singleton_method_added_spec
  - [ ] 4.9.13. core/basicobject/singleton_method_removed_spec
  - [ ] 4.9.14. core/basicobject/singleton_method_undefined_spec

### Exceptions and Signals

- [ ] 4.10. core/exception
- [ ] 4.11. core/systemexit
- [ ] 4.12. core/signal
- [ ] 4.13. core/warning

### Numeric Types

- [ ] 4.14. core/integer
- [ ] 4.15. core/float
- [ ] 4.16. core/numeric
- [ ] 4.17. core/complex
- [ ] 4.18. core/rational
- [ ] 4.19. core/math

### Collections

- [ ] 4.20. core/array
- [ ] 4.21. core/hash
- [ ] 4.22. core/set
- [ ] 4.23. core/range
- [ ] 4.24. core/struct
- [ ] 4.25. core/data

### Strings and Patterns

- [ ] 4.26. core/string
- [ ] 4.27. core/symbol
- [ ] 4.28. core/regexp
- [ ] 4.29. core/encoding
- [ ] 4.30. core/matchdata

### Callables and Introspection

- [ ] 4.31. core/proc
- [ ] 4.32. core/method
- [ ] 4.33. core/unboundmethod
- [ ] 4.34. core/binding

### IO and Filesystem

- [ ] 4.35. core/io
- [ ] 4.36. core/file
- [ ] 4.37. core/dir
- [ ] 4.38. core/filetest
- [ ] 4.39. core/env

### Concurrency

- [ ] 4.40. core/thread
- [ ] 4.41. core/fiber
- [ ] 4.42. core/mutex
- [ ] 4.43. core/conditionvariable
- [ ] 4.44. core/queue
- [ ] 4.45. core/sizedqueue
- [ ] 4.46. core/threadgroup

### Other

- [ ] 4.47. core/gc
- [ ] 4.48. core/objectspace
- [ ] 4.49. core/random
- [ ] 4.50. core/time
- [ ] 4.51. core/process
- [ ] 4.52. core/marshal
- [ ] 4.53. core/tracepoint
- [ ] 4.54. core/refinement
- [ ] 4.55. core/builtin_constants

Fixes during Phase 4 work:
- **Scope leak in `evaluate_branch_value`** — `?` operator could skip `pop_scope()`, causing method parameters to become invisible in rescue blocks. Same bug fixed in pattern matching (`execute_match`, `evaluate_guard_with_bindings`, `evaluate_case_expression`).
- Any object as hash key (Ruby semantics) — `object_to_dict_key` and hash index assignment accept all types
- `Array#index`/`Array#find_index` with value and block forms
- `Array#clear`
- `__send__`/`send`/`public_send` — method dispatch by name
- `frozen?` — returns true for immutable types (booleans, nil, integers, floats, symbols, strings)
- `eql?` — value equality method
- `singleton_method` — raises `NameError` (singleton methods not supported)
- `TrueClass`/`FalseClass`/`NilClass` registered as proper classes
- `true.class` → `TrueClass`, `false.class` → `FalseClass`, `nil.class` → `NilClass`
- `TrueClass.allocate` raises `TypeError`, `TrueClass.new` raises `NoMethodError`
- Exception `class` method returns specific type class (e.g., `NameError` not generic `Exception`)
- `===` (case equality) separated from `==` — `Class === obj` checks type membership
- `def (true).foo; end` defines method on `TrueClass` (singleton method on literal)
- Bitwise operators on nil: `nil & x` → false, `nil | x` / `nil ^ x` → truthiness of x
- `Hash#default` returns nil
- `Hash#key?`/`has_key?` accepts any key type
- `nil.to_r`/`nil.rationalize` → `Rational(0, 1)`
- `nil.to_c` → `Complex(0, 0)`
- `Rational()` and `Complex()` kernel functions with value equality
- Nested class/module definitions inside `module` bodies stored as module constants (`Foo::Bar`)
- `alias_method` in class bodies handled at class definition time
- `to_sym` on String and Symbol
- `object_id` / `__id__` for all types
- `between?` method using `<=>` dispatch
- `singleton_class` returns receiver's class (stub)
- Comparison operators (`<`, `>`, `<=`, `>=`) dispatch to `<=>` for non-numeric types (Comparable protocol)
- Comparison type mismatch raises `ArgumentError` instead of `TypeError`
- Exception `status` field for `SystemExit`
- `Include` inside module body adds mixin to module
