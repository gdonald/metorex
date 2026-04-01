# METOREX Development Roadmap

## Phase 1: Meta-Object Core & AST Interpreter (MVP)

Goal: Build a Minimum Viable Product (MVP) with functional Code-as-Object and dynamic dispatch model using direct Abstract Syntax Tree (AST) interpretation. Core implementation in Rust.

### 1. Project Setup and Infrastructure

- [x] 1.1. Initialize Rust Project
  - [x] 1.1.1. Run `cargo init metorex`
  - [x] 1.1.2. Create single crate with `src/` and `tests/` directories
  - [x] 1.1.3. Configure `Cargo.toml` with dependencies (clap, thiserror, etc.)
  - [x] 1.1.4. Set up `.gitignore` for Rust projects
  - [x] 1.1.5. Create `LICENSE` and `CONTRIBUTING.md`

- [x] 1.2. Error Handling Foundation
  - [x] 1.2.1. Define `MetorexError` enum in `src/error.rs`
  - [x] 1.2.2. Implement `SyntaxError` variant with line/column info
  - [x] 1.2.3. Implement `RuntimeError` variant with stack trace support
  - [x] 1.2.4. Implement `TypeError` variant for type mismatches
  - [x] 1.2.5. Add `Display` and `Error` trait implementations
  - [x] 1.2.6. Create error reporting utilities with source code snippets
  - [x] 1.2.7. Write unit tests for error formatting
  - [x] 1.2.8. Create example file: `examples/errors/basic_error.rb`

- [x] 1.3. Testing Infrastructure
  - [x] 1.3.1. Set up `tests/` directory structure
  - [x] 1.3.2. Create test harness for running `.rb` files
  - [x] 1.3.6. Configure code coverage reporting

### 2. Lexer (Tokenization)

- [x] 2.1. Token Type Definitions
  - [x] 2.1.1. Define `Token` enum in `src/lexer/token.rs`
  - [x] 2.1.2. Add keyword tokens (`Def`, `Class`, `If`, `Else`, `While`, `End`, `Do`)
  - [x] 2.1.3. Add literal tokens (`Int`, `Float`, `String`, `True`, `False`, `Nil`)
  - [x] 2.1.4. Add identifier token (`Ident`)
  - [x] 2.1.5. Add operator tokens (`Plus`, `Minus`, `Star`, `Slash`, `Percent`, `Equal`, `EqualEqual`, `BangEqual`, `Less`, `Greater`, `LessEqual`, `GreaterEqual`)
  - [x] 2.1.6. Add delimiter tokens (`LParen`, `RParen`, `LBrace`, `RBrace`, `LBracket`, `RBracket`, `Comma`, `Dot`, `Colon`, `Arrow`)
  - [x] 2.1.7. Add special tokens (`Newline`, `Semicolon`, `Comment`, `EOF`)
  - [x] 2.1.8. Add `TokenKind` and `Token` struct with position info (line, column, offset)
  - [x] 2.1.9. Implement `Display` for tokens

