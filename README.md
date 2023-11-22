# Metorex Programming Language

**METOREX** (**M**eta **O**bject **R**untime **E**xecution) is a modern, production-ready programming language that combines the expressiveness of Ruby and Python with the performance and safety of Rust. It features a unique **Code-as-Object** meta-programming system that exposes the AST as first-class runtime objects, enabling powerful DSL construction and runtime code manipulation.

⚠️ &nbsp;It's still very early in development.

🙂 &nbsp;[PRs](https://github.com/gdonald/metorex/pulls) and [new](https://github.com/gdonald/metorex/issues/new) [issues](https://github.com/gdonald/metorex/issues) are welcome.

###

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/gdonald/metorex/blob/main/LICENSE) [![CI](https://github.com/gdonald/metorex/workflows/CI/badge.svg)](https://github.com/gdonald/metorex/actions) [![codecov](https://codecov.io/gh/gdonald/metorex/graph/badge.svg?token=GQ4LA1VMRE)](https://codecov.io/gh/gdonald/metorex)

## Project Status

METOREX is currently in **active development** following a comprehensive 4-phase roadmap:

- **Phase 1 (MVP)**: AST Interpreter with meta-programming core - *In Progress*
- **Phase 2**: Bytecode VM with reflection maturity - *Planned*
- **Phase 3**: Optimization, concurrency, and production features - *Planned*
- **Phase 4**: Advanced features (macros, WebAssembly, functional programming) - *Planned*

See [ROADMAP.md](ROADMAP.md) for detailed implementation plans.

## Key Features

### Core Language Features
- **Exception Handling**: Full try/catch/finally with exception hierarchies and stack traces
- **Pattern Matching**: Powerful pattern matching with destructuring and guards
- **Built-in Testing**: Integrated test framework with assertions and test discovery
- **Traits/Interfaces**: Flexible polymorphism through trait system
- **Optional Type System**: Gradual typing with type inference for performance and safety
- **Advanced Collections**: Set, Deque, PriorityQueue, TreeMap, and immutable structures

### Meta-Programming (Core Innovation)
- **Code-as-Object**: AST nodes are first-class objects manipulable at runtime
- **Runtime Method Definition**: `define_method` for dynamic behavior
- **AST Reflection**: Inspect and modify code structure at runtime
- **Block Execution**: Blocks are objects with `.call()` method
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
| **Syntax Heritage**     | **Ruby** (block structure, optional parentheses), **Python** (readability, minimal syntax noise)                                  |
| **Primary Paradigms**   | **Full Object-Oriented**, **Imperative**, **Functional** (with ADTs and immutable structures)                                     |
| **Key Differentiator**  | **Code-as-Object (The Meta Core)**: The Abstract Syntax Tree (AST) is directly exposed as native, manipulable objects at runtime. |
| **Typing**              | **Dynamic by default**, with **optional static typing** and gradual type inference                                                |
| **Performance**         | **Bytecode VM** with **JIT compilation** for hot paths, built on Rust for safety                                                  |

## Syntax Overview

METOREX syntax prioritizes readability while minimizing keystrokes, combining elements from Ruby and Python.

### Basic Syntax

See [examples/basic_syntax.mx](examples/oop/basic_syntax.mx)

### Exception Handling

See [examples/exception_handling.mx](examples/advanced/exception_handling.mx)

### Pattern Matching

See [examples/pattern_matching.mx](examples/advanced/pattern_matching.mx)

### Traits (Interfaces)

See [examples/traits.mx](examples/advanced/traits.mx)

### Optional Type Annotations

See [examples/type_annotations.mx](examples/advanced/type_annotations.mx)

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

See [examples/implicit_block_capture.mx](examples/advanced/implicit_block_capture.mx)

### Dynamic Method Definition

See [examples/dynamic_method_definition.mx](examples/advanced/dynamic_method_definition.mx)

### Building DSLs

See [examples/dsl_example.mx](examples/advanced/dsl_example.mx)

## Architecture

### Multi-Phase Execution Model

1. **Phase 1 (MVP)**: Direct AST interpretation for rapid development
   - Lexer → Parser → AST → Interpreter
   - Full meta-programming capabilities
   - Exception handling, pattern matching, testing

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

See [examples/networking.mx](examples/advanced/networking.mx)

### Concurrency

See [examples/concurrency.mx](examples/advanced/concurrency.mx)

### Serialization

See [examples/serialization.mx](examples/advanced/serialization.mx)

## Roadmap Highlights

See [ROADMAP.md](ROADMAP.md) for complete details.

### Phase 1: MVP (In Progress)
- Lexer and Parser
- AST Interpreter
- Meta-programming core
- Exception handling
- Pattern matching
- Built-in testing framework

### Phase 2: Bytecode VM
- Bytecode compiler
- Stack-based VM
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
# Clone the repository
git clone https://github.com/yourusername/metorex.git
cd metorex

# Build the project
cargo build

# Run tests
cargo test

# Run the REPL
cargo run
```

## License

See [LICENSE](LICENSE) for details.

## Why METOREX?

**For DSL Creators**: Build domain-specific languages naturally with first-class AST access.

**For Scripters**: Ruby/Python-like syntax with powerful built-in libraries.

**For Systems Programmers**: Rust-based VM with performance and safety guarantees.

**For Functional Enthusiasts**: Optional algebraic data types, immutable structures, and functional patterns.

**For Pragmatists**: One language that adapts to your needs - from quick scripts to production systems.

**METOREX: Where meta-programming meets production-ready performance.**
