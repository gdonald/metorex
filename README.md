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
- An array literal gathers trailing `key: value` or `key => value` pairs into one Hash as its last element, so `[1, a: 2, b: 3]` is `[1, {a: 2, b: 3}]`
- `Kernel#p` writes each argument's `inspect` on its own line, honoring a user-defined `inspect`, and returns the argument, the argument list, or nil for none
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