- [x] 2.2. Lexer Core Implementation
  - [x] 2.2.1. Create `Lexer` struct in `src/lexer/mod.rs`
  - [x] 2.2.2. Implement character stream with lookahead
  - [x] 2.2.3. Implement `advance()` and `peek()` methods
  - [x] 2.2.4. Track current position (line, column, offset)
  - [x] 2.2.5. Implement whitespace skipping (not significant)
  - [x] 2.2.6. Implement comment handling (# to end of line)
  - [x] 2.2.7. Create `next_token()` method skeleton

- [x] 2.3. Literal Tokenization
  - [x] 2.3.1. Implement integer literal parsing
  - [x] 2.3.2. Implement float literal parsing (with decimal point)
  - [x] 2.3.3. Implement string literal parsing (single and double quotes)
  - [x] 2.3.4. Handle escape sequences in strings (`\n`, `\t`, `\"`, `\\`)
  - [x] 2.3.5. Implement string interpolation syntax (`"Hello {name}"`)
  - [x] 2.3.6. Add error handling for unterminated strings
  - [x] 2.3.7. Write unit tests for all literal types
  - [x] 2.3.8. Create example file: `examples/lexer/literals.rb`

- [x] 2.4. Identifier and Keyword Tokenization
  - [x] 2.4.1. Implement identifier parsing (alphanumeric + underscore)
  - [x] 2.4.2. Create keyword lookup table
  - [x] 2.4.3. Distinguish keywords from identifiers
  - [x] 2.4.4. Handle instance variables (@ prefix)
  - [x] 2.4.5. Handle class variables (@@ prefix)
  - [x] 2.4.6. Write unit tests for identifiers and keywords
  - [x] 2.4.7. Create example file: `examples/lexer/identifiers.rb`

- [x] 2.5. Operator and Delimiter Tokenization
  - [x] 2.5.1. Implement single-character operators
  - [x] 2.5.2. Implement multi-character operators (`==`, `!=`, `<=`, `>=`, `->`)
  - [x] 2.5.3. Implement compound assignment operators (`+=`, `-=`, `*=`, `/=`)
  - [x] 2.5.4. Handle operator precedence table preparation
  - [x] 2.5.5. Write unit tests for all operators
  - [x] 2.5.6. Create example file: `examples/lexer/operators.rb`
  - [x] 2.5.7. Add `LogicalAnd` token (`&&`) and `LogicalOr` token (`||`)
  - [x] 2.5.8. Add `ColonColon` token (`::`) for scope resolution

- [x] 2.6. Lexer Integration and Testing
  - [x] 2.6.1. Implement `Iterator` trait for `Lexer`
  - [x] 2.6.2. Create helper methods for token stream manipulation
  - [x] 2.6.3. Write comprehensive integration tests
  - [x] 2.6.4. Test error recovery on invalid tokens
  - [x] 2.6.6. Create test file: `tests/lexer_tests.rs`
  - [x] 2.6.7. Fix identifier tokenization edge cases (predicate `?` and bang `!` suffixes)
  - [x] 2.6.8. Handle all variable prefix types correctly (`$global`, `@@class`, `@instance`)

### 3. Parser and AST Construction

- [x] 3.1. AST Node Type System
  - [x] 3.1.1. Define `Node` trait in `src/ast/node.rs`
  - [x] 3.1.2. Define `Statement` trait extending `Node`
  - [x] 3.1.3. Define `Expression` trait extending `Node`
  - [x] 3.1.4. Add `accept()` method for visitor pattern support
  - [x] 3.1.5. Add `location()` method returning source position
  - [x] 3.1.6. Add `children()` method for AST traversal
  - [x] 3.1.7. Implement `Debug` and `Display` for AST visualization

- [x] 3.2. Expression AST Nodes
  - [x] 3.2.1. Implement `LiteralExpr` (int, float, string, bool, nil)
  - [x] 3.2.2. Implement `IdentifierExpr` (variable reference)
  - [x] 3.2.3. Implement `BinaryExpr` (left, operator, right)
  - [x] 3.2.4. Implement `UnaryExpr` (operator, operand)
  - [x] 3.2.5. Implement `GroupingExpr` (parenthesized expressions)
  - [x] 3.2.6. Implement `ArrayLiteralExpr` (`[1, 2, 3]`)
  - [x] 3.2.7. Implement `DictLiteralExpr` (`{key: value}`)
  - [x] 3.2.8. Implement `IndexExpr` (array/dict access `arr[0]`)
  - [x] 3.2.9. Write unit tests for expression nodes
  - [x] 3.2.10. Create example file: `examples/parser/expressions.rb`
  - [x] 3.2.11. Implement `LogicalAndExpr` / `LogicalOrExpr` (`&&`, `||`) with short-circuit semantics
  - [x] 3.2.12. Implement `ScopeResolutionExpr` (`Math::PI`, `Module::Constant`)

- [x] 3.3. Method Call and Access Expressions
  - [x] 3.3.1. Implement `MethodCallExpr` (receiver, method_name, arguments)
  - [x] 3.3.2. Handle optional parentheses for zero-argument calls
  - [x] 3.3.3. Implement `PropertyAccessExpr` (object.property)
  - [x] 3.3.4. Implement `SelfExpr` (implicit self reference)
  - [x] 3.3.5. Handle method chaining (receiver.method1.method2)
  - [x] 3.3.6. Write unit tests for method calls
  - [x] 3.3.7. Create example file: `examples/parser/method_calls.rb`

- [x] 3.4. Statement AST Nodes
  - [x] 3.4.1. Implement `AssignmentStmt` (target, value)
  - [x] 3.4.2. Implement `ExpressionStmt` (wraps expression)
  - [x] 3.4.3. Implement `ReturnStmt` (optional return value)
  - [x] 3.4.4. Implement `BreakStmt` and `ContinueStmt`
  - [x] 3.4.5. Write unit tests for statement nodes
  - [x] 3.4.6. Create example file: `examples/parser/statements.rb`

- [x] 3.5. Block Statement (Core Meta-Object)
  - [x] 3.5.1. Define `BlockStmt` struct with `Vec<Box<dyn Statement>>`
  - [x] 3.5.2. Add scope capture mechanism
  - [x] 3.5.3. Add metadata (source range, parent scope reference)
  - [x] 3.5.4. Implement block parameter support (`do |param|`)
  - [x] 3.5.5. Add implicit return value (last expression)
  - [x] 3.5.6. Implement `Clone` for blocks
  - [x] 3.5.7. Write unit tests for block construction
  - [x] 3.5.8. Create example file: `examples/parser/blocks.rb`

- [x] 3.6. Control Flow AST Nodes
  - [x] 3.6.1. Implement `IfStmt` (condition, then_block, else_block)
  - [x] 3.6.2. Handle `else if` chain parsing
  - [x] 3.6.3. Implement `WhileStmt` (condition, body)
  - [x] 3.6.4. Implement `ForStmt` (variable, iterable, body)
  - [x] 3.6.5. Implement `MatchStmt` for pattern matching
  - [x] 3.6.6. Write unit tests for control flow
  - [x] 3.6.7. Create example file: `examples/parser/control_flow.rb`
  - [x] 3.6.8. Create test file: `tests/control_flow_tests.rs`

- [x] 3.7. Exception Handling AST Nodes
  - [x] 3.7.1. Implement `BeginStmt` (body, rescue_clauses, else_clause, ensure_block)
  - [x] 3.7.2. Implement `RescueClause` (exception_type, variable_name, handler_block)
  - [x] 3.7.3. Implement `RaiseStmt` (exception expression)
  - [x] 3.7.4. Parse `begin...rescue...else...ensure...end` syntax
  - [x] 3.7.5. Parse multiple rescue clauses for different exception types
  - [x] 3.7.6. Parse bare `raise` for re-raising exceptions
  - [x] 3.7.7. Write unit tests for exception handling AST
  - [x] 3.7.8. Create example file: `examples/parser/exceptions.rb`
  - [x] 3.7.9. Create test file: `tests/exception_parsing_tests.rs`

- [x] 3.8. Pattern Matching AST Nodes
  - [x] 3.8.1. Expand `MatchStmt` with arms and patterns
  - [x] 3.8.2. Implement `MatchArm` (pattern, guard, body)
  - [x] 3.8.3. Implement `LiteralPattern` (match literal values)
  - [x] 3.8.4. Implement `IdentifierPattern` (bind to variable)
  - [x] 3.8.5. Implement `ArrayPattern` (destructure arrays `[a, b, ...rest]`)
  - [x] 3.8.6. Implement `ObjectPattern` (destructure objects `{x, y}`)
  - [x] 3.8.7. Implement `WildcardPattern` (underscore `_`)
  - [x] 3.8.8. Implement `GuardClause` (when conditions)
  - [x] 3.8.9. Write unit tests for pattern matching
  - [x] 3.8.10. Create example file: `examples/parser/pattern_matching.rb`
  - [x] 3.8.11. Create test file: `tests/pattern_matching_tests.rs`
  - **Ruby compliance note**: Guards (`when x if x > 0`), variable binding (`when x`), array/hash destructuring, and wildcards (`_`) in `case...when` are NOT valid Ruby syntax. These features require `case...in` (Ruby 2.7+ pattern matching). The current Metorex `case...when` extended syntax is Metorex-specific.
  - [x] 3.8.12. Add `case...in` pattern matching AST node (Ruby 2.7+ compatible)
  - [x] 3.8.13. Add `if`/`unless` expression form (assign result of if: `x = if cond then a else b end`)
  - [x] 3.8.15. Consider range patterns as when values: `when 1..10 then ...`

- [x] 3.9. Function and Method Definition Nodes
  - [x] 3.9.1. Implement `FunctionDefStmt` (name, parameters, body)
  - [x] 3.9.2. Handle parameter list parsing
  - [x] 3.9.3. Handle default parameter values
  - [x] 3.9.4. Handle variable argument lists (`*args`)
  - [x] 3.9.5. Handle keyword arguments (`kwargs`)
  - [x] 3.9.6. Implement `MethodDefStmt` (extends FunctionDefStmt)
  - [x] 3.9.7. Write unit tests for function definitions
  - [x] 3.9.8. Create example file: `examples/parser/functions.rb`
  - [x] 3.9.9. Implement keyword argument syntax in definitions (`def method(name:, age: 10)`)
  - [x] 3.9.10. Implement keyword argument syntax in calls (`method(name: "Bob")`)
  - [x] 3.9.11. Allow operator method names (`def +(other)`, `def ==(other)`, `def [](key)`)

- [x] 3.10. Class Definition Nodes
  - [x] 3.10.1. Implement `ClassDefStmt` (name, superclass, body)
  - [x] 3.10.2. Parse class body (methods and instance variables)
  - [x] 3.10.3. Handle inheritance syntax (`class Dog < Animal`)
  - [x] 3.10.4. Handle `initialize` constructor parsing
  - [x] 3.10.5. Parse instance variable declarations
  - [x] 3.10.6. Write unit tests for class definitions
  - [x] 3.10.7. Create example file: `examples/parser/classes.rb`
  - [x] 3.10.8. Create test file: `tests/class_parsing_tests.rs`

- [x] 3.11. Parser Core Implementation
  - [x] 3.11.1. Create `Parser` struct with token stream
  - [x] 3.11.2. Implement `advance()`, `peek()`, `expect()` methods
  - [x] 3.11.3. Implement error recovery and synchronization
  - [x] 3.11.4. Add panic mode recovery
  - [x] 3.11.5. Implement operator precedence climbing
  - [x] 3.11.6. Create parse methods for each AST node type
  - [x] 3.11.7. Handle block delimiters (`do...end`, `if...end`)
  - [x] 3.11.8. Implement newline handling (statement terminators)

- [x] 3.12. Parser Testing and Examples
  - [x] 3.12.1. Write unit tests for each parse method
  - [x] 3.12.2. Create integration tests with full programs
  - [x] 3.12.3. Test error recovery scenarios
  - [x] 3.12.4. Create example: `examples/parser/complete_program.rb`
  - [x] 3.12.5. Create test file: `tests/parser_tests.rs`
  - [x] 3.12.6. Create test file: `tests/parser_error_recovery_tests.rs`

- [x] 3.13. Module and Mixin Support
  - [x] 3.13.1. Add `Module`, `Include`, `Extend` keyword tokens
  - [x] 3.13.2. Implement `ModuleDefStmt` AST node (`module Name ... end`)
  - [x] 3.13.3. Implement `IncludeStmt` and `ExtendStmt` AST nodes
  - [x] 3.13.4. VM: Create module objects in global registry
  - [x] 3.13.5. VM: `include` copies module methods into class instance method table
  - [x] 3.13.6. VM: `extend` adds module methods as class-level (singleton) methods
  - [x] 3.13.7. Write unit tests for module parsing and execution
  - [x] 3.13.8. Create example file: `examples/oop/modules.rb`
  - [x] 3.13.9. Create test file: `tests/module_tests.rs`

### 4. Runtime Object System

- [x] 4.1. Core Object Representation
  - [x] 4.1.1. Define `Object` enum in `src/object.rs`
  - [x] 4.1.2. Add `Nil` variant
  - [x] 4.1.3. Add `Bool(bool)` variant
  - [x] 4.1.4. Add `Int(i64)` variant
  - [x] 4.1.5. Add `Float(f64)` variant
  - [x] 4.1.6. Add `String(Rc<String>)` variant
  - [x] 4.1.7. Add `Array(Rc<RefCell<Vec<Object>>>)` variant
  - [x] 4.1.8. Add `Dict(Rc<RefCell<HashMap<String, Object>>>)` variant
  - [x] 4.1.9. Add `Instance(Rc<RefCell<Instance>>)` variant
  - [x] 4.1.10. Add `Class(Rc<Class>)` variant
  - [x] 4.1.11. Add `Method(Rc<Method>)` variant
  - [x] 4.1.12. Add `Block(Rc<BlockStatement>)` variant (critical for meta-programming)
  - [x] 4.1.13. Add `Exception(Rc<RefCell<Exception>>)` variant
  - [x] 4.1.14. Add `Set(Rc<RefCell<HashSet<Object>>>)` variant
  - [x] 4.1.15. Add `Result(Result<Object, Object>)` variant (for explicit error handling)

- [x] 4.2. Object Type System
  - [x] 4.2.1. Implement `type_of()` method for objects
  - [x] 4.2.2. Implement `is_truthy()` method
  - [x] 4.2.3. Implement `to_string()` method for all types
  - [x] 4.2.4. Implement `equals()` method with deep comparison
  - [x] 4.2.5. Implement `hash()` method for hashable types
  - [x] 4.2.6. Write unit tests for type checking
  - [x] 4.2.7. Create example file: `examples/runtime/types.rb`
  - [x] 4.2.8. Create test file: `tests/type_system_tests.rs`

- [x] 4.3. Class Structure
  - [x] 4.3.1. Define `Class` struct in `src/class.rs`
  - [x] 4.3.2. Add name field
  - [x] 4.3.3. Add superclass reference (Option<Rc<Class>>)
  - [x] 4.3.4. Add method table (HashMap<String, Rc<Method>>)
  - [x] 4.3.5. Add instance variable list
  - [x] 4.3.6. Implement method lookup with inheritance chain
  - [x] 4.3.7. Implement `define_method()` for runtime method addition
  - [x] 4.3.8. Write unit tests for class structure
  - [x] 4.3.9. Create test file: `tests/class_system_tests.rs`

- [x] 4.4. Instance Structure
  - [x] 4.4.1. Define `Instance` struct in `src/instance.rs`
  - [x] 4.4.2. Add class reference
  - [x] 4.4.3. Add instance variable storage (HashMap<String, Object>)
  - [x] 4.4.4. Implement instance variable get/set methods
  - [x] 4.4.5. Implement method dispatch to class
  - [x] 4.4.6. Write unit tests for instances
  - [x] 4.4.7. Create example file: `examples/runtime/instances.rb`

- [x] 4.5. Method Structure
  - [x] 4.5.1. Define `Method` struct in `src/object.rs`
  - [x] 4.5.2. Add name field
  - [x] 4.5.3. Add parameter list
  - [x] 4.5.4. Add body (AST Statement references)
  - [x] 4.5.5. Add closure scope capture (handled by BlockStatement)
  - [x] 4.5.6. Implement `Callable` trait
  - [x] 4.5.7. Write unit tests for methods

- [x] 4.6. Exception Structure
  - [x] 4.6.1. Define `Exception` struct in `src/object.rs`
  - [x] 4.6.2. Add exception message field
  - [x] 4.6.3. Add exception type/class field
  - [x] 4.6.4. Add stack trace capture
  - [x] 4.6.5. Add source location (file, line, column)
  - [x] 4.6.6. Add cause chain (wrapped exceptions)
  - [x] 4.6.7. Implement exception hierarchy (base exception classes)
  - [x] 4.6.8. Write unit tests for exceptions
  - [x] 4.6.10. Create test file: `tests/exception_objects_tests.rs`

- [x] 4.7. Built-in Base Classes
  - [x] 4.7.1. Implement `Object` base class
  - [x] 4.7.2. Add `Object#to_s` method
  - [x] 4.7.3. Add `Object#class` method
  - [x] 4.7.4. Add `Object#respond_to?` method
  - [x] 4.7.5. Add `Object#define_method` (deferred to interpreter)
  - [x] 4.7.6. Add `Object#get_source` (deferred to interpreter)
  - [x] 4.7.7. Implement `String` class with methods
  - [x] 4.7.8. Implement `Integer` class with methods
  - [x] 4.7.9. Implement `Float` class with methods
  - [x] 4.7.10. Implement `Array` class with methods
  - [x] 4.7.11. Implement `Hash` class with methods
  - [x] 4.7.12. Implement `Set` class with methods
  - [x] 4.7.13. Implement base `Exception` class
  - [x] 4.7.14. Implement `StandardError`, `RuntimeError`, `TypeError`, `ValueError` exception classes
  - [x] 4.7.15. Write comprehensive tests for built-in classes
  - [x] 4.7.16. Create example file: `examples/runtime/builtin_classes.rb`
  - [x] 4.7.17. Create test file: `tests/builtin_classes_tests.rs`

### 5. Scope and Environment Management

- [x] 5.1. Scope Implementation
  - [x] 5.1.1. Define `Scope` struct in `src/scope.rs`
  - [x] 5.1.2. Add variable storage (HashMap<String, Object>)
  - [x] 5.1.3. Add parent scope reference (Option<Rc<RefCell<Scope>>>)
  - [x] 5.1.4. Implement `define()` method for new variables
  - [x] 5.1.5. Implement `get()` method with scope chain lookup
  - [x] 5.1.6. Implement `set()` method with scope chain traversal
  - [x] 5.1.7. Implement `get_at()` and `set_at()` for closure resolution
  - [x] 5.1.8. Write unit tests for scope operations
  - [x] 5.1.9. Create test file: `tests/scope_tests.rs`

- [x] 5.2. Environment Stack
  - [x] 5.2.1. Create `Environment` struct managing scope stack
  - [x] 5.2.2. Implement `push_scope()` method
  - [x] 5.2.3. Implement `pop_scope()` method
  - [x] 5.2.4. Implement `current_scope()` accessor
  - [x] 5.2.5. Add global scope initialization
  - [x] 5.2.6. Add scope depth tracking
  - [x] 5.2.7. Write unit tests for environment
  - [x] 5.2.8. Create test file: `tests/environment_tests.rs`

- [x] 5.3. Variable Resolution
  - [x] 5.3.1. Implement static variable resolution pass
  - [x] 5.3.2. Track variable declaration depths
  - [x] 5.3.3. Detect undefined variable access
  - [x] 5.3.4. Detect variable shadowing
  - [x] 5.3.5. Write unit tests for resolution
  - [x] 5.3.6. Create example file: `examples/runtime/variable_scope.rb`
  - [x] 5.3.7. Create test file: `tests/variable_resolution_tests.rs`

### 6. Virtual Machine (AST Interpreter)

- [x] 6.1. VM Core Structure
  - [x] 6.1.1. Define `VirtualMachine` struct in `src/vm.rs`
  - [x] 6.1.2. Add environment (scope stack) field
  - [x] 6.1.3. Add call stack for debugging
  - [x] 6.1.4. Add global object registry
  - [x] 6.1.5. Add heap allocator reference
  - [x] 6.1.6. Initialize built-in classes and objects
  - [x] 6.1.7. Implement VM constructor
  - [x] 6.1.8. Write unit tests for VM initialization

- [x] 6.2. Statement Execution
  - [x] 6.2.1. Implement `execute_statement()` dispatcher
  - [x] 6.2.2. Implement expression statement execution
  - [x] 6.2.3. Implement assignment statement execution
  - [x] 6.2.4. Implement return statement execution
  - [x] 6.2.5. Implement break/continue handling
  - [x] 6.2.6. Implement control flow statements (covered by 6.6)
  - [x] 6.2.7. Write unit tests for statement execution
  - [x] 6.2.8. Create test file: `tests/vm_statement_tests.rs`

- [x] 6.3. Expression Evaluation
  - [x] 6.3.1. Implement `evaluate_expression()` dispatcher
  - [x] 6.3.2. Implement literal evaluation
  - [x] 6.3.3. Implement identifier lookup
  - [x] 6.3.4. Implement binary operation evaluation
  - [x] 6.3.5. Implement unary operation evaluation
  - [x] 6.3.6. Implement array/dict literal evaluation
  - [x] 6.3.7. Implement indexing operations
  - [x] 6.3.8. Write unit tests for expression evaluation
  - [x] 6.3.9. Create test file: `tests/vm_expression_tests.rs`
  - [x] 6.3.10. Implement `&&` / `||` short-circuit evaluation
  - [x] 6.3.11. Implement scope resolution evaluation (`Math::PI`, `ClassName::CONST`)

- [x] 6.4. Method Call Implementation
  - [x] 6.4.1. Implement method lookup on receiver object
  - [x] 6.4.2. Handle method not found errors
  - [x] 6.4.3. Bind `self` to receiver
  - [x] 6.4.4. Evaluate and bind arguments
  - [x] 6.4.5. Create new scope for method execution
  - [x] 6.4.6. Execute method body
  - [x] 6.4.7. Handle return values
  - [x] 6.4.8. Implement native method calls
  - [x] 6.4.9. Write unit tests for method dispatch
  - [x] 6.4.10. Create example file: `examples/runtime/method_dispatch.rb`
  - [x] 6.4.11. Create test file: `tests/method_dispatch_tests.rs`
  - [x] 6.4.12. Implement operator overloading: before evaluating built-in binary ops, check if receiver class defines the operator as a method (`+`, `-`, `*`, `/`, `==`, `<`, `>`, `[]`)

- [x] 6.5. Block Execution (Critical Meta-Programming Feature)
  - [x] 6.5.1. Define `Callable` trait in `src/callable.rs`
  - [x] 6.5.2. Implement `Callable` for `BlockStatement`
  - [x] 6.5.3. Implement `BlockStatement::call()` method
  - [x] 6.5.4. Capture closure scope on block creation
  - [x] 6.5.5. Bind block parameters on call
  - [x] 6.5.6. Execute block body in captured scope
  - [x] 6.5.7. Return last expression value
  - [x] 6.5.8. Write unit tests for block execution
  - [x] 6.5.9. Create example file: `examples/metaprogramming/block_call.rb`
  - [x] 6.5.10. Create test file: `tests/block_execution_tests.rs`

- [x] 6.6. Control Flow Execution
  - [x] 6.6.1. Implement if/else execution
  - [x] 6.6.2. Implement while loop execution
  - [x] 6.6.3. Implement for loop execution
  - [x] 6.6.4. Handle break statements with loop unwinding
  - [x] 6.6.5. Handle continue statements
  - [x] 6.6.6. Write unit tests for control flow
  - [x] 6.6.7. Create example file: `examples/runtime/loops.rb`
  - [x] 6.6.8. Create test file: `tests/control_flow_execution_tests.rs`

- [x] 6.7. Class and Instance Creation
  - [x] 6.7.1. Implement class definition execution
  - [x] 6.7.2. Register class in global registry
  - [x] 6.7.3. Implement instance creation (`Class.new`)
  - [x] 6.7.4. Call `initialize` constructor
  - [x] 6.7.5. Initialize instance variables
  - [x] 6.7.6. Write unit tests for class instantiation
  - [x] 6.7.7. Create example file: `examples/runtime/class_instantiation.rb`
  - [x] 6.7.8. Create test file: `tests/class_instantiation_tests.rs`

- [x] 6.8. Inheritance Implementation
  - [x] 6.8.1. Implement superclass method lookup
  - [x] 6.8.2. Implement method overriding
  - [x] 6.8.3. Implement `super` keyword (requires lexer, parser, AST, VM changes)
  - [x] 6.8.4. Handle constructor chaining (requires `super` keyword)
  - [x] 6.8.5. Write unit tests for inheritance
  - [x] 6.8.6. Create example file: `examples/runtime/inheritance.rb`
  - [x] 6.8.7. Create test file: `tests/inheritance_tests.rs`

- [x] 6.9. Exception Handling Execution
  - [x] 6.9.1. Implement try block execution (begin...end blocks)
  - [x] 6.9.2. Implement catch clause matching and execution (rescue clauses)
  - [x] 6.9.3. Implement finally block execution (ensure blocks always run)
  - [x] 6.9.4. Implement raise statement execution
  - [x] 6.9.5. Implement exception unwinding through call stack (via ControlFlow::Exception)
  - [x] 6.9.6. Match exception types in catch clauses (rescue ExceptionType)
  - [x] 6.9.7. Bind exception to variable in catch clause (rescue => e)
  - [x] 6.9.8. Handle uncaught exceptions (converted to MetorexError at top level)
  - [x] 6.9.9. Implement bare raise (re-raises current exception from $!)
  - [x] 6.9.10. Write unit tests for exception handling (tests/exception_execution_tests.rs created, requires parser implementation to run)
  - [x] 6.9.11. Create example file: `examples/runtime/exception_handling.rb` (requires parser implementation)
  - [x] 6.9.12. Create test file: `tests/exception_handling_tests.rs` (requires parser to activate)

- [x] 6.10. Pattern Matching Execution
  - [x] 6.10.1. Implement match statement execution
  - [x] 6.10.2. Implement literal pattern matching
  - [x] 6.10.3. Implement identifier pattern binding
  - [x] 6.10.4. Implement array destructuring patterns
  - [x] 6.10.5. Implement object destructuring patterns
  - [x] 6.10.6. Implement wildcard pattern matching
  - [x] 6.10.7. Implement guard clause evaluation
  - [x] 6.10.8. Ensure exhaustiveness or provide default case
  - [x] 6.10.9. Write unit tests for pattern matching
  - [x] 6.10.10. Create example file: `examples/runtime/pattern_matching.rb`
  - [x] 6.10.11. Create test file: `tests/pattern_matching_execution_tests.rs`
  - [x] 6.10.12. Test `case` as expression in assignment: `x = case val when 1 then "one" end`
  - [x] 6.10.13. Test `case` expression in method call arguments and string interpolation
  - [x] 6.10.14. Test nested case expressions with separate scopes
  - [x] 6.10.15. Test `case` returns nil when no match and no else clause
  - [x] 6.10.16. Test multiple literal patterns per when clause: `when 1, 2, 3 then "small"`
  - [x] 6.10.17. Test `case` expression in arithmetic context: `1 + (case y when 1 then 2 else 3 end)`

- [x] 6.11. Error Reporting and Stack Traces
  - [x] 6.11.1. Implement call stack tracking
  - [x] 6.11.2. Capture source location on each call
  - [x] 6.11.3. Generate stack traces on errors
  - [x] 6.11.4. Add line number information to errors
  - [x] 6.11.5. Implement pretty-printing of stack traces
  - [x] 6.11.6. Write unit tests for error reporting
  - [x] 6.11.7. Create example file: `examples/runtime/error_reporting.rb`
  - [x] 6.11.8. Create test file: `tests/error_reporting_tests.rs`

### 7. Meta-Programming Features (MVP)

- [x] 7.1. Block as First-Class Object
  - [x] 7.1.1. Ensure blocks can be assigned to variables
  - [x] 7.1.2. Ensure blocks can be passed as arguments
  - [x] 7.1.3. Ensure blocks can be returned from methods
  - [x] 7.1.4. Implement block parameter syntax (`do |x, y|`)
  - [x] 7.1.5. Write unit tests for block objects
  - [x] 7.1.6. Create example file: `examples/metaprogramming/blocks_as_objects.rb`
  - [x] 7.1.7. Create test file: `tests/block_as_object_tests.rs`

- [x] 7.2. Implicit Block Capture
  - [x] 7.2.1. AST Changes: Add `trailing_block` field to `Expression::MethodCall`
  - [x] 7.2.2. AST Changes: Add `trailing_block` field to `Expression::Call` for function calls
  - [x] 7.2.3. Parser: Add `do...end` block literal parsing
  - [x] 7.2.4. Parser: Add `{...}` block literal parsing (alternative syntax)
  - [x] 7.2.5. Parser: Modify method call parser to capture trailing blocks after arguments
  - [x] 7.2.6. Parser: Handle precedence and associativity for block literals
  - [x] 7.2.7. Parser: Add tests for parsing method calls with trailing blocks
  - [x] 7.2.8. VM: Modify method execution to accept optional trailing block
  - [x] 7.2.9. VM: Pass trailing block as implicit last argument OR make available via `block` variable
  - [x] 7.2.10. VM: Implement `block_given?` or similar to check if block was passed
  - [x] 7.2.11. VM: Handle methods with explicit block parameter vs implicit block
  - [x] 7.2.12. Write unit tests for runtime behavior with implicit blocks
  - [x] 7.2.13. Create example file: `examples/metaprogramming/implicit_blocks.rb`
  - [x] 7.2.14. Create test file: `tests/implicit_block_tests.rs`
  - [x] 7.2.15. Document the implicit block capture feature

- [x] 7.3. `define_method` Implementation
  - [x] 7.3.1. Add `define_method(name, block)` to Object class
  - [x] 7.3.2. Extract BlockStatement from block object
  - [x] 7.3.3. Create Method object from block
  - [x] 7.3.4. Add method to class method table
  - [x] 7.3.5. Ensure method is callable on instances
  - [x] 7.3.6. Write unit tests for define_method
  - [x] 7.3.7. Create example file: `examples/metaprogramming/define_method.rb`
  - [x] 7.3.8. Create test file: `tests/blocks/define_method_tests.rs`

- [x] 7.4. `get_source` Implementation
  - [x] 7.4.1. Add `get_source(method_name)` to Object class
  - [x] 7.4.2. Lookup method in class hierarchy
  - [x] 7.4.3. Return Method object for method body
  - [x] 7.4.4. Handle native methods (return nil or method stub)
  - [x] 7.4.5. Write unit tests for get_source
  - [x] 7.4.6. Create example file: `tests/_examples/metaprogramming/get_source.rb`
  - [x] 7.4.7. Create test file: `tests/blocks/get_source_tests.rs`

- [x] 7.5. AST Inspection API
  - [x] 7.5.1. Expose AST node types as Metorex classes
  - [x] 7.5.2. Add methods to inspect node properties
  - [x] 7.5.3. Allow traversal of AST tree
  - [x] 7.5.4. Implement `BlockStatement#statements` accessor
  - [x] 7.5.5. Implement node type checking methods
  - [x] 7.5.6. Write unit tests for AST inspection
  - [x] 7.5.7. Create example file: `examples/metaprogramming/ast_inspection.rb`
  - [x] 7.5.8. Create test file: `tests/ast_inspection_tests.rs`

- [x] 7.6. DSL Examples and Tests
  - [x] 7.6.1. Create DSL example: simple test framework
  - [x] 7.6.2. Create DSL example: HTML builder
  - [x] 7.6.3. Create DSL example: query builder
  - [x] 7.6.4. Create DSL example: configuration language
  - [x] 7.6.5. Write tests for each DSL
  - [x] 7.6.6. Document DSL patterns
  - [x] 7.6.7. Create example file: `examples/dsl/test_framework.rb`
  - [x] 7.6.8. Create example file: `examples/dsl/html_builder.rb`

### 8. Standard Library (Core)

- [x] 8.1. String Methods
  - [x] 8.1.1. Implement `length`/`size`
  - [x] 8.1.2. Implement `upcase`/`downcase`
  - [x] 8.1.3. Implement `split`
  - [x] 8.1.4. Implement `join`
  - [x] 8.1.5. Implement `substring`/`slice`
  - [x] 8.1.6. Implement `contains?`
  - [x] 8.1.7. Implement `starts_with?`/`ends_with?`
  - [x] 8.1.8. Implement string interpolation
  - [x] 8.1.9. Write unit tests for string methods
  - [x] 8.1.10. Create example file: `examples/stdlib/strings.rb`
  - [x] 8.1.11. Create test file: `tests/string_methods_tests.rs`

- [x] 8.2. Array Methods
  - [x] 8.2.1. Implement `length`/`size`
  - [x] 8.2.2. Implement `push`/`append`
  - [x] 8.2.3. Implement `pop`
  - [x] 8.2.4. Implement `shift`/`unshift`
  - [x] 8.2.5. Implement `map`
  - [x] 8.2.6. Implement `filter`/`select`
  - [x] 8.2.7. Implement `reduce`/`fold`
  - [x] 8.2.8. Implement `each` (iterator with block)
  - [x] 8.2.9. Implement `sort`
  - [x] 8.2.10. Implement `reverse`
  - [x] 8.2.11. Write unit tests for array methods
  - [x] 8.2.12. Create example file: `examples/stdlib/arrays.rb`
  - [x] 8.2.13. Create test file: `tests/array_methods_tests.rs`

- [x] 8.3. Hash/Dict Methods
  - [x] 8.3.1. Implement `keys`
  - [x] 8.3.2. Implement `values`
  - [x] 8.3.3. Implement `has_key?`
  - [x] 8.3.4. Implement `get`/`fetch`
  - [x] 8.3.5. Implement `merge`
  - [x] 8.3.6. Implement `each` (iterator)
  - [x] 8.3.7. Write unit tests for hash methods
  - [x] 8.3.8. Create example file: `examples/stdlib/hashes.rb`
  - [x] 8.3.9. Create test file: `tests/hash_methods_tests.rs`

- [x] 8.4. Numeric Methods
  - [x] 8.4.1. Implement basic arithmetic operators
  - [x] 8.4.2. Implement comparison operators
  - [x] 8.4.3. Implement `abs`
  - [x] 8.4.4. Implement `ceil`/`floor`/`round` (for Float)
  - [x] 8.4.5. Implement `to_i`/`to_f`/`to_s`
  - [x] 8.4.6. Implement `times` iterator (for Int)
  - [x] 8.4.7. Write unit tests for numeric methods
  - [x] 8.4.8. Create example file: `examples/stdlib/numbers.rb`
  - [x] 8.4.9. Create test file: `tests/numeric_methods_tests.rs`

- [x] 8.4b. Type Introspection Built-ins
  - [x] 8.4b.1. Implement `type()` / `.class` function returning class name string
  - [x] 8.4b.2. Implement `is_a?` / `kind_of?` type checking
  - [x] 8.4b.3. Add class hierarchy inspection (`superclass`, `ancestors`)
  - [x] 8.4b.4. Implement instance variable inspection (`instance_variables`, `instance_variable_get`)
  - [x] 8.4b.5. Add instance copying/cloning (`dup`, `clone`)
  - [x] 8.4b.6. Write unit tests for type introspection
  - [x] 8.4b.7. Use `tests/_examples/builtins/type_introspection.rb` in an actual test in `examples_runner.rs`

- [x] 8.5. IO and Print Functions
  - [x] 8.5.1. Implement `print` function
  - [x] 8.5.2. Implement `puts`/`println` function
  - [x] 8.5.3. Implement `gets`/`readline` for input
  - [x] 8.5.4. Implement file reading (`File.read`)
  - [x] 8.5.5. Implement file writing (`File.write`)
  - [x] 8.5.6. Write unit tests for IO
  - [x] 8.5.7. Create example file: `examples/stdlib/io.rb`
  - [x] 8.5.8. Create test file: `tests/io_tests.rs`

- [x] 8.6. Set Methods
  - [x] 8.6.1. Implement `add`/`insert` for sets
  - [x] 8.6.2. Implement `remove`/`delete` for sets
  - [x] 8.6.3. Implement `contains?`/`has?` for sets
  - [x] 8.6.4. Implement set operations (`union`, `intersection`, `difference`)
  - [x] 8.6.5. Implement `size`/`length` for sets
  - [x] 8.6.6. Implement `each` iterator for sets
  - [x] 8.6.7. Write unit tests for set methods
  - [x] 8.6.8. Create example file: `examples/stdlib/sets.rb`
  - [x] 8.6.9. Create test file: `tests/set_methods_tests.rs`

- [x] 8.7. Built-in Testing Framework
  - [x] 8.7.1. Design test framework API (`describe`, `it`, `expect`)
  - [x] 8.7.2. Implement `Test` class for test definitions
  - [x] 8.7.3. Implement `TestRunner` for executing tests
  - [x] 8.7.4. Implement assertion methods (`assert`, `assert_equal`, `assert_raises`)
  - [x] 8.7.5. Implement test discovery (finding test files)
  - [x] 8.7.6. Implement test reporting (pass/fail counts, timing)
  - [x] 8.7.7. Implement `before` and `after` hooks
  - [x] 8.7.8. Implement test filtering (run specific tests)
  - [x] 8.7.9. Add colored output for test results
  - [x] 8.7.10. Write tests for the testing framework (meta!)
  - [x] 8.7.11. Create example file: `examples/stdlib/testing_framework.rb`
  - [x] 8.7.12. Create test file: `tests/testing_framework_tests.rs`

### 9. CLI and REPL

- [x] 9.1. Command Line Interface
  - [x] 9.1.1. Create `cli` crate
  - [x] 9.1.2. Implement argument parsing with clap
  - [x] 9.1.3. Add `metorex <file.rb>` command
  - [x] 9.1.4. Add `--version` flag
  - [x] 9.1.5. Add `--help` flag
  - [x] 9.1.6. Add `--ast` flag to dump AST
  - [x] 9.1.7. Add `--debug` flag for verbose output
  - [x] 9.1.8. Write integration tests for CLI
  - [x] 9.1.9. Create test file: `tests/cli_tests.rs`

- [x] 9.2. REPL Implementation
  - [x] 9.2.1. Implement basic read-eval-print loop
  - [x] 9.2.2. Add line editing with rustyline
  - [x] 9.2.3. Add history support
  - [x] 9.2.4. Add multi-line input support
  - [x] 9.2.5. Add `.exit` command
  - [x] 9.2.6. Add `.clear` command
  - [x] 9.2.7. Add `.help` command
  - [x] 9.2.8. Write tests for REPL
  - [x] 9.2.9. Create test file: `tests/repl/repl_test.rs`

- [x] 9.3. `require_relative` File Loading (Phases 1–4 complete)
  - [x] 9.3.1. VM: `current_file` tracking and `loaded_files` deduplication registry
  - [x] 9.3.2. `src/file_loader.rs`: `load_file_source()`, `resolve_relative_path()`, `parse_file()`
  - [x] 9.3.3. VM: `execute_file()` with deduplication and path restore
  - [x] 9.3.4. Native function `require_relative` registered in global scope
  - [x] 9.3.5. Test file extension auto-detection: `.rb`, `.rb`, and no extension
  - [x] 9.3.6. Test scope/variable sharing: variables, functions, and classes defined in required files are accessible in requiring file
  - [x] 9.3.7. Test nested requires (A → B → C execute in correct order)
  - [x] 9.3.8. Test circular requires are handled gracefully (no infinite loop)
  - [x] 9.3.9. Test diamond dependency (D loads only once when required by both B and C)
  - [x] 9.3.10. Review error messages for clarity; use absolute paths in errors
  - [x] 9.3.11. Add suggestions in error messages (e.g., "did you mean file.rb?")

### 10. End-to-End Testing and Examples

- [x] 10.1. Comprehensive Language Examples
  - [x] 10.1.1. Create example: FizzBuzz
  - [x] 10.1.2. Create example: Fibonacci sequence
  - [x] 10.1.3. Create example: Factorial
  - [x] 10.1.4. Create example: Calculator
  - [x] 10.1.5. Create example: Todo list application
  - [x] 10.1.7. Create example: Data processing pipeline
  - [x] 10.1.8. Create example: Object-oriented design patterns
  - [x] 10.1.9. All examples in `tests/_examples/programs/`

- [x] 10.2. Meta-Programming Showcase
  - [x] 10.2.1. Create example: Method delegation
  - [x] 10.2.2. Create example: Aspect-oriented programming
  - [x] 10.2.3. Create example: Lazy evaluation
  - [x] 10.2.4. Create example: Memoization decorator
  - [x] 10.2.5. Create example: Custom iterators
  - [x] 10.2.6. Create example: Method chaining DSL
  - [x] 10.2.7. All examples in `tests/_examples/metaprogramming/advanced/`

- [x] 10.3. Integration Test Suite
  - [x] 10.3.1. Test lexer + parser integration
  - [x] 10.3.2. Test parser + VM integration
  - [x] 10.3.3. Test full compilation pipeline
  - [x] 10.3.4. Test all language features together
  - [x] 10.3.5. Create test file: `tests/integration_tests/` module
  - [x] 10.3.6. Code coverage measured (current: 94.67%, 6977/7370 lines — see Future Enhancements for 100% goal)

- [x] 10.4. Examples Runner Coverage (`tests/integration/examples_runner.rs`)

  Tracks which `tests/_examples/` scripts are wired into the integration test runner with verified output. **Current status: 76/90 tests active.**

  - [x] 10.4.1. Core String Features
    - [x] Use `tests/_examples/basics/greeting_line.rb`
    - [x] Use `tests/_examples/basics/string_methods.rb`
    - [x] Use `tests/_examples/type_annotations/collection_types.rb`
    - [x] Use `tests/_examples/basics/simple_range.rb`
    - [x] Use `tests/_examples/basics/each_block.rb`

  - [x] 10.4.2. Data Structures
    - [x] Use `tests/_examples/data_structures/simple_dict.rb`
    - [x] Use `tests/_examples/data_structures/dict_access.rb`
    - [x] Use `tests/_examples/data_structures/hash_methods.rb`

  - [x] 10.4.3. Algorithms
    - [x] Use `tests/_examples/algorithms/factorial_iterative.rb`
    - [x] Use `tests/_examples/algorithms/average_temperature.rb`
    - [x] Use `tests/_examples/algorithms/primes_under_fifty.rb`
    - [x] Use `tests/_examples/algorithms/filter_even_numbers.rb`
    - [x] Use `tests/_examples/algorithms/character_counter.rb`
    - [x] Use `tests/_examples/algorithms/zip_merger.rb`
    - [x] Use `tests/_examples/algorithms/matrix_transpose.rb`
    - [x] Use `tests/_examples/algorithms/matrix_transpose_comprehensive.rb`
    - [x] Use `tests/_examples/algorithms/matrix_nested_ops.rb`

  - [x] 10.4.4. Functions and Closures
    - [x] Use `tests/_examples/functions/closures_nested.rb`
    - [x] Use `tests/_examples/functions/locals_scope.rb`
    - [x] Use `tests/_examples/functions/nonlocal_counter.rb`
    - [x] Use `tests/_examples/functions/test_lambdas.rb`
    - [x] Use `tests/_examples/metaprogramming/blocks_as_objects.rb`

  - [x] 10.4.5. OOP
    - [x] Use `tests/_examples/oop/super_basic.rb`
    - [x] Use `tests/_examples/oop/super_chain_basic.rb`
    - [x] Use `tests/_examples/oop/test_super_simple.rb`
    - [x] Use `tests/_examples/oop/attr_reader.rb`
    - [x] Use `tests/_examples/oop/attr_writer.rb`
    - [x] Use `tests/_examples/oop/attr_accessor.rb`
    - [x] Use `tests/_examples/oop/test_str.rb`
    - [x] Use `tests/_examples/oop/test_repr.rb`
    - [x] Use `tests/_examples/oop/special_methods.rb`
    - [x] Use `tests/_examples/oop/test_iter.rb`
    - [x] Use `tests/_examples/oop/test_method_missing.rb`

  - [x] 10.4.6. Control Flow
    - [x] Use `tests/_examples/basics/for_loop_array.rb`
    - [x] Use `tests/_examples/basics/for_loop_range.rb`
    - [x] Use `tests/_examples/basics/for_loop_break.rb`
    - [x] Use `tests/_examples/basics/for_loop_continue.rb`
    - [x] Use `tests/_examples/basics/elsif_basic.rb`
    - [x] Use `tests/_examples/basics/elsif_without_else.rb`
    - [x] Use `tests/_examples/basics/elsif_no_parens.rb`
    - [x] Use `tests/_examples/control_flow/test_pattern_matching.rb`
    - [x] Use `tests/_examples/control_flow/case_guard.rb`
    - [x] Use `tests/_examples/control_flow/case_array_destructure.rb`
    - [x] Use `tests/_examples/control_flow/case_object_destructure.rb`
    - [x] Use `tests/_examples/control_flow/case_variable_binding.rb`
    - [x] Use `tests/_examples/control_flow/case_type_basic.rb`
    - [x] Use `tests/_examples/control_flow/case_type_custom_class.rb`
    - [x] Use `tests/_examples/control_flow/case_type_mixed.rb`
    - [x] Use `tests/_examples/control_flow/case_expr_inline.rb`
    - [x] Use `tests/_examples/control_flow/case_expr_block.rb`
    - [x] Use `tests/_examples/control_flow/case_expr_mixed.rb`
    - [x] Use `tests/_examples/control_flow/case_expression_basic.rb`
    - [x] Use `tests/_examples/control_flow/case_expression_patterns.rb`
    - [x] Use `tests/_examples/control_flow/case_expression_nested.rb`
    - [x] Use `tests/_examples/control_flow/case_multi_value.rb`

  - [x] 10.4.7. Exceptions
    - [x] Use `tests/_examples/errors/simple_rescue.rb`
    - [x] Use `tests/_examples/advanced/exception_handling.rb`
    - [x] Use `tests/_examples/errors/exception_hierarchy.rb`
    - [x] Use `tests/_examples/errors/custom_exceptions.rb`
    - [x] Use `tests/_examples/errors/exception_chaining.rb`
    - [x] Use `tests/_examples/errors/stack_trace_basic.rb`
    - [x] Use `tests/_examples/errors/stack_trace_deep.rb`
    - [x] Use `tests/_examples/errors/error_location.rb`
    - [x] Use `tests/_examples/errors/backtrace_method.rb`

  - [x] 10.4.8. Introspection
    - [x] Use `tests/_examples/introspection/function_name.rb`
    - [x] Use `tests/_examples/introspection/function_module.rb`
    - [x] Use `tests/_examples/introspection/code_object.rb`
    - [x] Use `tests/_examples/introspection/closure_namespace.rb`
    - [x] Use `tests/_examples/introspection/basic_attributes.rb`
    - [x] Use `tests/_examples/introspection/annotations.rb`
    - [x] Use `tests/_examples/introspection/default_parameters.rb`

  - [x] 10.4.9. File Loading
    - [x] Use `tests/_examples/file_tracking/simple.rb`
    - [x] Use `tests/_examples/require/basic.rb`
    - [x] Use `tests/_examples/require/deduplication.rb`
    - [x] Use `tests/_examples/require/nested.rb`
    - [x] Use `tests/_examples/require/return_values.rb`

  - [x] 10.4.10. Advanced Features
    - [x] Use `tests/_examples/advanced/traits.rb`

  - [x] 10.4.11. Builtins
    - [x] Use `tests/_examples/builtins/type_introspection.rb`

---

## Phase 2: Bytecode VM and Reflection Maturity

Goal: Improve performance by migrating to a Bytecode Virtual Machine and implement full Meta-Object reflection and runtime definition features.

### 11. Bytecode Design and Implementation

- [x] 11.1. Bytecode Instruction Set
  - [x] 11.1.1. Define `OpCode` enum in `src/bytecode/opcode.rs`
  - [x] 11.1.2. Add `OP_CONSTANT` (load constant from pool)
  - [x] 11.1.3. Add `OP_NIL`, `OP_TRUE`, `OP_FALSE`
  - [x] 11.1.4. Add `OP_POP` (pop from stack)
  - [x] 11.1.5. Add `OP_GET_LOCAL`, `OP_SET_LOCAL`
  - [x] 11.1.6. Add `OP_GET_GLOBAL`, `OP_SET_GLOBAL`
  - [x] 11.1.7. Add `OP_GET_INSTANCE`, `OP_SET_INSTANCE`
  - [x] 11.1.8. Add `OP_DEFINE_GLOBAL`
  - [x] 11.1.9. Add arithmetic ops: `OP_ADD`, `OP_SUBTRACT`, `OP_MULTIPLY`, `OP_DIVIDE`, `OP_MODULO`
  - [x] 11.1.10. Add comparison ops: `OP_EQUAL`, `OP_NOT_EQUAL`, `OP_GREATER`, `OP_LESS`, `OP_GREATER_EQUAL`, `OP_LESS_EQUAL`
  - [x] 11.1.11. Add unary ops: `OP_NEGATE`, `OP_NOT`
  - [x] 11.1.12. Add `OP_JUMP`, `OP_JUMP_IF_FALSE`, `OP_LOOP`
  - [x] 11.1.13. Add `OP_CALL` (method invocation)
  - [x] 11.1.14. Add `OP_INVOKE` (optimized method call)
  - [x] 11.1.15. Add `OP_CLOSURE` (create closure)
  - [x] 11.1.16. Add `OP_GET_UPVALUE`, `OP_SET_UPVALUE`, `OP_CLOSE_UPVALUE`
  - [x] 11.1.17. Add `OP_CLASS` (define class)
  - [x] 11.1.18. Add `OP_METHOD` (add method to class)
  - [x] 11.1.19. Add `OP_RETURN`
  - [x] 11.1.20. Add `OP_ARRAY` (create array)
  - [x] 11.1.21. Add `OP_HASH` (create hash)
  - [x] 11.1.22. Add `OP_INDEX_GET`, `OP_INDEX_SET`
  - [x] 11.1.23. Add `OP_TRY` (begin try block, push exception handler)
  - [x] 11.1.24. Add `OP_CATCH` (catch exception, pop handler)
  - [x] 11.1.25. Add `OP_FINALLY` (begin finally block)
  - [x] 11.1.26. Add `OP_END_TRY` (end try/catch/finally block)
  - [x] 11.1.27. Add `OP_RAISE` (throw exception)
  - [x] 11.1.28. Add `OP_MATCH` (pattern matching)
  - [x] 11.1.29. Add `OP_MATCH_PATTERN` (test pattern against value)
  - [x] 11.1.30. Document each opcode with comments
  - [x] 11.1.31. Create test file: `tests/bytecode/opcode_tests.rs`

- [x] 11.2. Chunk (Bytecode Container)
  - [x] 11.2.1. Define `Chunk` struct in `src/bytecode/chunk.rs`
  - [x] 11.2.2. Add `code: Vec<u8>` for bytecode
  - [x] 11.2.3. Add `constants: Vec<Object>` for constant pool
  - [x] 11.2.4. Add `lines: Vec<usize>` for line number mapping
  - [x] 11.2.5. Implement `write_byte()` method
  - [x] 11.2.6. Implement `add_constant()` method
  - [x] 11.2.7. Implement `get_line()` method for debugging
  - [x] 11.2.8. Implement chunk disassembler for debugging
  - [x] 11.2.9. Write unit tests for chunk operations
  - [x] 11.2.10. Create test file: `tests/bytecode/chunk_tests.rs`

- [x] 11.3. Bytecode Disassembler
  - [x] 11.3.1. Create disassembler in `src/bytecode/disassembler.rs`
  - [x] 11.3.2. Implement instruction formatting
  - [x] 11.3.3. Show constant values in disassembly
  - [x] 11.3.4. Show line numbers
  - [x] 11.3.5. Add offset information
  - [x] 11.3.6. Create readable output format
  - [x] 11.3.8. Write tests for disassembler
  - [x] 11.3.9. Create test file: `tests/bytecode/disassembler_tests.rs`

### 12. Compiler (AST to Bytecode)

- [x] 12.1. Compiler Core Structure
  - [x] 12.1.1. Define `Compiler` struct in `src/compiler/mod.rs`
  - [x] 12.1.2. Add current chunk reference
  - [x] 12.1.3. Add scope depth tracking
  - [x] 12.1.4. Add local variable stack
  - [x] 12.1.5. Add upvalue tracking for closures
  - [x] 12.1.6. Add enclosing compiler reference (for nested functions)
  - [x] 12.1.7. Implement `new()` constructor
  - [x] 12.1.8. Implement `compile()` entry point

- [x] 12.2. Expression Compilation
  - [x] 12.2.1. Implement literal compilation
  - [x] 12.2.2. Implement variable access compilation
  - [x] 12.2.3. Implement binary expression compilation
  - [x] 12.2.4. Implement unary expression compilation
  - [x] 12.2.5. Implement grouping expression compilation
  - [x] 12.2.6. Implement array literal compilation
  - [x] 12.2.7. Implement hash literal compilation
  - [x] 12.2.8. Implement index expression compilation
  - [x] 12.2.9. Implement method call compilation
  - [x] 12.2.10. Write unit tests for expression compilation
  - [x] 12.2.11. Create test file: `tests/compiler/compiler_expression_tests.rs`

- [x] 12.3. Statement Compilation
  - [x] 12.3.1. Implement expression statement compilation
  - [x] 12.3.2. Implement variable declaration compilation
  - [x] 12.3.3. Implement assignment compilation
  - [x] 12.3.4. Implement block compilation
  - [x] 12.3.5. Implement return statement compilation
  - [x] 12.3.6. Write unit tests for statement compilation
  - [x] 12.3.7. Create test file: `tests/compiler_statement_tests.rs`

- [x] 12.4. Control Flow Compilation
  - [x] 12.4.1. Implement if/else compilation with jumps
  - [x] 12.4.2. Implement while loop compilation
  - [x] 12.4.3. Implement for loop compilation
  - [x] 12.4.4. Implement break statement compilation
  - [x] 12.4.5. Implement continue statement compilation
  - [x] 12.4.6. Implement jump patching
  - [x] 12.4.7. Write unit tests for control flow compilation
  - [x] 12.4.8. Create test file: `tests/compiler_control_flow_tests.rs`

- [x] 12.5. Function and Method Compilation
  - [x] 12.5.1. Implement function definition compilation
  - [x] 12.5.2. Create nested compiler for function body
  - [x] 12.5.3. Compile function parameters
  - [x] 12.5.4. Emit `OP_RETURN` at function end
  - [x] 12.5.5. Store compiled function in constant pool
  - [x] 12.5.6. Implement method compilation
  - [x] 12.5.7. Write unit tests for function compilation
  - [x] 12.5.8. Create test file: `tests/compiler_function_tests.rs`

- [x] 12.6. Class Compilation
  - [x] 12.6.1. Implement class definition compilation
  - [x] 12.6.2. Emit `OP_CLASS` instruction
  - [x] 12.6.3. Compile class methods
  - [x] 12.6.4. Emit `OP_METHOD` for each method
  - [x] 12.6.5. Handle inheritance compilation
  - [x] 12.6.6. Write unit tests for class compilation
  - [x] 12.6.7. Create test file: `tests/compiler_class_tests.rs`

- [x] 12.7. Closure and Upvalue Compilation
  - [x] 12.7.1. Implement local variable resolution
  - [x] 12.7.2. Implement upvalue resolution
  - [x] 12.7.3. Track captured variables
  - [x] 12.7.4. Emit `OP_CLOSURE` with upvalue list
  - [x] 12.7.5. Emit `OP_CLOSE_UPVALUE` when needed
  - [x] 12.7.6. Write unit tests for closures
  - [x] 12.7.7. Create example file: `examples/compiler/closures.rb`
  - [x] 12.7.8. Create test file: `tests/compiler_closure_tests.rs`

- [x] 12.8. Block Compilation (Meta-Programming)
  - [x] 12.8.1. Compile blocks as closures
  - [x] 12.8.2. Capture block environment
  - [x] 12.8.3. Emit `OP_CLOSURE` for blocks
  - [x] 12.8.4. Store block object in constant pool
  - [x] 12.8.5. Write unit tests for block compilation
  - [x] 12.8.6. Create test file: `tests/compiler_block_tests.rs`

- [ ] 12.9. Optimization Passes
  - [ ] 12.9.1. Implement constant folding
  - [ ] 12.9.2. Implement dead code elimination
  - [ ] 12.9.3. Implement peephole optimization
  - [ ] 12.9.4. Optimize tail calls
  - [ ] 12.9.5. Write unit tests for optimizations
  - [ ] 12.9.6. Create test file: `tests/compiler_optimization_tests.rs`

### 13. Bytecode Virtual Machine

- [ ] 13.1. Stack-Based VM Structure
  - [ ] 13.1.1. Refactor `VirtualMachine` for bytecode execution
  - [ ] 13.1.2. Add value stack (`Vec<Object>`)
  - [ ] 13.1.3. Add call frame stack
  - [ ] 13.1.4. Add instruction pointer (IP)
  - [ ] 13.1.5. Add globals table
  - [ ] 13.1.6. Add open upvalues list
  - [ ] 13.1.7. Initialize stack with capacity
  - [ ] 13.1.8. Implement stack operations (push, pop, peek)

- [ ] 13.2. Call Frame Implementation
  - [ ] 13.2.1. Define `CallFrame` struct
  - [ ] 13.2.2. Add closure/function reference
  - [ ] 13.2.3. Add instruction pointer
  - [ ] 13.2.4. Add stack slot offset
  - [ ] 13.2.5. Implement frame push/pop operations
  - [ ] 13.2.6. Write unit tests for call frames
  - [ ] 13.2.7. Create test file: `tests/call_frame_tests.rs`

- [ ] 13.3. Bytecode Execution Loop
  - [ ] 13.3.1. Implement main `run()` loop
  - [ ] 13.3.2. Fetch-decode-execute cycle
  - [ ] 13.3.3. Read instruction at IP
  - [ ] 13.3.4. Dispatch to instruction handler
  - [ ] 13.3.5. Advance IP
  - [ ] 13.3.6. Handle errors and exceptions
  - [ ] 13.3.7. Add execution tracing for debugging

- [ ] 13.4. Basic Instruction Execution
  - [ ] 13.4.1. Implement `OP_CONSTANT` execution
  - [ ] 13.4.2. Implement `OP_NIL`, `OP_TRUE`, `OP_FALSE` execution
  - [ ] 13.4.3. Implement `OP_POP` execution
  - [ ] 13.4.4. Implement arithmetic operations
  - [ ] 13.4.5. Implement comparison operations
  - [ ] 13.4.6. Implement unary operations
  - [ ] 13.4.7. Write unit tests for basic instructions
  - [ ] 13.4.8. Create test file: `tests/vm_basic_instructions_tests.rs`

- [ ] 13.5. Variable Access Execution
  - [ ] 13.5.1. Implement `OP_GET_LOCAL` execution
  - [ ] 13.5.2. Implement `OP_SET_LOCAL` execution
  - [ ] 13.5.3. Implement `OP_GET_GLOBAL` execution
  - [ ] 13.5.4. Implement `OP_SET_GLOBAL` execution
  - [ ] 13.5.5. Implement `OP_DEFINE_GLOBAL` execution
  - [ ] 13.5.6. Write unit tests for variable access
  - [ ] 13.5.7. Create test file: `tests/vm_variable_tests.rs`

- [ ] 13.6. Control Flow Execution
  - [ ] 13.6.1. Implement `OP_JUMP` execution
  - [ ] 13.6.2. Implement `OP_JUMP_IF_FALSE` execution
  - [ ] 13.6.3. Implement `OP_LOOP` execution
  - [ ] 13.6.4. Write unit tests for control flow
  - [ ] 13.6.5. Create test file: `tests/vm_control_flow_tests.rs`

- [ ] 13.7. Function Call Execution
  - [ ] 13.7.1. Implement `OP_CALL` execution
  - [ ] 13.7.2. Create new call frame
  - [ ] 13.7.3. Bind arguments to parameters
  - [ ] 13.7.4. Handle argument count mismatches
  - [ ] 13.7.5. Implement `OP_RETURN` execution
  - [ ] 13.7.6. Pop call frame and restore IP
  - [ ] 13.7.7. Return value to caller
  - [ ] 13.7.8. Write unit tests for function calls
  - [ ] 13.7.9. Create test file: `tests/vm_function_call_tests.rs`

- [ ] 13.8. Closure and Upvalue Execution
  - [ ] 13.8.1. Implement `OP_CLOSURE` execution
  - [ ] 13.8.2. Create closure object with upvalues
  - [ ] 13.8.3. Implement `OP_GET_UPVALUE` execution
  - [ ] 13.8.4. Implement `OP_SET_UPVALUE` execution
  - [ ] 13.8.5. Implement `OP_CLOSE_UPVALUE` execution
  - [ ] 13.8.6. Close upvalues on scope exit
  - [ ] 13.8.7. Write unit tests for closures
  - [ ] 13.8.8. Create example file: `examples/vm/closures.rb`
  - [ ] 13.8.9. Create test file: `tests/vm_closure_tests.rs`

- [ ] 13.9. Class and Object Execution
  - [ ] 13.9.1. Implement `OP_CLASS` execution
  - [ ] 13.9.2. Create class object
  - [ ] 13.9.3. Implement `OP_METHOD` execution
  - [ ] 13.9.4. Add method to class
  - [ ] 13.9.5. Implement `OP_INVOKE` execution (optimized method call)
  - [ ] 13.9.6. Implement `OP_GET_INSTANCE` execution
  - [ ] 13.9.7. Implement `OP_SET_INSTANCE` execution
  - [ ] 13.9.8. Write unit tests for OOP
  - [ ] 13.9.9. Create test file: `tests/vm_oop_tests.rs`

- [ ] 13.10. Collection Execution
  - [ ] 13.10.1. Implement `OP_ARRAY` execution
  - [ ] 13.10.2. Implement `OP_HASH` execution
  - [ ] 13.10.3. Implement `OP_INDEX_GET` execution
  - [ ] 13.10.4. Implement `OP_INDEX_SET` execution
  - [ ] 13.10.5. Write unit tests for collections
  - [ ] 13.10.6. Create test file: `tests/vm_collections_tests.rs`

### 14. Advanced Meta-Programming (Bytecode Era)

- [ ] 14.1. Runtime Method Definition
  - [ ] 14.1.1. Update `define_method` to work with bytecode
  - [ ] 14.1.2. Compile block to bytecode on-the-fly
  - [ ] 14.1.3. Create Method object with bytecode
  - [ ] 14.1.4. Add to class method table
  - [ ] 14.1.5. Ensure callable from bytecode VM
  - [ ] 14.1.6. Write unit tests for runtime method definition
  - [ ] 14.1.7. Create example file: `examples/metaprogramming/bytecode_define_method.rb`
  - [ ] 14.1.8. Create test file: `tests/bytecode_define_method_tests.rs`

- [ ] 14.2. Method Missing Hook
  - [ ] 14.2.1. Add `method_missing` to Object class
  - [ ] 14.2.2. Check for `method_missing` on lookup failure
  - [ ] 14.2.3. Call `method_missing` with method name and args
  - [ ] 14.2.4. Write unit tests for method_missing
  - [ ] 14.2.5. Create example file: `examples/metaprogramming/method_missing.rb`
  - [ ] 14.2.6. Create test file: `tests/method_missing_tests.rs`

- [ ] 14.3. Runtime Class Modification
  - [ ] 14.3.1. Implement `remove_method`
  - [ ] 14.3.2. Implement `undef_method`
  - [ ] 14.3.3. Implement `alias_method`
  - [ ] 14.3.4. Implement `module_function`
  - [ ] 14.3.5. Write unit tests for class modification
  - [ ] 14.3.6. Create example file: `examples/metaprogramming/class_modification.rb`
  - [ ] 14.3.7. Create test file: `tests/class_modification_tests.rs`

- [ ] 14.4. Reflection and Introspection
  - [ ] 14.4.1. Implement `class` method
  - [ ] 14.4.2. Implement `instance_of?` method
  - [ ] 14.4.3. Implement `respond_to?` method
  - [ ] 14.4.4. Implement `methods` method (list all methods)
  - [ ] 14.4.5. Implement `instance_variables` method
  - [ ] 14.4.6. Implement `send` method (dynamic dispatch)
  - [ ] 14.4.7. Write unit tests for reflection
  - [ ] 14.4.8. Create example file: `examples/metaprogramming/reflection.rb`
  - [ ] 14.4.9. Create test file: `tests/reflection_tests.rs`

- [ ] 14.5. AST Manipulation
  - [ ] 14.5.1. Implement `eval` function (compile and execute string)
  - [ ] 14.5.2. Implement AST modification API
  - [ ] 14.5.3. Allow blocks to be modified at runtime
  - [ ] 14.5.4. Implement code generation helpers
  - [ ] 14.5.5. Write unit tests for AST manipulation
  - [ ] 14.5.6. Create example file: `examples/metaprogramming/ast_manipulation.rb`
  - [ ] 14.5.7. Create test file: `tests/ast_manipulation_tests.rs`

- [ ] 14.6. Trait/Interface System
  - [ ] 14.6.1. Define `Trait` struct in `src/trait.rs`
  - [ ] 14.6.2. Add trait name and method signatures
  - [ ] 14.6.3. Implement trait definition syntax parsing (`trait Drawable ... end`)
  - [ ] 14.6.4. Parse trait method declarations (without implementations)
  - [ ] 14.6.5. Parse trait implementation syntax (`impl Drawable for Circle ... end`)
  - [ ] 14.6.6. Add `OP_TRAIT` bytecode instruction (define trait)
  - [ ] 14.6.7. Add `OP_IMPL_TRAIT` bytecode instruction (implement trait)
  - [ ] 14.6.8. Add `OP_CHECK_TRAIT` bytecode instruction (runtime check)
  - [ ] 14.6.9. Implement trait table in VM (registry of traits)
  - [ ] 14.6.10. Implement trait implementation table (which types implement which traits)
  - [ ] 14.6.11. Implement trait method dispatch
  - [ ] 14.6.12. Implement default trait implementations
  - [ ] 14.6.13. Implement trait bounds for generic code
  - [ ] 14.6.14. Implement associated types in traits
  - [ ] 14.6.15. Implement trait objects (dynamic dispatch)
  - [ ] 14.6.16. Add `implements?` method to check trait implementation
  - [ ] 14.6.17. Write unit tests for trait system
  - [ ] 14.6.18. Create example file: `examples/traits/basic_traits.rb`
  - [ ] 14.6.19. Create example file: `examples/traits/trait_bounds.rb`
  - [ ] 14.6.20. Create test file: `tests/trait_system_tests.rs`

### 15. Performance and Memory Management

- [ ] 15.1. Basic Garbage Collection
  - [ ] 15.1.1. Implement reference counting for objects
  - [ ] 15.1.2. Detect reference cycles
  - [ ] 15.1.3. Implement cycle breaking
  - [ ] 15.1.4. Add weak references
  - [ ] 15.1.5. Write unit tests for GC
  - [ ] 15.1.6. Benchmark GC performance
  - [ ] 15.1.7. Create test file: `tests/gc_tests.rs`

- [ ] 15.2. Mark-and-Sweep GC (Alternative)
  - [ ] 15.2.1. Implement mark phase
  - [ ] 15.2.2. Implement sweep phase
  - [ ] 15.2.3. Integrate with VM execution
  - [ ] 15.2.4. Add GC pause time tracking
  - [ ] 15.2.5. Implement incremental marking
  - [ ] 15.2.6. Write unit tests for mark-and-sweep
  - [ ] 15.2.7. Create test file: `tests/mark_sweep_gc_tests.rs`

- [ ] 15.3. Memory Optimization
  - [ ] 15.3.1. Implement object pooling for common types
  - [ ] 15.3.2. Use string interning
  - [ ] 15.3.3. Optimize stack frame allocation
  - [ ] 15.3.4. Reduce boxing/unboxing overhead
  - [ ] 15.3.5. Profile memory usage
  - [ ] 15.3.6. Write memory stress tests
  - [ ] 15.3.7. Create test file: `tests/memory_optimization_tests.rs`

- [ ] 15.4. Performance Benchmarking
  - [ ] 15.4.1. Create benchmark suite
  - [ ] 15.4.2. Benchmark arithmetic operations
  - [ ] 15.4.3. Benchmark method dispatch
  - [ ] 15.4.4. Benchmark closure creation
  - [ ] 15.4.5. Benchmark object allocation
  - [ ] 15.4.6. Compare against other languages
  - [ ] 15.4.7. Create continuous benchmarking setup
  - [ ] 15.4.8. Create benchmark file: `benches/vm_benchmarks.rs`

---

## Phase 3: Optimization, Concurrency, and V1.0

Goal: Achieve production-ready performance, stability, and features necessary for ecosystem growth.

### 16. Module System

- [ ] 16.1. Module Object and Runtime Type
  - [ ] 16.1.1. Define `Module` runtime type in `src/module.rs`
  - [ ] 16.1.2. Add module's own global namespace (symbol table)
  - [ ] 16.1.3. Add module metadata (name, file path, load status)
  - [ ] 16.1.4. Implement `Module::new()` constructor
  - [ ] 16.1.5. Add `Object::Module` variant to runtime object system
  - [ ] 16.1.6. Write unit tests for module objects
  - [ ] 16.1.7. Create test file: `tests/module_object_tests.rs`

- [ ] 16.2. Import Syntax and Parsing
  - [ ] 16.2.1. Extend parser to recognize `import module` statement
  - [ ] 16.2.2. Implement `from module import name` syntax parsing
  - [ ] 16.2.3. Implement `import module as alias` syntax parsing
  - [ ] 16.2.4. Add `ImportStmt` AST node with variants for different import types
  - [ ] 16.2.5. Parse multiple imports in one statement (`from x import a, b, c`)
  - [ ] 16.2.6. Write unit tests for import parsing
  - [ ] 16.2.7. Create example file: `examples/modules/import_syntax.rb`
  - [ ] 16.2.8. Create test file: `tests/import_parsing_tests.rs`

- [ ] 16.3. File Loading System
  - [ ] 16.3.1. Implement file system layer to locate `.rb` files from disk
  - [ ] 16.3.2. Create module search path system (current directory, standard library paths)
  - [ ] 16.3.3. Implement module name to file path resolution
  - [ ] 16.3.4. Handle file not found errors gracefully
  - [ ] 16.3.5. Add support for absolute and relative file paths
  - [ ] 16.3.6. Write unit tests for file loading
  - [ ] 16.3.7. Create test file: `tests/module_file_loading_tests.rs`

- [ ] 16.4. Module Cache and Registry
  - [ ] 16.4.1. Create module registry/cache in the VM
  - [ ] 16.4.2. Store already-loaded modules by fully qualified name
  - [ ] 16.4.3. Implement cache lookup to prevent redundant compilation
  - [ ] 16.4.4. Add module loading state tracking (not loaded, loading, loaded)
  - [ ] 16.4.5. Implement cache invalidation for hot reload
  - [ ] 16.4.6. Write unit tests for module caching
  - [ ] 16.4.7. Create test file: `tests/module_cache_tests.rs`

- [ ] 16.5. Compilation and Initialization
  - [ ] 16.5.1. Implement module source lexing on import
  - [ ] 16.5.2. Compile module source into bytecode
  - [ ] 16.5.3. Execute bytecode in a fresh namespace to populate module's globals
  - [ ] 16.5.4. Handle compilation errors during import
  - [ ] 16.5.5. Handle runtime errors during module initialization
  - [ ] 16.5.6. Write unit tests for module compilation
  - [ ] 16.5.7. Create test file: `tests/module_compilation_tests.rs`

- [ ] 16.6. Import Opcodes
  - [ ] 16.6.1. Add `OP_IMPORT_MODULE` bytecode instruction
  - [ ] 16.6.2. Add `OP_IMPORT_FROM` bytecode instruction
  - [ ] 16.6.3. Implement compiler emission of import operations
  - [ ] 16.6.4. Implement VM execution of `OP_IMPORT_MODULE`
  - [ ] 16.6.5. Implement VM execution of `OP_IMPORT_FROM`
  - [ ] 16.6.6. Write unit tests for import opcodes
  - [ ] 16.6.7. Create test file: `tests/import_opcodes_tests.rs`

- [ ] 16.7. Namespace Binding
  - [ ] 16.7.1. Implement logic to bind imported names into importing module's namespace
  - [ ] 16.7.2. Handle `import math` creating local binding to module object
  - [ ] 16.7.3. Handle `from math import sqrt` binding specific name
  - [ ] 16.7.4. Handle `import math as m` creating aliased binding
  - [ ] 16.7.5. Handle import name conflicts (error or warning)
  - [ ] 16.7.6. Write unit tests for namespace binding
  - [ ] 16.7.7. Create example file: `examples/modules/namespace_binding.rb`
  - [ ] 16.7.8. Create test file: `tests/namespace_binding_tests.rs`

- [ ] 16.8. Attribute Access on Modules
  - [ ] 16.8.1. Extend `GET_ATTR` opcode to work on module objects
  - [ ] 16.8.2. Allow code like `math.sqrt` to retrieve symbols from module's namespace
  - [ ] 16.8.3. Implement attribute error for non-existent module attributes
  - [ ] 16.8.4. Write unit tests for module attribute access
  - [ ] 16.8.5. Create test file: `tests/module_attribute_access_tests.rs`

- [ ] 16.9. Circular Import Detection
  - [ ] 16.9.1. Add safeguards to detect circular dependencies
  - [ ] 16.9.2. Mark modules as "loading" during initialization
  - [ ] 16.9.3. Prevent infinite loops in circular imports
  - [ ] 16.9.4. Provide clear error messages for circular imports
  - [ ] 16.9.5. Write unit tests for circular import detection
  - [ ] 16.9.6. Create example file: `examples/modules/circular_imports.rb`
  - [ ] 16.9.7. Create test file: `tests/circular_import_tests.rs`

- [ ] 16.10. Relative Imports (Optional)
  - [ ] 16.10.1. Support relative import syntax (`from . import sibling`)
  - [ ] 16.10.2. Support parent imports (`from .. import parent`)
  - [ ] 16.10.3. Implement package-aware module resolution
  - [ ] 16.10.4. Handle relative import depth tracking
  - [ ] 16.10.5. Write unit tests for relative imports
  - [ ] 16.10.6. Create example file: `examples/modules/relative_imports.rb`
  - [ ] 16.10.7. Create test file: `tests/relative_import_tests.rs`

- [ ] 16.11. Package Support (Optional)
  - [ ] 16.11.1. Implement package semantics with `initialize.rb` files
  - [ ] 16.11.2. Support hierarchical module namespaces (`package.submodule`)
  - [ ] 16.11.3. Implement package initialization on first import
  - [ ] 16.11.4. Handle package-level imports
  - [ ] 16.11.5. Write unit tests for packages
  - [ ] 16.11.6. Create example: `examples/modules/package_example/`
  - [ ] 16.11.7. Create test file: `tests/package_tests.rs`

- [ ] 16.12. Standard Library Modules
  - [ ] 16.12.1. Create `Math` module
  - [ ] 16.12.2. Create `File` module
  - [ ] 16.12.3. Create `JSON` module
  - [ ] 16.12.4. Create `Time` module
  - [ ] 16.12.5. Create `Regex` module
  - [ ] 16.12.6. Write tests for stdlib modules
  - [ ] 16.12.7. Create examples for each module

- [ ] 16.13. Networking Library
  - [ ] 16.13.1. Implement `HTTP` module with client support
  - [ ] 16.13.2. Add HTTP GET, POST, PUT, DELETE methods
  - [ ] 16.13.3. Add request headers and body support
  - [ ] 16.13.4. Implement HTTP response parsing
  - [ ] 16.13.5. Implement `HTTPServer` class for creating HTTP servers
  - [ ] 16.13.6. Add routing support for HTTP servers
  - [ ] 16.13.7. Add middleware support
  - [ ] 16.13.8. Implement `WebSocket` client
  - [ ] 16.13.9. Implement `WebSocket` server
  - [ ] 16.13.10. Implement `TCP` socket support (client and server)
  - [ ] 16.13.11. Implement `UDP` socket support
  - [ ] 16.13.12. Add TLS/SSL support for secure connections
  - [ ] 16.13.13. Implement DNS resolution
  - [ ] 16.13.14. Add connection pooling for HTTP clients
  - [ ] 16.13.15. Write comprehensive tests for networking
  - [ ] 16.13.16. Create example file: `examples/networking/http_client.rb`
  - [ ] 16.13.17. Create example file: `examples/networking/http_server.rb`
  - [ ] 16.13.18. Create example file: `examples/networking/websocket.rb`
  - [ ] 16.13.19. Create test file: `tests/networking_tests.rs`

- [ ] 16.14. Serialization Library
  - [ ] 16.14.1. Enhance `JSON` module with streaming support
  - [ ] 16.14.2. Add JSON schema validation
  - [ ] 16.14.3. Implement `XML` module for parsing and generation
  - [ ] 16.14.4. Add XML schema validation
  - [ ] 16.14.5. Implement `YAML` module for parsing and generation
  - [ ] 16.14.6. Implement `TOML` module for configuration files
  - [ ] 16.14.7. Implement `CSV` module for CSV parsing and writing
  - [ ] 16.14.8. Add CSV dialect support (different delimiters, quotes)
  - [ ] 16.14.9. Implement `MessagePack` for binary serialization
  - [ ] 16.14.10. Implement native `Pickle` format for Metorex objects
  - [ ] 16.14.11. Add serialization traits for custom types
  - [ ] 16.14.12. Write tests for all serialization formats
  - [ ] 16.14.13. Create example file: `examples/serialization/json_advanced.rb`
  - [ ] 16.14.14. Create example file: `examples/serialization/xml.rb`
  - [ ] 16.14.15. Create example file: `examples/serialization/yaml.rb`
  - [ ] 16.14.16. Create test file: `tests/serialization_tests.rs`

- [ ] 16.15. Cryptography Library
  - [ ] 16.15.1. Implement `Hash` module with SHA-256, SHA-512 support
  - [ ] 16.15.2. Add MD5 hashing (with deprecation warning)
  - [ ] 16.15.3. Implement `HMAC` for message authentication
  - [ ] 16.15.4. Implement `Crypto` module for encryption/decryption
  - [ ] 16.15.5. Add AES encryption support (AES-128, AES-256)
  - [ ] 16.15.6. Add RSA encryption support
  - [ ] 16.15.7. Implement key generation for RSA
  - [ ] 16.15.8. Implement `SecureRandom` for cryptographically secure randomness
  - [ ] 16.15.9. Add password hashing (bcrypt, argon2)
  - [ ] 16.15.10. Implement digital signatures
  - [ ] 16.15.11. Add X.509 certificate parsing
  - [ ] 16.15.12. Add certificate validation
  - [ ] 16.15.13. Write comprehensive tests for cryptography
  - [ ] 16.15.14. Create example file: `examples/crypto/hashing.rb`
  - [ ] 16.15.15. Create example file: `examples/crypto/encryption.rb`
  - [ ] 16.15.16. Create example file: `examples/crypto/secure_random.rb`
  - [ ] 16.15.17. Create test file: `tests/crypto_tests.rs`

- [ ] 16.16. Advanced Collections
  - [ ] 16.16.1. Implement `Deque` (double-ended queue) class
  - [ ] 16.16.2. Add push/pop from both ends for Deque
  - [ ] 16.16.3. Implement `PriorityQueue` class with heap-based implementation
  - [ ] 16.16.4. Add custom comparator support for PriorityQueue
  - [ ] 16.16.5. Implement `TreeMap` for sorted key-value pairs
  - [ ] 16.16.6. Implement `TreeSet` for sorted unique values
  - [ ] 16.16.7. Implement `LinkedList` class
  - [ ] 16.16.8. Add iterator support for all collection types
  - [ ] 16.16.9. Implement immutable collection variants
  - [ ] 16.16.10. Add persistent data structure implementations
  - [ ] 16.16.11. Implement `CircularBuffer` class
  - [ ] 16.16.12. Write comprehensive tests for collections
  - [ ] 16.16.13. Create example file: `examples/collections/advanced.rb`
  - [ ] 16.16.14. Create test file: `tests/advanced_collections_tests.rs`

### 17. Foreign Function Interface (FFI)

- [ ] 17.1. Rust FFI Design
  - [ ] 17.1.1. Design API for registering Rust functions
  - [ ] 17.1.2. Create type conversion layer (Rust ↔ Metorex)
  - [ ] 17.1.3. Implement function registration macro
  - [ ] 17.1.4. Handle argument marshaling
  - [ ] 17.1.5. Handle return value marshaling
  - [ ] 17.1.6. Write unit tests for FFI
  - [ ] 17.1.7. Create example: `examples/ffi/rust_functions.rs`
  - [ ] 17.1.8. Create test file: `tests/ffi_tests.rs`

- [ ] 17.2. Native Extensions
  - [ ] 17.2.1. Create extension loading mechanism
  - [ ] 17.2.2. Support dynamic library loading
  - [ ] 17.2.3. Implement extension initialization
  - [ ] 17.2.4. Create sample extension project
  - [ ] 17.2.5. Write documentation for extension development
  - [ ] 17.2.6. Create example: `examples/ffi/sample_extension/`

- [ ] 17.3. C FFI (Optional)
  - [ ] 17.3.1. Design C API
  - [ ] 17.3.2. Generate C headers
  - [ ] 17.3.3. Implement C function calling
  - [ ] 17.3.4. Test with sample C library
  - [ ] 17.3.5. Create example: `examples/ffi/c_interop.rb`

### 18. Concurrency Support

- [ ] 18.1. Fiber/Green Thread Implementation
  - [ ] 18.1.1. Design fiber/coroutine model
  - [ ] 18.1.2. Implement fiber scheduler
  - [ ] 18.1.3. Implement `spawn` function
  - [ ] 18.1.4. Implement `yield` function
  - [ ] 18.1.5. Implement fiber communication (channels)
  - [ ] 18.1.6. Write unit tests for fibers
  - [ ] 18.1.7. Create example file: `examples/concurrency/fibers.rb`
  - [ ] 18.1.8. Create test file: `tests/fiber_tests.rs`

- [ ] 18.2. Async/Await Support
  - [ ] 18.2.1. Design async/await syntax
  - [ ] 18.2.2. Implement async function compilation
  - [ ] 18.2.3. Implement await expression
  - [ ] 18.2.4. Integrate with Rust async runtime
  - [ ] 18.2.5. Write unit tests for async
  - [ ] 18.2.6. Create example file: `examples/concurrency/async_await.rb`
  - [ ] 18.2.7. Create test file: `tests/async_await_tests.rs`

- [ ] 18.3. Thread Safety
  - [ ] 18.3.1. Implement thread-safe object sharing
  - [ ] 18.3.2. Add mutex/lock primitives
  - [ ] 18.3.3. Implement atomic operations
  - [ ] 18.3.4. Detect data races (optional)
  - [ ] 18.3.5. Write unit tests for thread safety
  - [ ] 18.3.6. Create test file: `tests/thread_safety_tests.rs`

- [ ] 18.4. OS-Level Threading
  - [ ] 18.4.1. Implement `Thread` class wrapping OS threads
  - [ ] 18.4.2. Add `Thread.new` constructor with closure
  - [ ] 18.4.3. Implement `Thread#start` method
  - [ ] 18.4.4. Implement `Thread#join` method
  - [ ] 18.4.5. Implement `Thread#detach` method
  - [ ] 18.4.6. Add thread naming support
  - [ ] 18.4.7. Implement `Thread.current` for current thread access
  - [ ] 18.4.8. Implement `Thread.sleep` method
  - [ ] 18.4.9. Implement `ThreadPool` class
  - [ ] 18.4.10. Add work stealing for thread pools
  - [ ] 18.4.11. Implement thread-local storage
  - [ ] 18.4.12. Add thread priority support
  - [ ] 18.4.13. Write comprehensive tests for threading
  - [ ] 18.4.14. Create example file: `examples/concurrency/threads.rb`
  - [ ] 18.4.15. Create test file: `tests/os_threads_tests.rs`

- [ ] 18.5. Synchronization Primitives
  - [ ] 18.5.1. Implement `Mutex` class for mutual exclusion
  - [ ] 18.5.2. Add `Mutex#lock` and `Mutex#unlock` methods
  - [ ] 18.5.3. Implement `Mutex#with_lock` with block (automatic unlock)
  - [ ] 18.5.4. Implement `RWLock` (reader-writer lock)
  - [ ] 18.5.5. Add read lock and write lock methods for RWLock
  - [ ] 18.5.6. Implement `Semaphore` class
  - [ ] 18.5.7. Add `acquire` and `release` methods for Semaphore
  - [ ] 18.5.8. Implement `ConditionVariable` for thread coordination
  - [ ] 18.5.9. Add `wait` and `notify` methods for ConditionVariable
  - [ ] 18.5.10. Implement `Barrier` for synchronizing multiple threads
  - [ ] 18.5.11. Implement deadlock detection (optional)
  - [ ] 18.5.12. Write comprehensive tests for synchronization
  - [ ] 18.5.13. Create example file: `examples/concurrency/synchronization.rb`
  - [ ] 18.5.14. Create test file: `tests/synchronization_tests.rs`

- [ ] 18.6. Atomic Operations and Lock-Free Structures
  - [ ] 18.6.1. Implement `Atomic` class wrapper
  - [ ] 18.6.2. Add `AtomicInt` for atomic integer operations
  - [ ] 18.6.3. Add `AtomicBool` for atomic boolean operations
  - [ ] 18.6.4. Implement `compare_and_swap` operation
  - [ ] 18.6.5. Implement `fetch_add`, `fetch_sub` operations
  - [ ] 18.6.6. Add memory ordering options (relaxed, acquire, release, seq_cst)
  - [ ] 18.6.7. Implement lock-free stack
  - [ ] 18.6.8. Implement lock-free queue
  - [ ] 18.6.9. Write comprehensive tests for atomics
  - [ ] 18.6.10. Create example file: `examples/concurrency/atomics.rb`
  - [ ] 18.6.11. Create test file: `tests/atomic_operations_tests.rs`

- [ ] 18.7. Channels and Message Passing
  - [ ] 18.7.1. Implement `Channel` class for message passing
  - [ ] 18.7.2. Add unbounded channel support
  - [ ] 18.7.3. Add bounded channel support with capacity
  - [ ] 18.7.4. Implement `send` and `receive` methods
  - [ ] 18.7.5. Implement non-blocking `try_send` and `try_receive`
  - [ ] 18.7.6. Implement `select` for waiting on multiple channels
  - [ ] 18.7.7. Add channel close semantics
  - [ ] 18.7.8. Implement MPSC (multi-producer, single-consumer) channels
  - [ ] 18.7.9. Implement MPMC (multi-producer, multi-consumer) channels
  - [ ] 18.7.10. Write comprehensive tests for channels
  - [ ] 18.7.11. Create example file: `examples/concurrency/channels.rb`
  - [ ] 18.7.12. Create test file: `tests/channels_tests.rs`

### 19. Advanced Optimization

- [ ] 19.1. JIT Preparation
  - [ ] 19.1.1. Implement bytecode profiling
  - [ ] 19.1.2. Identify hot paths
  - [ ] 19.1.3. Track type feedback
  - [ ] 19.1.4. Collect branch statistics
  - [ ] 19.1.5. Write profiler output to file
  - [ ] 19.1.6. Create visualization tools for profiling data
  - [ ] 19.1.7. Create test file: `tests/profiler_tests.rs`

- [ ] 19.2. Inline Caching
  - [ ] 19.2.1. Implement method lookup caching
  - [ ] 19.2.2. Implement polymorphic inline caches
  - [ ] 19.2.3. Invalidate caches on class modification
  - [ ] 19.2.4. Benchmark cache hit rates
  - [ ] 19.2.5. Write unit tests for inline caches
  - [ ] 19.2.6. Create test file: `tests/inline_cache_tests.rs`

- [ ] 19.3. Constant Propagation and Folding
  - [ ] 19.3.1. Implement compile-time constant evaluation
  - [ ] 19.3.2. Fold constant arithmetic
  - [ ] 19.3.3. Fold constant comparisons
  - [ ] 19.3.4. Propagate constants through code
  - [ ] 19.3.5. Write unit tests for constant folding
  - [ ] 19.3.6. Create test file: `tests/constant_folding_tests.rs`

- [ ] 19.4. Dead Code Elimination
  - [ ] 19.4.1. Detect unreachable code
  - [ ] 19.4.2. Remove unreachable basic blocks
  - [ ] 19.4.3. Remove unused variables
  - [ ] 19.4.4. Write unit tests for DCE
  - [ ] 19.4.5. Create test file: `tests/dead_code_elimination_tests.rs`

- [ ] 19.5. JIT Compilation with LLVM
  - [ ] 19.5.1. LLVM/Inkwell Setup: Set up the LLVM `Context`, `Module`, and `ExecutionEngine` using the `inkwell` crate to manage the JIT compilation process
  - [ ] 19.5.2. Hotspot Identification: Modify the VM to track block execution counts. Flag a bytecode block as "hot" when it is frequently executed
  - [ ] 19.5.3. Bytecode to IR Translator: Implement the core translation function: it takes a sequence of bytecode and generates equivalent, optimized LLVM IR instructions using the `inkwell` builder
  - [ ] 19.5.4. Runtime Switch: Implement the dynamic execution switch. When the VM reaches a JIT-compiled block, it calls the native machine code function generated by LLVM instead of interpreting the bytecode
  - [ ] 19.5.5. Write unit tests for JIT compilation
  - [ ] 19.5.6. Benchmark JIT vs interpreter performance
  - [ ] 19.5.7. Create example file: `examples/optimization/jit_compilation.rb`
  - [ ] 19.5.8. Create test file: `tests/jit_compilation_tests.rs`

- [ ] 19.6. Optional Type System
  - [ ] 19.6.1. Design type annotation syntax (`: Type` suffix)
  - [ ] 19.6.2. Parse type annotations in function signatures
  - [ ] 19.6.3. Parse type annotations in variable declarations
  - [ ] 19.6.4. Implement basic types (Int, Float, String, Bool, Nil, Any)
  - [ ] 19.6.5. Implement generic types (`Array<T>`, `Dict<K, V>`)
  - [ ] 19.6.6. Implement union types (`Int | String`)
  - [ ] 19.6.7. Implement type aliases (`type Point = {x: Int, y: Int}`)
  - [ ] 19.6.8. Implement type inference engine
  - [ ] 19.6.9. Implement Hindley-Milner style inference
  - [ ] 19.6.10. Implement gradual typing (mix of static and dynamic)
  - [ ] 19.6.11. Implement type checking pass in compiler
  - [ ] 19.6.12. Add type checking opcodes (`OP_TYPE_CHECK`, `OP_TYPE_ASSERT`)
  - [ ] 19.6.13. Implement runtime type guards
  - [ ] 19.6.14. Add configurable type checking modes (strict, gradual, dynamic)
  - [ ] 19.6.15. Implement type error reporting with suggestions
  - [ ] 19.6.16. Write comprehensive tests for type system
  - [ ] 19.6.17. Create example file: `examples/types/type_annotations.rb`
  - [ ] 19.6.18. Create example file: `examples/types/generics.rb`
  - [ ] 19.6.19. Create test file: `tests/type_system_tests.rs`

### 20. Tooling and Developer Experience

- [ ] 20.1. Debugger
  - [ ] 20.1.1. Implement breakpoint support
  - [ ] 20.1.2. Add stepping (step over, step into, step out)
  - [ ] 20.1.3. Add variable inspection
  - [ ] 20.1.4. Add call stack inspection
  - [ ] 20.1.5. Implement watch expressions
  - [ ] 20.1.6. Create debugger REPL
  - [ ] 20.1.7. Integrate with VS Code (DAP)
  - [ ] 20.1.8. Write documentation for debugger
  - [ ] 20.1.9. Create example: `examples/tools/debugging.rb`

- [ ] 20.2. Language Server Protocol (LSP)
  - [ ] 20.2.1. Implement LSP server
  - [ ] 20.2.2. Add syntax highlighting
  - [ ] 20.2.3. Add code completion
  - [ ] 20.2.4. Add go-to-definition
  - [ ] 20.2.5. Add find references
  - [ ] 20.2.6. Add hover documentation
  - [ ] 20.2.7. Add diagnostics (errors/warnings)
  - [ ] 20.2.8. Create VS Code extension
  - [ ] 20.2.9. Test with multiple editors

- [ ] 20.3. Package Manager
  - [ ] 20.3.1. Design package manifest format
  - [ ] 20.3.2. Implement package registry client
  - [ ] 20.3.3. Implement dependency resolution
  - [ ] 20.3.4. Implement package installation
  - [ ] 20.3.5. Implement version management
  - [ ] 20.3.6. Create publish workflow
  - [ ] 20.3.7. Write package manager CLI
  - [ ] 20.3.8. Create documentation for package system

- [ ] 20.4. Formatter
  - [ ] 20.4.1. Implement code formatter
  - [ ] 20.4.2. Define formatting rules
  - [ ] 20.4.3. Add CLI for formatting
  - [ ] 20.4.4. Add editor integration
  - [ ] 20.4.5. Write tests for formatter
  - [ ] 20.4.6. Create test file: `tests/formatter_tests.rs`

- [ ] 20.5. Linter
  - [ ] 20.5.1. Implement linter framework
  - [ ] 20.5.2. Add common lint rules
  - [ ] 20.5.3. Add configurable rule sets
  - [ ] 20.5.4. Add autofix capabilities
  - [ ] 20.5.5. Write tests for linter
  - [ ] 20.5.6. Create test file: `tests/linter_tests.rs`

- [ ] 20.6. Documentation System
  - [ ] 20.6.1. Design doc comment syntax (`///` for single-line, `/ */` for multi-line)
  - [ ] 20.6.2. Parse doc comments in lexer
  - [ ] 20.6.3. Attach doc comments to AST nodes
  - [ ] 20.6.4. Support Markdown in doc comments
  - [ ] 20.6.5. Implement code examples in doc comments
  - [ ] 20.6.6. Implement doc tests (executable code in docs)
  - [ ] 20.6.7. Create doc test runner
  - [ ] 20.6.8. Implement documentation generator (`mxdoc` tool)
  - [ ] 20.6.9. Generate HTML documentation from source
  - [ ] 20.6.10. Add cross-referencing between docs
  - [ ] 20.6.11. Implement search functionality in generated docs
  - [ ] 20.6.12. Add syntax highlighting in doc code examples
  - [ ] 20.6.13. Support multiple documentation formats (HTML, Markdown, JSON)
  - [ ] 20.6.14. Implement versioned documentation
  - [ ] 20.6.15. Write comprehensive tests for doc system
  - [ ] 20.6.16. Create example file: `examples/docs/documented_module.rb`
  - [ ] 20.6.17. Create test file: `tests/documentation_tests.rs`

- [ ] 20.7. Build System Features
  - [ ] 20.7.1. Implement build configuration file (`build.rb` or `Metorex.toml`)
  - [ ] 20.7.2. Add build profiles (debug, release, custom)
  - [ ] 20.7.3. Implement incremental compilation
  - [ ] 20.7.4. Track file dependencies for incremental builds
  - [ ] 20.7.5. Implement build scripts (pre-build, post-build hooks)
  - [ ] 20.7.6. Add conditional compilation support
  - [ ] 20.7.7. Implement feature flags
  - [ ] 20.7.8. Add cross-compilation support
  - [ ] 20.7.9. Implement parallel compilation
  - [ ] 20.7.10. Add build cache for compilation artifacts
  - [ ] 20.7.11. Implement link-time optimization (LTO)
  - [ ] 20.7.12. Add build progress reporting
  - [ ] 20.7.13. Implement clean build command
  - [ ] 20.7.14. Write comprehensive tests for build system
  - [ ] 20.7.15. Create example file: `examples/build/build_config.rb`
  - [ ] 20.7.16. Create test file: `tests/build_system_tests.rs`

### 21. Documentation and Release

- [ ] 21.1. Language Documentation
  - [ ] 21.1.1. Write language reference manual
  - [ ] 21.1.2. Document all syntax features
  - [ ] 21.1.3. Document all built-in classes and methods
  - [ ] 21.1.4. Write meta-programming guide
  - [ ] 21.1.5. Write getting started tutorial
  - [ ] 21.1.6. Create API documentation
  - [ ] 21.1.7. Add code examples throughout

- [ ] 21.2. Developer Documentation
  - [ ] 21.2.1. Write architecture overview
  - [ ] 21.2.2. Document compiler internals
  - [ ] 21.2.3. Document VM internals
  - [ ] 21.2.4. Document object system
  - [ ] 21.2.5. Write contributing guide
  - [ ] 21.2.6. Document build process
  - [ ] 21.2.7. Create API docs for Rust crates

- [ ] 21.3. Comprehensive Test Coverage
  - [ ] 21.3.1. Achieve 90%+ code coverage
  - [ ] 21.3.2. Add edge case tests
  - [ ] 21.3.3. Add stress tests
  - [ ] 21.3.4. Add fuzz testing
  - [ ] 21.3.5. Run tests on multiple platforms
  - [ ] 21.3.6. Set up continuous testing

- [ ] 21.4. Release Engineering
  - [ ] 21.4.1. Create release checklist
  - [ ] 21.4.2. Set up versioning scheme
  - [ ] 21.4.3. Create changelog
  - [ ] 21.4.4. Build binaries for multiple platforms
  - [ ] 21.4.5. Create installation packages
  - [ ] 21.4.6. Publish to package repositories
  - [ ] 21.4.7. Write release announcement
  - [ ] 21.4.8. Tag v1.0.0 release

### 22. Community and Ecosystem

- [ ] 22.1. Community Infrastructure
  - [ ] 22.1.1. Create website
  - [ ] 22.1.2. Set up discussion forum
  - [ ] 22.1.3. Create Discord/Slack community
  - [ ] 22.1.4. Set up issue templates
  - [ ] 22.1.5. Create code of conduct
  - [ ] 22.1.6. Set up governance model

- [ ] 22.2. Example Projects
  - [ ] 22.2.1. Build sample web application
  - [ ] 22.2.2. Build sample CLI tool
  - [ ] 22.2.3. Build sample game
  - [ ] 22.2.4. Build sample data analysis script
  - [ ] 22.2.5. Build sample API server
  - [ ] 22.2.6. Publish example projects

- [ ] 22.3. Educational Materials
  - [ ] 22.3.1. Create video tutorials
  - [ ] 22.3.2. Write blog posts
  - [ ] 22.3.3. Create interactive playground
  - [ ] 22.3.4. Write comparison guides (vs Python, vs Ruby, vs Rust)
  - [ ] 22.3.5. Create cheat sheet

---

## Phase 4: Advanced Language Features

Goal: Implement advanced features that distinguish METOREX as a cutting-edge programming language with best-in-class developer experience and powerful abstractions.

### 23. Macros and Compile-Time Features

- [ ] 23.1. Hygienic Macro System
  - [ ] 23.1.1. Design macro syntax (`macro name(args) ... end`)
  - [ ] 23.1.2. Parse macro definitions
  - [ ] 23.1.3. Implement macro expansion phase
  - [ ] 23.1.4. Implement hygiene (prevent variable capture)
  - [ ] 23.1.5. Add macro pattern matching
  - [ ] 23.1.6. Implement repetition patterns (`$(...)*`, `$(...)+`)
  - [ ] 23.1.7. Add macro debugging tools
  - [ ] 23.1.8. Write comprehensive tests for macros
  - [ ] 23.1.9. Create example file: `examples/macros/basic_macros.rb`
  - [ ] 23.1.10. Create test file: `tests/macro_tests.rs`

- [ ] 23.2. Procedural Macros
  - [ ] 23.2.1. Design procedural macro API
  - [ ] 23.2.2. Implement function-like procedural macros
  - [ ] 23.2.3. Implement derive macros (auto-implement traits)
  - [ ] 23.2.4. Implement attribute macros
  - [ ] 23.2.5. Provide AST manipulation API for macros
  - [ ] 23.2.6. Write tests for procedural macros
  - [ ] 23.2.7. Create example file: `examples/macros/procedural.rb`
  - [ ] 23.2.8. Create test file: `tests/procedural_macro_tests.rs`

- [ ] 23.3. Compile-Time Evaluation
  - [ ] 23.3.1. Implement `comptime` keyword for compile-time execution
  - [ ] 23.3.2. Add const evaluation engine
  - [ ] 23.3.3. Implement compile-time function calls
  - [ ] 23.3.4. Add compile-time assertions (`static_assert`)
  - [ ] 23.3.5. Implement const generics (type-level integers)
  - [ ] 23.3.6. Write tests for compile-time features
  - [ ] 23.3.7. Create example file: `examples/comptime/const_eval.rb`
  - [ ] 23.3.8. Create test file: `tests/comptime_tests.rs`

### 24. Functional Programming Features

- [ ] 24.1. Advanced Function Features
  - [ ] 24.1.1. Implement partial application
  - [ ] 24.1.2. Implement automatic currying
  - [ ] 24.1.3. Implement function composition operator (`compose`, `∘`)
  - [ ] 24.1.4. Implement pipe operator (`|>`)
  - [ ] 24.1.5. Add reverse pipe operator (`<|`)
  - [ ] 24.1.6. Write tests for function features
  - [ ] 24.1.7. Create example file: `examples/functional/function_composition.rb`
  - [ ] 24.1.8. Create test file: `tests/functional_features_tests.rs`

- [ ] 24.2. Algebraic Data Types
  - [ ] 24.2.1. Implement sum types (tagged unions)
  - [ ] 24.2.2. Implement product types (records/structs)
  - [ ] 24.2.3. Implement enum variants with data
  - [ ] 24.2.4. Add exhaustive pattern matching for ADTs
  - [ ] 24.2.5. Implement `Option<T>` type (Some/None)
  - [ ] 24.2.6. Implement `Result<T, E>` type (Ok/Err) - full implementation
  - [ ] 24.2.7. Add question mark operator (`?`) for error propagation
  - [ ] 24.2.8. Write comprehensive tests for ADTs
  - [ ] 24.2.9. Create example file: `examples/functional/adt.rb`
  - [ ] 24.2.10. Create test file: `tests/adt_tests.rs`

- [ ] 24.3. Immutable Data Structures
  - [ ] 24.3.1. Implement persistent vector
  - [ ] 24.3.2. Implement persistent map
  - [ ] 24.3.3. Implement persistent set
  - [ ] 24.3.4. Add structural sharing for efficiency
  - [ ] 24.3.5. Implement transients for batch updates
  - [ ] 24.3.6. Write tests for persistent structures
  - [ ] 24.3.7. Create example file: `examples/functional/immutable.rb`
  - [ ] 24.3.8. Create test file: `tests/persistent_structures_tests.rs`

- [ ] 24.4. Lenses and Optics
  - [ ] 24.4.1. Implement lens abstraction
  - [ ] 24.4.2. Implement prism abstraction
  - [ ] 24.4.3. Implement traversal abstraction
  - [ ] 24.4.4. Add lens composition
  - [ ] 24.4.5. Write tests for optics
  - [ ] 24.4.6. Create example file: `examples/functional/lenses.rb`
  - [ ] 24.4.7. Create test file: `tests/optics_tests.rs`

### 25. WebAssembly Support

- [ ] 25.1. WebAssembly Compilation
  - [ ] 25.1.1. Implement Wasm backend for compiler
  - [ ] 25.1.2. Generate Wasm binary format
  - [ ] 25.1.3. Implement Wasm import/export handling
  - [ ] 25.1.4. Add Wasm memory management
  - [ ] 25.1.5. Optimize for Wasm execution
  - [ ] 25.1.6. Write tests for Wasm compilation
  - [ ] 25.1.7. Create example file: `examples/wasm/basic.rb`
  - [ ] 25.1.8. Create test file: `tests/wasm_compilation_tests.rs`

- [ ] 25.2. WASI Support
  - [ ] 25.2.1. Implement WASI system interface
  - [ ] 25.2.2. Add file system access via WASI
  - [ ] 25.2.3. Add environment variable access
  - [ ] 25.2.4. Add command-line argument handling
  - [ ] 25.2.5. Write tests for WASI
  - [ ] 25.2.6. Create example file: `examples/wasm/wasi.rb`
  - [ ] 25.2.7. Create test file: `tests/wasi_tests.rs`

- [ ] 25.3. Browser Integration
  - [ ] 25.3.1. Create JavaScript bindings generator
  - [ ] 25.3.2. Implement DOM manipulation API
  - [ ] 25.3.3. Add event handling for browser
  - [ ] 25.3.4. Implement fetch API wrapper
  - [ ] 25.3.5. Write tests for browser integration
  - [ ] 25.3.6. Create example file: `examples/wasm/browser_app.rb`
  - [ ] 25.3.7. Create test file: `tests/browser_integration_tests.rs`

### 26. Security Features

- [ ] 26.1. Sandboxing and Isolation
  - [ ] 26.1.1. Implement sandboxed execution mode
  - [ ] 26.1.2. Add resource limits (CPU, memory, time)
  - [ ] 26.1.3. Implement capability-based security
  - [ ] 26.1.4. Add filesystem sandboxing
  - [ ] 26.1.5. Implement network isolation
  - [ ] 26.1.6. Write tests for sandboxing
  - [ ] 26.1.7. Create example file: `examples/security/sandbox.rb`
  - [ ] 26.1.8. Create test file: `tests/sandbox_tests.rs`

- [ ] 26.2. Taint Checking
  - [ ] 26.2.1. Implement taint tracking for untrusted data
  - [ ] 26.2.2. Add taint propagation rules
  - [ ] 26.2.3. Implement sanitization functions
  - [ ] 26.2.4. Add taint checking at dangerous operations
  - [ ] 26.2.5. Write tests for taint checking
  - [ ] 26.2.6. Create example file: `examples/security/taint_checking.rb`
  - [ ] 26.2.7. Create test file: `tests/taint_checking_tests.rs`

- [ ] 26.3. Security Policies
  - [ ] 26.3.1. Implement Content Security Policy equivalent
  - [ ] 26.3.2. Add security policy configuration
  - [ ] 26.3.3. Implement safe mode (restricted features)
  - [ ] 26.3.4. Add security audit logging
  - [ ] 26.3.5. Write tests for security policies
  - [ ] 26.3.6. Create example file: `examples/security/policies.rb`
  - [ ] 26.3.7. Create test file: `tests/security_policy_tests.rs`

### 27. Advanced Tooling

- [ ] 27.1. CPU Profiler
  - [ ] 27.1.1. Implement sampling-based profiler
  - [ ] 27.1.2. Implement instrumentation-based profiler
  - [ ] 27.1.3. Generate flame graphs
  - [ ] 27.1.4. Add call graph generation
  - [ ] 27.1.5. Implement profile-guided optimization
  - [ ] 27.1.6. Write tests for profiler
  - [ ] 27.1.7. Create example file: `examples/profiling/cpu_profile.rb`
  - [ ] 27.1.8. Create test file: `tests/profiler_tests.rs`

- [ ] 27.2. Memory Profiler
  - [ ] 27.2.1. Implement heap profiler
  - [ ] 27.2.2. Track allocation sources
  - [ ] 27.2.3. Generate heap dumps
  - [ ] 27.2.4. Implement leak detector
  - [ ] 27.2.5. Add memory timeline visualization
  - [ ] 27.2.6. Write tests for memory profiler
  - [ ] 27.2.7. Create example file: `examples/profiling/memory_profile.rb`
  - [ ] 27.2.8. Create test file: `tests/memory_profiler_tests.rs`

- [ ] 27.3. Static Analysis Tools
  - [ ] 27.3.1. Implement control flow analysis
  - [ ] 27.3.2. Implement data flow analysis
  - [ ] 27.3.3. Add dead code detection
  - [ ] 27.3.4. Implement unused variable detection
  - [ ] 27.3.5. Add complexity metrics
  - [ ] 27.3.6. Write tests for static analysis
  - [ ] 27.3.7. Create test file: `tests/static_analysis_tests.rs`

### 28. Interoperability

- [ ] 28.1. C ABI Compatibility
  - [ ] 28.1.1. Implement C ABI calling convention
  - [ ] 28.1.2. Generate C header files from Metorex code
  - [ ] 28.1.3. Add struct layout control (`#[repr(C)]`)
  - [ ] 28.1.4. Write tests for C ABI
  - [ ] 28.1.5. Create example file: `examples/interop/c_abi.rb`
  - [ ] 28.1.6. Create test file: `tests/c_abi_tests.rs`

- [ ] 28.2. Language Bridges
  - [ ] 28.2.1. Implement Python bridge (call Python from Metorex)
  - [ ] 28.2.2. Implement JavaScript bridge (Node.js/Deno)
  - [ ] 28.2.3. Implement embedding API (use Metorex from other languages)
  - [ ] 28.2.4. Generate bindings for host languages
  - [ ] 28.2.5. Write tests for language bridges
  - [ ] 28.2.6. Create example file: `examples/interop/python_bridge.rb`
  - [ ] 28.2.7. Create test file: `tests/language_bridge_tests.rs`

- [ ] 28.3. Protocol Support
  - [ ] 28.3.1. Implement gRPC support
  - [ ] 28.3.2. Add GraphQL support
  - [ ] 28.3.3. Implement message queue support (AMQP, MQTT)
  - [ ] 28.3.4. Write tests for protocols
  - [ ] 28.3.5. Create example file: `examples/interop/grpc.rb`
  - [ ] 28.3.6. Create test file: `tests/protocol_tests.rs`

### 29. Language Specification

- [ ] 29.1. Formal Grammar
  - [ ] 29.1.1. Write EBNF grammar specification
  - [ ] 29.1.2. Document operator precedence formally
  - [ ] 29.1.3. Add grammar railroad diagrams
  - [ ] 29.1.4. Validate grammar completeness
  - [ ] 29.1.5. Create grammar test suite

- [ ] 29.2. Language Specification Document
  - [ ] 29.2.1. Write formal semantics specification
  - [ ] 29.2.2. Document type system formally
  - [ ] 29.2.3. Document memory model
  - [ ] 29.2.4. Document concurrency model
  - [ ] 29.2.5. Add examples throughout specification

- [ ] 29.3. Conformance Test Suite
  - [ ] 29.3.1. Create comprehensive test suite for language features
  - [ ] 29.3.2. Add edge case tests
  - [ ] 29.3.3. Add stress tests
  - [ ] 29.3.4. Implement conformance testing framework
  - [ ] 29.3.5. Document conformance requirements

### 30. Deployment and Distribution

- [ ] 30.1. Packaging
  - [ ] 30.1.1. Create official Docker images
  - [ ] 30.1.2. Build standalone executables (no runtime dependencies)
  - [ ] 30.1.3. Implement ahead-of-time (AOT) compilation
  - [ ] 30.1.4. Add binary stripping for smaller size
  - [ ] 30.1.5. Create installation packages (deb, rpm, pkg, msi)
  - [ ] 30.1.6. Write packaging tests

- [ ] 30.2. Distribution
  - [ ] 30.2.1. Set up official package repositories
  - [ ] 30.2.2. Publish to package managers (brew, apt, scoop)
  - [ ] 30.2.3. Create install scripts
  - [ ] 30.2.4. Set up automatic update mechanism
  - [ ] 30.2.5. Write distribution documentation

- [ ] 30.3. Deployment Tools
  - [ ] 30.3.1. Create deployment templates
  - [ ] 30.3.2. Add cloud platform integrations
  - [ ] 30.3.3. Implement serverless deployment support
  - [ ] 30.3.4. Add container orchestration configs (K8s, Docker Compose)
  - [ ] 30.3.5. Write deployment guides

### 31. Metorex-to-Rust Transpiler (Experimental Track)

Goal: Add an AOT transpilation pipeline that converts supported Metorex programs into Rust code plus a small runtime, while preserving current language behavior for the supported subset.

- [ ] 31.1. Scope and Compatibility Contract
  - [ ] 31.1.1. Define the v1 transpiler subset (literals, variables, functions, classes, control flow)
  - [ ] 31.1.2. Define unsupported/deferred features (open classes, method_missing, runtime monkey patching, advanced metaprogramming)
  - [ ] 31.1.3. Define behavior parity criteria between VM execution and transpiled execution
  - [ ] 31.1.4. Document the transpiler architecture and tradeoffs in a dedicated design doc

- [ ] 31.2. Transpiler Pipeline and IR
  - [ ] 31.2.1. Add transpiler crate/module structure (`src/transpiler/`)
  - [ ] 31.2.2. Define a backend-friendly intermediate representation for lowering from AST
  - [ ] 31.2.3. Implement AST-to-IR lowering for core expressions and statements
  - [ ] 31.2.4. Implement IR validation pass with clear diagnostics
  - [ ] 31.2.5. Add unit tests for lowering and IR validation
  - [ ] 31.2.6. Create example file: `examples/transpiler/pipeline_demo.rb`

- [ ] 31.3. Rust Code Generation
  - [ ] 31.3.1. Implement Rust code generation for literals, arithmetic, and variable bindings
  - [ ] 31.3.2. Implement function/method code generation and call wiring
  - [ ] 31.3.3. Implement control flow code generation (`if`, `while`, `for`, `case` subset)
  - [ ] 31.3.4. Generate a compilable Rust project layout for transpiled output
  - [ ] 31.3.5. Add golden tests for generated Rust source stability
  - [ ] 31.3.6. Create example file: `examples/transpiler/basic_codegen.rb`

- [ ] 31.4. Runtime Support Layer
  - [ ] 31.4.1. Create a minimal runtime crate for dynamic object behavior used by generated code
  - [ ] 31.4.2. Implement method dispatch helpers used by generated call sites
  - [ ] 31.4.3. Implement exception/error bridging between language semantics and Rust results
  - [ ] 31.4.4. Implement collection/object helpers required by transpiled output
  - [ ] 31.4.5. Add unit tests for runtime primitives

- [ ] 31.5. Correctness, Examples, and CLI Integration
  - [ ] 31.5.1. Add parity tests: run script in VM and transpiled binary, compare output/errors
  - [ ] 31.5.2. Add transpiler test scripts in `tests/_examples/` and include them in `examples_runner.rs`; add user-facing examples in `examples/transpiler/`
  - [ ] 31.5.3. Add CLI command/flag for transpilation (`--transpile` or `metorex transpile`) — prerequisite: section 9.1 (CLI) must be complete
  - [ ] 31.5.4. Add integration tests for end-to-end transpilation + execution
  - [ ] 31.5.5. Document transpiler usage and subset limits in README/roadmap docs

- [ ] 31.6. Optimization and Type-Aware Lowering
  - [ ] 31.6.1. Add optional type-guided lowering for native Rust primitives where safe
  - [ ] 31.6.2. Add fallback path to dynamic runtime calls when static lowering is unsafe
  - [ ] 31.6.3. Implement basic optimization passes for generated code (constant folding, dead branch pruning)
  - [ ] 31.6.4. Add benchmarks comparing VM vs transpiled execution on representative workloads
  - [ ] 31.6.5. Define exit criteria for promoting transpiler from experimental to supported

### Future Enhancements

- [ ] Add CLI flag `--disassemble` to show bytecode (requires compiler)
- [ ] 11.4. Runtime Compilation for Meta-Programming (`EVAL_STRING`, `BUILD_LAMBDA`, `DEFINE_METHOD_RUNTIME`, `GET_METHOD_AST`, `SET_METHOD_AST`, compiler API for runtime use, AST preservation)
- [ ] 11.5. Closures, GC, and Yield (`CAPTURE_VAR`, upvalue closing, GC root scanning, `YIELD`, `CALL_BLOCK`, `BLOCK_GIVEN`, non-local returns, inline method cache)
- [ ] Regex patterns in case/when: `when /pattern/ then ...` (requires Regex type)
- [ ] Achieve 100% code coverage (current: 94.67% — remaining gaps are native method error paths, interactive REPL wrapper, and main.rs)
- [ ] Examples runner: `tests/_examples/advanced/dynamic_method_definition.rb` (requires section 7.2–7.3)
- [ ] Examples runner: `tests/_examples/advanced/implicit_block_capture.rb` (requires section 7.2–7.3)
- [ ] Examples runner: `tests/_examples/advanced/dsl_example.rb` (requires section 3.13, 7.6)
- [ ] Examples runner: `tests/_examples/advanced/serialization.rb` (requires section 3.13, 7.6)
- [ ] Examples runner: `tests/_examples/advanced/concurrency.rb` (requires Phase 3)
- [ ] Examples runner: `tests/_examples/advanced/networking.rb` (requires Phase 3)
- [ ] Create example: Simple web server (requires networking)
- [ ] Implement `require` (non-relative, uses `$LOAD_PATH`)
- [ ] Implement `load` (always re-loads file)
- [ ] Implement `autoload` (lazy loading)
- [ ] Add `__FILE__` and `__LINE__` magic constants
- [ ] Move `require_relative` into a proper `Kernel` module
- [ ] REPL syntax highlighting
- [ ] REPL auto-completion

