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

- [ ] 4.1. core/true
- [ ] 4.2. core/nil
