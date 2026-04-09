# Ruby Spec Suite (mspec) Compatibility

Goal: Get the official Ruby spec suite (ruby/spec) running against Metorex via mspec, progressively passing more specs.

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
- `mspec/utils/options.rb` fully parses (521/521 lines). `mspec/utils/script.rb` partially parses — multiple nesting issues in deeper sections remain.

## Phase 4: Passing Specs

- [ ] 4.1. core/true
- [ ] 4.2. core/nil
