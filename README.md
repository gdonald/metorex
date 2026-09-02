# Metorex Programming Language

**METOREX** (**M**eta **O**bject **R**untime **E**xecution) is a programming language that combines the expressiveness of Ruby with the performance and safety of Rust. It features a unique **Code-as-Object** meta-programming system that exposes the AST as first-class runtime objects, enabling powerful DSL construction and runtime code manipulation.

⚠️ &nbsp;It's still very early in development.

🙂 &nbsp;[PRs](https://github.com/gdonald/metorex/pulls) and [new](https://github.com/gdonald/metorex/issues/new) [issues](https://github.com/gdonald/metorex/issues) are welcome.

###

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/gdonald/metorex/blob/main/LICENSE) [![CI](https://github.com/gdonald/metorex/workflows/CI/badge.svg)](https://github.com/gdonald/metorex/actions) [![codecov](https://codecov.io/gh/gdonald/metorex/graph/badge.svg?token=GQ4LA1VMRE)](https://codecov.io/gh/gdonald/metorex)

## Project Status

METOREX is currently in **active development** following a 4-phase roadmap:

- **Phase 1 (MVP)**: AST Interpreter with meta-programming core - *In Progress*
- **Phase 2**: Bytecode VM with reflection maturity - *Planned*
- **Phase 3**: Optimization, concurrency, and production features - *Planned*
- **Phase 4**: Advanced features (macros, WebAssembly, functional programming) - *Planned*

See [ROADMAP.md](ROADMAP.md) for detailed implementation plans.

## Key Features

### Core Language Features
- **Exception Handling**: Full begin/rescue/ensure with exception hierarchies and stack traces
- **Pattern Matching**: Powerful pattern matching with destructuring and guards
- **Built-in Testing**: Integrated test framework with assertions and test discovery
- **Traits/Interfaces**: Flexible polymorphism through trait system
- **Optional Type System**: Gradual typing with type inference for performance and safety
- **Advanced Collections**: Set, Deque, PriorityQueue, TreeMap, and immutable structures
- **Struct**: `Struct.new` builds member classes with accessors, `[]`/`[]=`, `to_a`, `to_h`, `each`, `each_pair`, `dig`, `values_at`, and value equality, with `keyword_init:` and a class-body block
- **Kernel Conversion Functions**: `Hash()`, `Integer()`, `Rational()`, and `String()`, with coercion through `to_hash` / `to_int` / `to_i` / `to_r` / `to_s` and `exception: false`
- **Rational Numbers**: the `5r` literal suffix, arithmetic and ordering against Integer, Float, and Rational, `to_r` on String, Integer, and Float, and results always in lowest terms and frozen
- **Numeric Literals**: decimal, `0x`/`0b`/`0o`/`0d` radix prefixes, bare-leading-zero octal, scientific notation, `_` digit separators, and the `r` rational suffix
- **String Subclasses**: `class Name < String` instances carry their characters and answer String's methods, comparing equal to a String with the same content
- **Runtime Class System**: Classes support inheritance, runtime method definition, instance variables, and class-level state
- **File Loading**: `require_relative` with extension auto-detection, deduplication, circular dependency handling, and shared scope

### Meta-Programming (Core Innovation)
- **Code-as-Object**: AST nodes are first-class objects manipulable at runtime
- **Runtime Method Definition**: `define_method` for dynamic behavior
- **Method Missing Hook**: `method_missing` intercepts calls to undefined methods with method name and arguments
- **Runtime Class Modification**: `remove_method`, `undef_method`, `alias_method`, `module_function`, `class_variable_set`, `class_variable_get`, `class_variable_defined?`, `class_variables` for dynamic class/module manipulation
- **Reflection and Introspection**: `class`, `instance_of?`, `is_a?`, `itself`, `respond_to?`, `methods`, `send`, `instance_variables`, `instance_variable_get`, `instance_variable_set`, `local_variables`, `__method__`, `__callee__`
- **AST Manipulation**: `eval` for runtime code execution, `parse` for AST inspection, runtime code generation via string evaluation
- **Block Execution**: Blocks are objects with `.call()` method; trailing `do...end` and `{...}` blocks captured implicitly via `&block` parameter with `block_given?` support
- **DSL Construction**: Build domain-specific languages naturally

### Standard Library
- **Networking**: HTTP client/server, WebSocket, TCP/UDP, TLS/SSL
- **Serialization**: JSON, XML, YAML, CSV, MessagePack
- **Cryptography**: Hashing, encryption, secure random, certificates
- **Concurrency**: OS threads, fibers, async/await, channels, atomics
- **Advanced Math**: Complex numbers, arbitrary precision, statistics

### Developer Experience
- **Documentation System**: Doc comments with automatic HTML generation
- **Debugger**: Full debugging with breakpoints and inspection
- **LSP Support**: Language Server Protocol for IDE integration
- **Build System**: Incremental compilation, profiles, and optimization
- **Linter & Formatter**: Code quality and style enforcement

## Core Philosophy and Identity

| Element                 | Description                                                                                                                       |
| :---------------------- | :-------------------------------------------------------------------------------------------------------------------------------- |
| **Foundation Language** | **Rust** (for VM safety and speed)                                                                                                |
| **Syntax Heritage**     | **Ruby** (block structure, optional parentheses)                                                                                  |
| **Primary Paradigms**   | **Full Object-Oriented**, **Imperative**, **Functional** (with ADTs and immutable structures)                                     |
| **Key Differentiator**  | **Code-as-Object (The Meta Core)**: The Abstract Syntax Tree (AST) is directly exposed as native, manipulable objects at runtime. |
| **Typing**              | **Dynamic by default**, with **optional static typing** and gradual type inference                                                |
| **Performance**         | **Bytecode VM** with **JIT compilation** for hot paths, built on Rust for safety                                                  |

## Syntax Overview

METOREX syntax prioritizes readability while minimizing keystrokes, combining elements from Ruby.

### Basic Syntax

See [examples/basic_syntax.rb](examples/oop/basic_syntax.rb)

### Exception Handling

See [examples/exception_handling.rb](examples/advanced/exception_handling.rb)

### Pattern Matching

See [examples/pattern_matching.rb](examples/advanced/pattern_matching.rb)

### Traits (Interfaces)

See [examples/traits.rb](examples/advanced/traits.rb)

### Optional Type Annotations

See [examples/type_annotations.rb](examples/advanced/type_annotations.rb)

## Meta-Programming: The Core Innovation

METOREX exposes the program's structure as native objects, eliminating the need for external `eval` functions.

### Code-as-Object Hierarchy

The parser converts source code into an in-memory graph of objects, defined in the Rust core and exposed in Metorex.

| Metorex Class Name   | Role                                                                                                              | Example of Manipulation                    |
| :------------------- | :---------------------------------------------------------------------------------------------------------------- | :----------------------------------------- |
| **`BlockStatement`** | **The Core Meta-Object.** Represents a sequence of code lines (a method body, loop body, or implicit code block). | `block.call` to execute the code.          |
| **`Assignment`**     | Represents `x = 10`.                                                                                              | `.target` to see the variable name.        |
| **`MethodCall`**     | Represents a function/method invocation.                                                                          | `.receiver` and `.args` for code analysis. |

### Implicit Block Capture and Execution

Methods can accept code blocks as objects.

See [examples/metaprogramming/blocks_as_objects.rb](examples/metaprogramming/blocks_as_objects.rb)

### Dynamic Method Definition

See [examples/dynamic_method_definition.rb](examples/advanced/dynamic_method_definition.rb)

### Building DSLs

See [examples/dsl_example.rb](examples/advanced/dsl_example.rb)

## Architecture

### Multi-Phase Execution Model

1. **Phase 1 (MVP)**: Direct AST interpretation for rapid development
   - Lexer → Parser → AST → Interpreter
   - Full meta-programming capabilities
   - Exception handling, pattern matching, testing

### Runtime Components (MVP)

- `VirtualMachine` (`src/vm.rs`) seeds the AST interpreter with the environment stack, global object registry, call stack, heap placeholder, and built-in class initialization.

2. **Phase 2**: Bytecode compilation for performance
   - AST → Bytecode Compiler → VM
   - Reflection and runtime definition
   - Traits and advanced OOP

3. **Phase 3**: Production optimizations
   - JIT compilation for hot paths (LLVM)
   - Full concurrency support (threads, channels, atomics)
   - Optional type system with inference
   - Comprehensive standard library

4. **Phase 4**: Advanced features
   - Macro system for compile-time metaprogramming
   - Algebraic data types and functional features
   - WebAssembly compilation target
   - Security features and sandboxing
   
   ## Design Principles

| Principle             | Implementation                                                                         |
| :-------------------- | :------------------------------------------------------------------------------------- |
| **Syntax Simplicity** | Non-whitespace sensitive with mandatory `end` blocks. No colons, optional parentheses. |
| **OO Purity**         | Everything is an object rooted in `Object` class. No standalone functions.             |
| **Meta-First**        | AST is always accessible as first-class objects. Code can inspect and modify itself.   |
| **Gradual Typing**    | Dynamic by default, optional static types for performance. Best of both worlds.        |
| **Performance**       | Rust-based VM with bytecode compilation and JIT for hot paths.                         |
| **Safety**            | Exception handling, memory safety from Rust, optional sandboxing.                      |
| **Concurrency**       | Multiple models: fibers, async/await, OS threads, channels. Choose the right tool.     |
| **Productivity**      | Built-in testing, documentation, linting, formatting. Everything you need included.    |

## Standard Library Highlights

### Networking

See [examples/networking.rb](examples/advanced/networking.rb)

### Concurrency

See [examples/concurrency.rb](examples/advanced/concurrency.rb)

### Serialization

See [examples/serialization.rb](examples/advanced/serialization.rb)

## Roadmap Highlights

See [ROADMAP.md](ROADMAP.md) for complete details.

### Phase 1: MVP (In Progress)
- Lexer and Parser
- AST Interpreter
- Expression evaluation (arithmetic, collections, indexing)
- Logical operators (`&&`, `||`) with short-circuit evaluation
- Logical NOT operator (`!`)
- Scope resolution (`::`) for class constants
- Global variables (`$variable`)
- Method dispatch for built-in objects
- Meta-programming core
- Exception handling
- Pattern matching (`case/when` and `case/in` with Ruby 2.7+ `=> name` binding)
- Keyword arguments (`def method(name:, age: 10)` and `method(name: "Bob")`)
- Operator method names (`def +(other)`, `def ==(other)`, `def [](key)`, `def []=(key, value)`)
- Module and mixin support (`module`, `include`, `extend`)
- `define_method` for dynamic method definition on classes, taking a block, a Proc, a `Method`, or an `UnboundMethod`. It returns the method name as a Symbol, inherits the current `private`/`public` visibility when called from inside the target module, always makes `initialize` private, fires the `method_added` hook, and raises `FrozenError` on a frozen module
- `define_singleton_method` for defining a method on a single object's singleton class, accepting the same block, Proc, `Method`, or `UnboundMethod` bodies as `define_method`
- Bodies installed by `define_method` follow lambda control flow: `return`, `break`, and `next` finish the method with a value, and `redo` re-runs it
- Proc and Method objects: `Kernel#proc`, `Kernel#lambda`, `Proc.new`, `Proc#lambda?`, `Symbol#to_proc`, `Method#to_proc` (which stays bound to its original receiver), `Method#unbind`, and `Method#owner` (returns the defining module)
- `Object#method` converts its name argument with `#to_str`, and builds a `method_missing` dispatcher for a name the object claims through `respond_to_missing?`
- `Object#public_method` does the same lookup but raises `NameError` for a private or protected name, and asks `respond_to_missing?` without the private flag
- Procs and lambdas are distinct kinds. A lambda checks its arity and its `return` returns from the lambda, while a proc pads missing arguments with nil, drops extras, and its `return` returns from the method that created it
- `Kernel#loop` runs its block until `break` (whose value the loop returns) or until the block raises `StopIteration` or one of its subclasses. Every other exception propagates
- `method_missing` hook for intercepting undefined method calls
- Runtime class modification: `remove_method`, `undef_method`, `alias_method`, `module_function`
- Constant visibility on a module receiver: `private_constant`, `public_constant`, and `deprecate_constant` (which returns the receiver and raises `NameError` for an undefined name). Reading a deprecated constant through `::`, `const_get`, or `remove_const` warns once the `Warning[:deprecated]` category is switched on
- `Warning[:category]` and `Warning[:category] = bool` for reading and setting the warning category switches. Like MRI, `:deprecated` starts off
- Class variables: `class_variable_set`, `class_variable_get`, `class_variable_defined?`, `class_variables` (lookup walks included modules and superclasses; `class_variables(false)` lists only own names)
- Module ancestry comparison with `<=>`: `-1` when the receiver is a descendant or includer of the argument, `+1` when it is an ancestor or included-by, `0` when they are the same module, and `nil` when unrelated or the argument is not a module
- Reflection: `class`, `instance_of?`, `is_a?`, `itself`, `respond_to?`, `send`, `instance_variables` (Symbols, in declaration order), `instance_variable_get`, `instance_variable_set`, `local_variables`
- `Object#methods` reports `def obj.name`, `class << obj`, `define_singleton_method`, and the modules `extend` attached, leaving out private ones and anything `undef_method` removed
- `Symbol` is its own class rather than an alias of `String`, keeping String's character-level methods (`length`, `upcase`, `start_with?`)
- `Array#&` and `Array#|` for intersection and union, both dropping duplicates
- `!~` dispatches `=~` on the receiver and negates the result, raising `NoMethodError` when the receiver has no `=~`
- `object_id` identifies a reference type by its address and an immediate by its value, so two equal Symbol, String, Integer, or Float literals share an id
- Hashes iterate in insertion order. A reassigned key keeps its position and a deleted one leaves the rest in place
- `Hash#each_pair`, Ruby's alias for `Hash#each`
- `Object#public_methods`, `Object#private_methods`, and `Object#protected_methods` report the methods of that visibility, including those a `class << obj` or `extend` supplied, and, unless passed false or nil, the ancestors' and mixins'. On a class they walk the class-method chain
- `Integer#divmod` returns the floored quotient and the modulus, with the signs following the divisor
- `Kernel#rand` draws a Float in [0, 1) with no argument, an Integer below a given bound (whose sign it ignores), or a value from a Range, answering nil for a backwards one. It converts other arguments with `#to_int`
- `Kernel#srand` installs a seed and answers the one it replaced, picking a seed of its own when given none. It converts its argument with `#to_int`, and the same seed repeats a whole sequence
- `Kernel#readline` reads a line and raises `EOFError` at end of input, where `gets` answers nil. `Kernel#readlines` collects every remaining line into an Array
- `Kernel#remove_instance_variable` takes a variable off an object and answers what it held, raising `NameError` for one that is not defined and `FrozenError` on a frozen receiver
- `Kernel.instance_methods` lists Kernel's native methods, with the private ones reported as private
- `respond_to?` falls back to `respond_to_missing?` for a name the lookup missed, passing along the private flag it was given. Every object carries a default `respond_to_missing?` that answers false
- `Method#owner` answers the module itself for a native Kernel method, not its name
- A class reports and enforces visibility on its natively-implemented methods, so `private_class_method :new` makes `respond_to?(:new)` false and `Klass.new` raise `NoMethodError`
- `singleton_class` answers `NilClass`, `TrueClass`, or `FalseClass` for those three objects, raises `TypeError` for an Integer, Float, or Symbol, and is frozen when the object is
- `-"str"` and `+"str"` both answer the string, matching Ruby's deduplicated and mutable forms
- `singleton_method` looks only at the singleton layer: a `def obj.name`, a class method, and the modules `include`, `prepend`, or `extend` attached. A method the object's class defines raises `NameError`
- `singleton_methods(all = true)` reports the same layer as names, adding what the ancestors' singleton classes supply. Passing false leaves out both the ancestors and the modules `extend` attached
- `extend self` inside a module body, and `extend Mod` at the top level
- `ary[range]` slices an Array, counting a negative bound from the end and answering nil for a start past the end
- `sprintf` and `format` convert their format argument with `#to_str` and raise `TypeError` otherwise, and `%s` renders a Symbol without its colon
- `Float::INFINITY`, `NAN`, `EPSILON`, `MAX`, `MIN`, `DIG`, and `MANT_DIG`
- `tap`, `then`, and `yield_self` raise `LocalJumpError` when given no block, and `throw` raises `ArgumentError` for the wrong argument count
- `Numeric` is a real superclass of `Integer` and `Float`
- An Integer and a Float compare against each other, so `(0...1).include?(0.38)` and `0.38 <=> 0` answer correctly
- `class << target = value` assigns first, then opens the singleton class of what was assigned
- `%i[a b c]` and `%i(a b c)` build an Array of Symbols, alongside `%w` for Strings
- `Kernel#raise` is a method as well as a keyword, so `send(:raise, ...)`, `Kernel.raise`, `method(:raise)`, and a singleton that makes it public all reach it. A bare `raise` with nothing to re-raise gives `RuntimeError: unhandled exception`
- `=~` and `!~` match a Symbol against a Regexp on the characters it is named with
- `Kernel#proc` is reachable through `send` and hands back an existing Proc unchanged, keeping a lambda a lambda. Without a literal block it raises `ArgumentError`
- `equal?` compares reference types by address, so two Procs, Sets, or Exceptions are only equal when they are the same object
- An array literal gathers trailing `key: value` or `key => value` pairs into one Hash as its last element, so `[1, a: 2, b: 3]` is `[1, {a: 2, b: 3}]`
- `Kernel#p` writes each argument's `inspect` on its own line, honoring a user-defined `inspect`, and returns the argument, the argument list, or nil for none
- `puts`, `print`, `p`, and `warn` write through `$stdout` and `$stderr`, so assigning an object with its own `write` captures the output. `print` with no arguments writes `$_`
- `puts` and `print` render with `to_s`, so a Symbol prints without its colon, while `p` and `inspect` keep it
- A bare method call reaches the method on `self` before a same-named Kernel function, so a class defining `to_s` can call it bare from another of its methods
- `Kernel#trace_var` runs a hook every time the named global is assigned, taking the hook as a block, a Proc, or a String of code to evaluate, and raising `ArgumentError` when given none. `Kernel#untrace_var` drops every hook on a global, or just the one it is handed. A `:$name` symbol names the global for both
- `Kernel#warn` writes through `Warning.warn`, which is defined in Ruby so a program can replace it, and stays silent while `$VERBOSE` is nil. Each argument warns on its own line, as does each element of an Array argument, and a message already ending in a newline keeps just the one. `uplevel:` prefixes `path:line: warning: ` taken from that many frames out, `category:` converts through `to_sym`, and a negative uplevel raises `ArgumentError`
- `**hash` in a call passes keyword arguments rather than a positional Hash, so an empty Hash contributes no argument at all
- A required parameter written after an optional one binds from the end of the argument list: `def pad(prefix = "<", value)` called with one argument fills `value`
- `Method#source_location` answers `[path, lineno]`, and a bare `method(:name)` inside an instance method resolves against `self`
- `Enumerator` steps through the values a method yields: `to_enum`/`enum_for` build one over any method that yields, `next` and `peek` walk it, `rewind` restarts it, and running past the end raises `StopIteration`. A method that yields answers one when called without a block, which is what `then` and `yield_self` return
- A method body and a class body each own their locals. An assignment inside one defines a new local there rather than reaching a same-named variable outside it, and a block still sees and assigns the locals of the scope it was written in
- `send`, `__send__`, and `public_send` raise `ArgumentError` with no method name and `TypeError` for a name that is neither a Symbol nor a String. `BasicObject` reports its own instance methods, `__id__` and `__send__` among them
- `super` from a method defined in an anonymous module reaches the next module in the chain, because each running method records the module it was defined in rather than being placed by name
- A splat inside an array literal splices its elements in place, so `[first, *rest]` flattens the one level Ruby flattens
- A class descending from `BasicObject` does not reach top-level constants, since Ruby finds those among Object's own. A leading `::` reads the top level directly, in a reference, in an `include`, and on the left of an assignment
- `Object.constants` lists every top-level constant, `BasicObject` holds the constant naming itself, and `instance_of?` reports a Class as an instance of `Class` and a Module of `Module`
- Integer bit operations: `<<` and `>>` shift (a negative count shifts the other way, and a count past the word width leaves 0 or the sign bit), `~` complements, and `bit_length` counts the bits a value needs. `:>>` and `:~` are symbol names like any other operator
- The default `initialize` takes no arguments, so `new` given any raises `ArgumentError`. Every arity error reads the way Ruby's does: `wrong number of arguments (given 0, expected 2)`, `(given 0, expected 1..2)` when a parameter has a default, and `(given 0, expected 1+)` when one is variadic
- Integers are arbitrary precision. A literal, a sum, a product, or a power past the machine word keeps its exact value rather than saturating or turning into a Float, and a result that fits again narrows back, so `(2 ** 64) - (2 ** 64)` is the same `0` any other expression produces. Comparison, sorting, `divmod`, the shifts, `abs`, `bit_length`, `Float#to_i`, `Kernel#Integer`, and `Rational` are all exact at any size. Two separately built values of the same large size are separate objects, as Ruby's are
- Dividing by zero raises `ZeroDivisionError` reading `divided by 0`
- `expr rescue fallback` answers the fallback when the expression raises a `StandardError`. On the right of an assignment the modifier binds to the value, so `value = risky rescue nil` assigns nil
- `instance_eval` takes source as well as a block: either runs with `self` bound to the receiver. The block form yields the receiver and takes no other arguments, the source form takes one to three
- `raise(SomeError, "message")` parses with parentheses, not only in its paren-less form
- `instance_exec` raises `LocalJumpError` without a block, reports an arity of -1, and refuses a `def` in a block run against an immediate, where a singleton method cannot exist
- A call that visibility refuses reaches a user-defined `method_missing` before raising, the way Ruby's does, and `super` from an override of it raises the NoMethodError the default would
- `NoMethodError` and `NameError` carry `#name` and `#receiver`, so a rescue can tell which method was called and on what
- `!=` is the negation of `==`, so a class that defines only `==` gets both, and `send(:!=, other)` reaches the same definition
- `singleton_method_added` fires for every way a singleton method arrives: `def obj.name`, a `class << obj` body, `alias`, `alias_method`, `define_method`, and `define_singleton_method`. Undefining the hook makes the next definition raise `NoMethodError`, or reach `method_missing` when one is defined
- A `class << obj` body answers with its own last value rather than re-running the last statement, so a side effect there happens once
- A `def` nested inside a block or a `begin` within a class body installs on that class, where it used to be an internal error
- `singleton_method_removed` fires when a singleton method goes, and `remove_method` in a `class << Klass` body reaches a method that `def Klass.name` put there
- `singleton_method_undefined` fires the same way for `undef_method`, which retires the method so `respond_to?` answers false and a call raises `NoMethodError`
- `Exception#to_s` is the message alone, and the class name when there is no message. A message argument that is not a String is rendered with `to_s`, which any object may define
- A bare `raise` followed by a trailing comment is still the re-raise form
- `Exception#backtrace` is nil until the exception is raised, then answers the same Array every time, so an update through it sticks. `Exception#set_backtrace` takes nil, a String, or an Array of Strings, keeping the very Array it is handed, and refuses anything else with a `TypeError`
- A rescue clause sets `$!` to the exception it is handling and `$@` to that exception's backtrace
- `Exception#backtrace_locations` answers `Thread::Backtrace::Location` objects carrying a `path` and a `lineno`, nil until the exception is raised, and the same Array on every call. `set_backtrace` accepts those objects as well as Strings
- `Array#each_with_index` yields each element with its position, and answers an Enumerator without a block
- The `Errno` namespace is populated: every `Errno::EXXX` is a subclass of `SystemCallError` carrying the platform's own number in its `Errno` constant, taken from libc rather than a table. `SystemCallError.new(message, errno)` answers the class that number names, and `#errno` reads it back
- `===` on an object reaches a user-defined `===` or `method_missing` before the default, and `:===` is a symbol name like any other operator
- `Exception#cause` is the exception a rescue clause was handling when this one was raised, set once and never to the exception itself. An error the interpreter raises inside a rescue body records what it followed just as an explicit `raise` does
- `is_a?` and `kind_of?` walk an exception's real ancestry, so a rescued `ZeroDivisionError` answers true for `StandardError`
- `Exception#detailed_message` decorates the message with the class name, stands in with `unhandled exception` or the class name for an empty one, and takes a `highlight:` keyword. `Exception#full_message` renders the backtrace with it, `order:` deciding which end the message sits at, and honors a class that defines its own `detailed_message`
- An exception carries the class it was built from, so an anonymous `Class.new(RuntimeError)` subclass reports it through `#class`, `is_a?`, and `===`
- `def exception.name` installs a singleton method, and each exception gets its own singleton class
- An exception subclass runs its own `initialize` and carries the instance variables it sets, so `attr_reader` on one works. `dup` copies that state along with the message, backtrace, and cause, calls `initialize_copy`, and leaves the singleton class behind
- Two exceptions are equal when they share a class, a message, and a backtrace, so a copy equals its original
- Each `Errno::EXXX` carries the message its number stands for, so `Errno::EINVAL.new.message` reads `Invalid argument`. A custom message and location are appended as `<default> - <custom>` and `<default> @ <location> - <custom>`, and a subclass inherits the default
- `Exception#exception` answers self with no argument or when handed self, and otherwise a copy carrying the new message, without re-running `initialize`. `Exception.exception` is another name for `new`
- An exception built with no message reports its class name, where one built with an empty message reports that. A bare `raise` makes the second kind, and the "unhandled exception" wording belongs to the uncaught report
- Modifying a frozen object raises `FrozenError` naming the class and inspecting the object, and the error carries it as `#receiver`, which `FrozenError.new` also takes by keyword. When inspecting would itself modify the object, the message shows `...`
- An Array, Hash, or Set can be frozen, and every method that changes one in place refuses a frozen receiver
- A backtrace entry reads `file:line:in 'label'`, the way Ruby's does. `Exception#full_message` appends the cause chain after the exception's own report
- A block parameter list takes `*`, `**`, and `&` without naming them, so `{ |**| }` accepts and discards keywords
- `String#lines` splits on the line separator, keeping it on each piece
- An Array index past either end answers nil rather than raising, and a negative one counts back from the end
- The built-in exception hierarchy is complete, rooted at Object the way every class is. `NoMemoryError`, `SecurityError`, `SystemStackError`, `FiberError`, `ThreadError`, and `ClosedQueueError` are defined, and `Interrupt` sits under `SignalException`
- `Exception#inspect` reads `#<ClassName: message>`, using whatever `to_s` answers, and the class name alone when that is empty
- `Signal.list` and `Signal.trap`, with `Process.kill` running the handler in force when the target is this process. The default disposition raises `Interrupt` for SIGINT and `SignalException` elsewhere, both answering `signo` and `signm`
- `IO::WaitReadable` and `IO::WaitWritable`, with the four `IO::EAGAINWaitReadable`-style classes that pair them with `Errno::EAGAIN`. The `EWOULDBLOCK` spellings name the same classes, since the two errno values match
- `KeyError.new` takes `receiver:` and `key:`, read back through `#receiver` and `#key`. A KeyError with no key recorded raises ArgumentError from `#key`, the way Ruby does
- `LoadError#path` answers the feature that could not be loaded, and nil on a LoadError raised by nothing in particular. A `require_relative` of a missing file raises LoadError rather than RuntimeError
- `Exception#message` dispatches `to_s`, so a subclass or a singleton that redefines `to_s` decides its message. `raise SomeClass` instantiates the class, which keeps a subclass `initialize` and `to_s` in force
- `NameError#name` is set on every path that raises one: an undefined variable or method, a constant, and an unset class variable. `instance_variable_get` and `class_variable_get` report back the very name object they were handed
- Class variables are looked up through the superclass chain, and reading one that was never assigned raises NameError instead of answering nil
- `NameError.new` takes a name as its second argument and a `receiver:` keyword, and `#dup` carries both to the copy. An undefined name records the object the lookup was made on
- `NoMethodError#args` answers the arguments the failed call was made with, and `NoMethodError.new` takes them as a third argument
- `String#*` repeats a string, raising ArgumentError on a negative count
- `NameError#receiver` is set on every path that raises one: a method call, a bare or namespaced constant, an unset class variable, and `instance_variable_get` / `class_variable_get`. Asking an exception that has none raises ArgumentError, as Ruby does
- `StopIteration#result` answers what the underlying `each` returned once an Enumerator runs out
- `Thread::Backtrace::Location` answers `label`, `base_label`, and `absolute_path` alongside `path` and `lineno`, and renders as `path:lineno:in 'label'`
- `SignalException.new` is named by the signal it stands for, taking a number, a name, or a symbol with or without the `SIG` prefix, with a second argument replacing the name. An argument that names no signal raises ArgumentError
- A bare `rescue` catches StandardError rather than everything, so an Exception outside that family goes past it
- `StringIO` collects what is written to it and hands back the string, and reads through it a line or a length at a time
- `printf` writes a formatted string to `$stdout`, or to an IO given as its first argument
- `require` of a library metorex provides itself, such as `stringio`, answers without looking for a file
- `File.new` opens a file the way `File.open` does when given no block
- `pp` prints the same inspect form `p` does and answers its argument the same way
- A Hash renders the way Ruby shows one: a Symbol key as `name: value` and every other kind as `key => value`, with the bookkeeping entries left out
- Reassigning `$stdout` to a File handle sends `p`, `puts`, and `print` there
- `Array#inspect` renders each element through its own `inspect`, so an object that defines one is shown the way it asks to be
- `File#read` takes a length and leaves the rest for the next read
- `Kernel#open` opens a file by path, taking one from anything answering `to_path` or `to_str`, and hands it to a block when given one. An argument answering `to_open` is asked to open itself and its answer is what comes back
- `File#gets` and `#readline` read a line at a time, and `#read` picks up where they left off
- The `File::CREAT` family of open flags
- `Enumerator.new { |yielder| ... }` builds an enumerator from a generator block, with `yielder <<` and `yielder.yield` collecting what it produces
- `loop` without a block answers an enumerator that yields forever and reports `Float::INFINITY` for its size. With a block, a StopIteration ends it and the loop answers the result the finished iterator carried
- `load` runs the file every time and leaves `$LOADED_FEATURES` alone, while `require` answers false only for a file that list still names. Both take a path from anything answering `to_path` or `to_str`, expand a leading `~` against `ENV["HOME"]`, and name an absolute or `./`-relative path outright rather than searching `$LOAD_PATH`. A file that does not exist or cannot be read raises LoadError
- `load(path, true)` runs the file inside a fresh anonymous module and `load(path, SomeModule)` inside that one, so its constants and top-level methods land there rather than on Object
- `$LOAD_PATH` entries may be objects answering `to_path`
- `File::Separator` and its siblings, `File.chmod`, and `Process.euid` / `Process.uid`
- An endless definition, `def name = expression`, defines a method whose body is that expression, with or without parameters and for a class method as readily as an instance one
- `Object#inspect` shows the instance variables alongside the class and address, each rendered as `inspect` would. An `instance_variables_to_inspect` method chooses which to show, nil from it means all of them, and anything else raises TypeError
- A format string reads `%{name}` and `%<name>` from the Hash it was given, raising KeyError for a name that Hash does not carry. With `$VERBOSE` on, arguments the format never reached are pointed out, and a keyword Hash is not counted among them
- `format` and `sprintf` are private instance methods of Kernel, and `Kernel.format` names the same one
- `fork` splits the process: the child answers nil, or runs the block and exits with its status, while the parent answers the child's process id. Every thread from the parent is marked finished in the child
- `Process.wait`, `.waitpid`, `.wait2`, `.waitpid2`, and `.waitall` wait for a child and record `$?`. `Process.exit`, `.exit!`, and `.abort` end the process the way the bare forms do
- `Thread.current` answers the main thread outside any thread block, and `Thread#kill` marks a thread finished so `alive?` reports false
- `exit` reads its status from an Integer, a boolean, a truncated Float, or anything answering `to_int`, and refuses the rest with TypeError. `exit!` skips the `at_exit` handlers and any `ensure` clause
- `at_exit` handlers run before an uncaught exception is reported, so a handler calling `exit!` replaces both the report and the status
- `abort`, `exit`, and `exit!` are reachable with an explicit receiver on any object, which is what a class makes public with `public :exit`
- `self` at the top level is `main`, rather than an undefined name
- `exec` replaces the process with the command, so nothing after it runs. A command with nothing for the shell to do is run directly, so a missing program raises `Errno::ENOENT` rather than becoming the shell's own exit status. `Kernel.exec` names the same method, and it is one of Kernel's private instance methods
- `def foo(...)` collects every argument and `foo(...)` passes them all on, alongside the bare `*`, `**`, and `&` forwarding forms. A `...` with an operand after it is still a beginless range
- A heredoc opens with a bare `<<TERMINATOR` as well as `<<-` and `<<~`, wherever a value is expected, so `<<TEXT.upcase` reads the heredoc and calls on it while `array << value` stays the shovel operator
- A lambda literal takes part in the expression around it, so `-> { 5 }.call == 5` compares rather than stopping at the call
- `clone` copies the singleton class along with the object, so a method defined on the original answers on the copy. It carries the original's frozen state, or whatever `freeze:` names, and refuses any other value for it with ArgumentError. `initialize_clone` is called with the keyword it was given
- A method that declares no keyword parameters counts a trailing keyword hash as an ordinary positional argument, so passing one to a single-parameter method raises ArgumentError
- `super` inside a singleton method reaches the class's own copy of that method
- `IO.popen` takes an argv Array as readily as a command String, and its handle can be written to as well as read: what is written reaches the child when the input is closed, and the child is waited for once the block returns
- The `-n` flag runs the program once for each line of standard input, with the line in `$_`
- `chomp` and `chop` with no receiver rewrite `$_` in place, `chomp` taking its separator from `$/`. `Kernel.chomp` and `Kernel.chop` do the same, and both are private methods of Kernel
- `Kernel.private_method_defined?` reports the Kernel methods that live in the native dispatch tables rather than in a method map
- `caller` reports each frame as `file:line:in 'label'`, taking the same start and length or Range that `caller_locations` does. A block is named by the scope holding it, so a block written at top level reads `block in <main>`
- `puts` writes an Array a line per element, however deeply nested, and a line of its own for an empty one. A string already ending in a newline is not given a second
- A block's body belongs to the file it was written in, so a backtrace entry for a call made from it names that file wherever the block is called from
- `caller_locations` takes a start and length or a Range, including endless, beginless, and negative-ended ones. Omitting more locations than there are answers nil and omitting exactly as many answers an empty Array. Level 0 is the line the call sits on, each location names the file its frame was called from, and `caller_locations` is one of Kernel's private instance methods
- A backtick literal runs its command through the shell and answers what it wrote to stdout, interpolating the way a double-quoted string does. The command's stderr passes through, `$?` reports how it ended, and a command the shell cannot find raises `Errno::ENOENT`. `Kernel.\`` and the `:\`` symbol name the same method
- `Process::Status` is a constant on Process, answering `stopped?`, `stopsig`, and `pid` alongside its other readers
- `Encoding.default_external` is remembered and reported, and `Encoding::SHIFT_JIS` is defined
- `String#b`
- `autoload` and `autoload?` are reachable by name, registering on Object where top-level constants live, and are listed among Kernel's private instance methods
- A file reached by `load` runs at top level, so a `def` in it belongs to Object rather than to whatever class or module body called the load
- `at_exit` registers a handler and answers it, raising ArgumentError when given no block. Handlers run in reverse order of registration once the program is over, however it ends, and one registered inside a handler runs right after it. A handler still runs when an earlier one raised, `exit` inside one settles the status, and `$!` there is the exception that ended the program
- `rescue => $global` binds the rescued exception to a global variable
- `$?` answers the status of the last child process waited for
- `-r` and `-I` accept their value attached, as `-rfoo`, and `-r` takes an absolute or `./`-relative path outright
- `__dir__` answers the real directory holding the running file, expanding a relative script path, and nil where no file stands behind the code. `eval` with a filename reports that file's directory, and eval through a binding reports nil
- `Dir.chdir` changes the working directory, restoring the previous one after a block and answering what the block returned
- `__FILE__` reports the path the main script was named by on the command line, and constant source locations record the same spelling
- `File.expand_path` expands a relative base against the working directory, so its answer is always absolute
- `Kernel#Float` reads a String strictly: a sign, `_` only between digits, an optional fraction and `e` exponent, and the `0x` hexadecimal form with a `p` binary exponent. Bad text raises ArgumentError, nil raises TypeError, a Complex with an imaginary part raises RangeError, and `exception: false` answers nil
- `Float#nan?`, `#finite?`, and `#infinite?`, which answers the sign of an infinity and nil otherwise
- A Float renders with its fractional part, so `1.0` reads as "1.0" rather than "1"
- Two infinities of the same sign are equal, and a NaN is identical to itself through `equal?` while equal to nothing through `==`
- `Complex` answers `real`, `imaginary`, `to_s`, `inspect`, and `==`, where a complex with no imaginary part equals the plain number it holds. `Complex.polar` and `Complex.rectangular` build one from either pair of components
- `Kernel#Complex` reads a String literal in every form Ruby accepts: integers, fractions, floats, scientific notation, the `i`/`I`/`j`/`J` units, `a+bi`, `m@a` polar form, and `_` digit separators. Bad text raises ArgumentError, a non-number raises TypeError, and `exception: false` answers nil instead
- An Integer equals the Float holding the same value, so `1 == 1.0` is true, in an Array or Hash comparison as much as on its own
- `defined?` on a method call answers nil when the receiver does not answer to that method, rather than reporting every call as a method
- `Encoding::CompatibilityError`, `Encoding::UndefinedConversionError`, and `Encoding::InvalidByteSequenceError` are defined, under StandardError
- `Kernel#Array` puts its argument through `to_ary` and then `to_a`, either of which may be private. One answering nil moves on to the next, one answering a non-Array raises TypeError, and an argument with neither is wrapped in a one-element Array
- A lambda literal is a receiver like any other, so `-> value { value * 2 }.call(3)` chains onto it, with or without parentheses around the parameters and in the `do` form
- `ruby2_keywords` raises NameError for a name no method answers to, and warns rather than applying when the method takes keywords or has no bare `*args` splat
- `refine` takes a module as readily as a class, requires a block, and registers the refinement before the block runs, so calls inside it and every sibling refinement in the same module are already in force
- `Hash#map` and `#collect` yield each key and value and answer an Array of what the block returned
- `String#dump` renders a string as source that reads back as itself, escaping control and non-ASCII characters and the `#` that would start an interpolation
- A prepended module's visibility is the one in force for the methods it supplies, so `private :name` on the class does not restrict a public method the prepended module defines under that name
- `prepend` puts a module ahead of the class in the ancestry, so its methods shadow the class's own and `super` from one reaches the class's copy. Constants, `ancestors`, `is_a?`, `constants`, and `singleton_method` all read the prepended module in that position, and a module prepended in one place still appears again where a superclass or an include already carried it
- `alias_method` records the aliasing class as the alias's owner, even when the method it copies came from a prepended or included module
- `super` with nothing above the defining class raises NoMethodError rather than a plain error
- `Array#take` and `Array#drop` split an array at a count, raising ArgumentError on a negative one
- `IO.popen` runs a command through the shell and hands back a handle answering `read`, `pid`, `close`, and `closed?`, in the block form too. `err: [:child, :out]` folds the child's stderr into what `read` returns
- `Process.last_status` answers a `Process::Status` for the last child waited for, reading back through `exited?`, `exitstatus`, `signaled?`, `termsig`, `success?`, and `to_i`
- Reopening `Module` or `Class` adds a method every class and module answers, which `respond_to?` reports too. A `const_added` defined that way is the hook for each of them
- `const_added` fires for a top-level class, module, or constant, with Object as the receiver, and only the first time the constant is defined
- `exit` raises SystemExit rather than ending the process outright, so an `ensure` block runs and a `rescue SystemExit` sees it, reading the code through `#status` and `#success?`. `exit!` ends the process immediately. An uncaught SystemExit exits with the status it carries
- A rescue clause places an exception by its class chain, which reaches a namespaced or anonymous subclass that has no name to look up
- `expr; rescue` inside `begin ... end` opens a rescue clause. Only a `rescue` directly following an expression is the modifier form
- `eval` and `parse` for runtime code execution and AST inspection
- `get_source` for runtime method introspection
- AST Inspection API: `Method#body`, `Block#statements`, node type/property access
- DSL Examples: test framework, HTML builder, query builder, configuration language
- Array Methods: `length`/`size`, `push`/`pop`, `shift`/`unshift`, `sort`, `reverse`, `map`, `select`/`filter`, `reduce`, `each`, `join`

### Phase 2: Bytecode VM
- Bytecode compiler (complete: expression, statement, control flow, function/method, class, closure, block compilation, optimization passes)
- Stack-based VM (complete: structure, call frames, execution loop, basic instructions, variables, control flow, function calls, closures, classes/objects, collections)
- Trait/interface system
- Advanced reflection

### Phase 3: Production Ready
- JIT compilation (LLVM)
- Full concurrency (threads, channels, atomics)
- Networking library (HTTP, WebSocket, TCP/UDP)
- Cryptography library
- Optional type system
- Documentation generator
- LSP support

### Phase 4: Advanced Features
- Macro system
- Algebraic data types (Option, Result)
- Functional programming features
- WebAssembly compilation
- Security and sandboxing
- Advanced tooling (profilers, static analysis)

## Contributing

METOREX is in active development. We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repository (including Ruby spec submodules)
git clone https://github.com/gdonald/metorex.git
cd metorex
git submodule update --init

# Build the project
cargo build

# Run tests
cargo test

# Run the REPL
cargo run

# Run a Metorex file
cargo run -- my_script.rb

# Discover and run test files in a directory
cargo run -- --test tests/

# Install code coverage tool (required for cargo tarpaulin)
cargo install cargo-tarpaulin

# Run code coverage
cargo tarpaulin --out Stdout

# Run Ruby spec suite (requires submodules)
scripts/run_ruby_spec.sh
```

## License

See [LICENSE](LICENSE) for details.

## Why METOREX?

**For DSL Creators**: Build domain-specific languages naturally with first-class AST access.

**For Scripters**: Ruby-like syntax with powerful built-in libraries.

**For Systems Programmers**: Rust-based VM with performance and safety guarantees.

**For Functional Enthusiasts**: Optional algebraic data types, immutable structures, and functional patterns.

**For Pragmatists**: One language that adapts to your needs - from quick scripts to production systems.

**METOREX: Where meta-programming meets production-ready performance.**
