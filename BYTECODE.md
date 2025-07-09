# Bytecode VM Implementation Plan

This document outlines the comprehensive plan for transitioning Metorex from a tree-walking AST interpreter to a bytecode-compiled virtual machine. This is **Phase 2** of the Metorex roadmap.

## Overview

**Current State**: Metorex uses a tree-walking interpreter that directly executes the AST ([src/vm/core.rs](src/vm/core.rs:104-181))

**Target State**: A bytecode compiler that transforms the AST into compact bytecode instructions, executed by a stack-based virtual machine

**Key Benefits**:
- **Performance**: 5-10x faster execution than tree-walking interpretation
- **Memory Efficiency**: Bytecode is more compact than AST nodes
- **Optimization Opportunities**: Bytecode enables peephole optimization, constant folding, and JIT preparation
- **Debugging**: Bytecode format supports better debugger integration and profiling

**Key Challenges**:
- Preserving meta-programming capabilities (Code-as-Object)
- Maintaining exception handling with proper stack unwinding
- Supporting runtime method definition and reflection
- Ensuring all existing tests continue to pass

**Critical Implementation Risks** (Addressed in Phase 1.4 and 1.5):
1. **Meta-Programming Support** (Phase 1.4): Runtime code compilation via `eval`, `define_method`, etc. Must design early.
2. **Closure/Upvalue Management** (Phase 1.5): Most complex part of bytecode VMs - variables must "close over" when scopes end
3. **Garbage Collection Integration** (Phase 1.5): Stack, call frames, and constant pool must properly integrate with GC root scanning
4. **Block Yield Semantics** (Phase 1.5): Ruby-like yield behavior with non-local returns is tricky
5. **Method Resolution Caching** (Phase 1.5): Performance depends on efficient method lookup with cache invalidation
6. **Operator Overloading** (Phase 1.5): Custom operators must work seamlessly while keeping built-in types fast

These risks are addressed early (Phase 1.4-1.5) because they affect the fundamental VM design and cannot be easily retrofitted later.

---

## Phase 1: Foundation - Bytecode Instruction Set Design

### 1.1 Define Core Instruction Set

- [ ] 1.1.1. Design bytecode instruction format (opcode + operands)
  - [ ] 1.1.1.1. Choose instruction encoding (1-byte opcode, variable-length operands)
  - [ ] 1.1.1.2. Design operand encoding (immediate values, register indices, constant pool indices)
  - [ ] 1.1.1.3. Document instruction format specification
- [ ] 1.1.2. Define stack manipulation instructions
  - [ ] 1.1.2.1. `PUSH_CONST <index>` - Push constant from constant pool
  - [ ] 1.1.2.2. `PUSH_NIL` - Push nil value
  - [ ] 1.1.2.3. `PUSH_TRUE` / `PUSH_FALSE` - Push boolean values
  - [ ] 1.1.2.4. `POP` - Discard top of stack
  - [ ] 1.1.2.5. `DUP` - Duplicate top of stack
  - [ ] 1.1.2.6. `SWAP` - Swap top two stack values
- [ ] 1.1.3. Define arithmetic and comparison instructions
  - [ ] 1.1.3.1. `ADD`, `SUB`, `MUL`, `DIV`, `MOD` - Binary arithmetic
  - [ ] 1.1.3.2. `NEG`, `NOT` - Unary operations
  - [ ] 1.1.3.3. `EQ`, `NE`, `LT`, `LE`, `GT`, `GE` - Comparisons
  - [ ] 1.1.3.4. `AND`, `OR` - Logical operations
- [ ] 1.1.4. Define variable access instructions
  - [ ] 1.1.4.1. `LOAD_LOCAL <index>` - Load local variable by index
  - [ ] 1.1.4.2. `STORE_LOCAL <index>` - Store to local variable
  - [ ] 1.1.4.3. `LOAD_GLOBAL <name_index>` - Load global variable
  - [ ] 1.1.4.4. `STORE_GLOBAL <name_index>` - Store global variable
  - [ ] 1.1.4.5. `LOAD_IVAR <name_index>` - Load instance variable (@var)
  - [ ] 1.1.4.6. `STORE_IVAR <name_index>` - Store instance variable
  - [ ] 1.1.4.7. `LOAD_CVAR <name_index>` - Load class variable (@@var)
  - [ ] 1.1.4.8. `STORE_CVAR <name_index>` - Store class variable
- [ ] 1.1.5. Define control flow instructions
  - [ ] 1.1.5.1. `JUMP <offset>` - Unconditional jump
  - [ ] 1.1.5.2. `JUMP_IF_FALSE <offset>` - Conditional jump
  - [ ] 1.1.5.3. `JUMP_IF_TRUE <offset>` - Conditional jump
  - [ ] 1.1.5.4. `RETURN` - Return from function/method
  - [ ] 1.1.5.5. `BREAK` - Break from loop
  - [ ] 1.1.5.6. `CONTINUE` - Continue loop iteration
- [ ] 1.1.6. Define function/method call instructions
  - [ ] 1.1.6.1. `CALL <arg_count>` - Call function with N arguments
  - [ ] 1.1.6.2. `CALL_METHOD <name_index> <arg_count>` - Call method
  - [ ] 1.1.6.3. `CALL_SUPER <name_index> <arg_count>` - Call super method
  - [ ] 1.1.6.4. `LOAD_SELF` - Push self onto stack
- [ ] 1.1.7. Define collection instructions
  - [ ] 1.1.7.1. `BUILD_ARRAY <count>` - Build array from stack values
  - [ ] 1.1.7.2. `BUILD_DICT <count>` - Build dictionary from stack pairs
  - [ ] 1.1.7.3. `BUILD_RANGE <exclusive>` - Build range from two stack values
  - [ ] 1.1.7.4. `INDEX_GET` - Array/dict index access
  - [ ] 1.1.7.5. `INDEX_SET` - Array/dict index assignment
- [ ] 1.1.8. Define class and object instructions
  - [ ] 1.1.8.1. `DEFINE_CLASS <name_index> <has_super>` - Define class
  - [ ] 1.1.8.2. `DEFINE_METHOD <name_index>` - Define method in class
  - [ ] 1.1.8.3. `NEW_INSTANCE <class_index>` - Create instance
  - [ ] 1.1.8.4. `DEFINE_FUNCTION <name_index>` - Define top-level function
- [ ] 1.1.9. Define exception handling instructions
  - [ ] 1.1.9.1. `BEGIN_TRY <rescue_offset>` - Mark exception handler start
  - [ ] 1.1.9.2. `END_TRY` - Mark exception handler end
  - [ ] 1.1.9.3. `RESCUE <exception_type_index>` - Catch specific exception
  - [ ] 1.1.9.4. `RAISE` - Raise exception from stack
  - [ ] 1.1.9.5. `ENSURE_START` - Mark ensure block start
  - [ ] 1.1.9.6. `ENSURE_END` - Mark ensure block end
- [ ] 1.1.10. Define meta-programming instructions
  - [ ] 1.1.10.1. `BUILD_LAMBDA <code_index>` - Create lambda/block object
  - [ ] 1.1.10.2. `CAPTURE_VAR <name_index>` - Capture variable for closure
  - [ ] 1.1.10.3. `GET_UPVALUE <index>` - Load value from upvalue
  - [ ] 1.1.10.4. `SET_UPVALUE <index>` - Store value to upvalue
  - [ ] 1.1.10.5. `CLOSE_UPVALUE <stack_index>` - Close upvalue when scope ends
  - [ ] 1.1.10.6. `CALL_BLOCK` - Call block object
  - [ ] 1.1.10.7. `YIELD <arg_count>` - Yield to block with arguments
  - [ ] 1.1.10.8. `BLOCK_GIVEN` - Check if block was passed to current method
  - [ ] 1.1.10.9. `DEFINE_METHOD_RUNTIME <name_index>` - Runtime method definition
  - [ ] 1.1.10.10. `EVAL_STRING` - Pop string, parse to AST, compile to bytecode, execute
  - [ ] 1.1.10.11. `GET_METHOD_AST <method_name_index>` - Push method's AST onto stack
  - [ ] 1.1.10.12. `GET_CLASS_AST <class_name_index>` - Push class definition AST onto stack
  - [ ] 1.1.10.13. `SET_METHOD_AST <method_name_index>` - Pop AST, recompile and replace method
  - [ ] 1.1.10.14. `REMOVE_METHOD <class_name_index> <method_name_index>` - Remove method from class
- [ ] 1.1.11. Define pattern matching instructions
  - [ ] 1.1.11.1. `MATCH_START` - Begin pattern matching
  - [ ] 1.1.11.2. `MATCH_CASE <pattern_type> <jump_offset>` - Test pattern
  - [ ] 1.1.11.3. `MATCH_BIND <name_index>` - Bind matched value to variable
  - [ ] 1.1.11.4. `MATCH_END` - End pattern matching
- [ ] 1.1.12. Define debugging and introspection instructions
  - [ ] 1.1.12.1. `SET_LINE <line_number>` - Mark source line (for stack traces)
  - [ ] 1.1.12.2. `BREAKPOINT` - Debugger breakpoint
  - [ ] 1.1.12.3. `GET_AST` - Retrieve AST for meta-programming

### 1.2 Create Bytecode Data Structures

- [ ] 1.2.1. Implement `OpCode` enum with all instructions
  - [ ] 1.2.1.1. **TEST**: Write tests/bytecode_opcode_tests.rs for Display trait
  - [ ] 1.2.1.2. Create `src/bytecode/opcode.rs` and define OpCode enum
  - [ ] 1.2.1.3. Implement `Display` trait to make tests pass
  - [ ] 1.2.1.4. **TEST**: Write test for opcode enum completeness
  - [ ] 1.2.1.5. Verify all tests pass, run clippy & fmt
- [ ] 1.2.2. Implement `Instruction` struct
  - [ ] 1.2.2.1. **TEST**: Write test for instruction encoding to bytes
  - [ ] 1.2.2.2. Define Instruction { opcode: OpCode, operands: Vec&lt;u16&gt;, position: Position }
  - [ ] 1.2.2.3. Implement instruction encoding to make test pass
  - [ ] 1.2.2.4. **TEST**: Write test for instruction decoding from bytes
  - [ ] 1.2.2.5. Implement instruction decoding to make test pass
  - [ ] 1.2.2.6. **TEST**: Write round-trip encoding/decoding tests for all opcodes
  - [ ] 1.2.2.7. Fix any failures, verify 100% coverage for instruction.rs
- [ ] 1.2.3. Implement `ConstantPool` for storing literals
  - [ ] 1.2.3.1. **TEST**: Write tests/bytecode_constant_pool_tests.rs for adding/retrieving constants
  - [ ] 1.2.3.2. Create `src/bytecode/constant_pool.rs` and define ConstantPool struct
  - [ ] 1.2.3.3. Implement add_constant() and get_constant() to make tests pass
  - [ ] 1.2.3.4. **TEST**: Write test for string interning (deduplication)
  - [ ] 1.2.3.5. Implement string interning to make test pass
  - [ ] 1.2.3.6. **TEST**: Write tests for out-of-bounds access and capacity limits
  - [ ] 1.2.3.7. Implement error handling, verify 100% coverage
- [ ] 1.2.4. Implement `BytecodeChunk` container
  - [ ] 1.2.4.1. **TEST**: Write tests/bytecode_chunk_tests.rs for chunk operations
  - [ ] 1.2.4.2. Create `src/bytecode/chunk.rs` and define Chunk struct
  - [ ] 1.2.4.3. Implement add_instruction() and get_instruction() to make tests pass
  - [ ] 1.2.4.4. **TEST**: Write test for source line mapping
  - [ ] 1.2.4.5. Add source line mapping (instruction index → source line)
  - [ ] 1.2.4.6. **TEST**: Write test for large chunks (>65536 instructions)
  - [ ] 1.2.4.7. Verify all tests pass, 100% coverage for chunk.rs
- [ ] 1.2.5. Implement bytecode serialization/deserialization
  - [ ] 1.2.5.1. **TEST**: Write tests/bytecode_serialization_tests.rs for serialization
  - [ ] 1.2.5.2. Design bytecode file format (.mxc extension)
  - [ ] 1.2.5.3. Implement Chunk::serialize() to bytes to make test pass
  - [ ] 1.2.5.4. **TEST**: Write test for deserialization
  - [ ] 1.2.5.5. Implement Chunk::deserialize() from bytes
  - [ ] 1.2.5.6. **TEST**: Write round-trip serialization test
  - [ ] 1.2.5.7. **TEST**: Write test for version compatibility checking
  - [ ] 1.2.5.8. Add version header and magic number, verify all tests pass

### 1.3 Create Disassembler for Debugging

- [ ] 1.3.1. Implement bytecode disassembler
  - [ ] 1.3.1.1. **TEST**: Write tests/bytecode_disassembler_tests.rs for simple arithmetic
  - [ ] 1.3.1.2. Create `src/bytecode/disassembler.rs`
  - [ ] 1.3.1.3. Implement Disassembler::disassemble_chunk() basic version
  - [ ] 1.3.1.4. **TEST**: Write test expecting format "offset | opcode operands | source_line"
  - [ ] 1.3.1.5. Implement formatting to make test pass
  - [ ] 1.3.1.6. **TEST**: Write test for constant pool display
  - [ ] 1.3.1.7. Add constant pool contents display
  - [ ] 1.3.1.8. **TEST**: Write test for jump target annotations
  - [ ] 1.3.1.9. Add jump target annotations, verify all tests pass
- [ ] 1.3.2. Add CLI flag for disassembly output
  - [ ] 1.3.2.1. Add `--disassemble` flag to main.rs
  - [ ] 1.3.2.2. Compile and print bytecode instead of executing
  - [ ] 1.3.2.3. **TEST**: Manually test with example .mx files

---

## Phase 1.4: Runtime Compilation for Meta-Programming

**Context**: Meta-programming requires compiling and executing code at runtime. The compiler must be callable from the VM.

### 1.4.1 Add Runtime Compilation Instructions

- [ ] 1.4.1.1. Implement EVAL_STRING instruction
  - [ ] 1.4.1.1.1. **TEST**: Write test for eval("puts 'Hello'")
  - [ ] 1.4.1.1.2. Pop string from stack
  - [ ] 1.4.1.1.3. Parse string → AST using existing parser
  - [ ] 1.4.1.1.4. Compile AST → bytecode chunk using compiler
  - [ ] 1.4.1.1.5. Execute chunk in current scope context
  - [ ] 1.4.1.1.6. Push result onto stack
  - [ ] 1.4.1.1.7. Verify test passes

### 1.4.2 Design Compiler API for Runtime Use

- [ ] 1.4.2.1. Make compiler callable from VM
  - [ ] 1.4.2.1.1. **TEST**: Write test that calls compiler at runtime
  - [ ] 1.4.2.1.2. Add `Compiler::compile_in_context(ast, scope, globals)`
  - [ ] 1.4.2.1.3. Allow compiler to access current VM state (for variable resolution)
  - [ ] 1.4.2.1.4. **TEST**: Verify eval can access local variables in scope
  - [ ] 1.4.2.1.5. **TEST**: Verify eval can define new variables in scope
  - [ ] 1.4.2.1.6. Verify all tests pass

### 1.4.3 Handle define_method and Runtime Method Definition

- [ ] 1.4.3.1. Support define_method with blocks
  - [ ] 1.4.3.1.1. **TEST**: Write test for define_method(:foo) { |x| x * 2 }
  - [ ] 1.4.3.1.2. Block already has AST (from BUILD_LAMBDA)
  - [ ] 1.4.3.1.3. Compile block body to bytecode chunk
  - [ ] 1.4.3.1.4. Add method to class with compiled chunk
  - [ ] 1.4.3.1.5. **TEST**: Verify dynamically defined method works
  - [ ] 1.4.3.1.6. Verify test passes

- [ ] 1.4.3.2. Support class_eval and instance_eval
  - [ ] 1.4.3.2.1. **TEST**: Write test for MyClass.class_eval("def foo; 42; end")
  - [ ] 1.4.3.2.2. Parse string → AST
  - [ ] 1.4.3.2.3. Compile with appropriate self context (class or instance)
  - [ ] 1.4.3.2.4. Execute in that context
  - [ ] 1.4.3.2.5. Verify test passes

### 1.4.4 Full AST Preservation for Introspection

**Context**: Full meta-programming requires storing AST for all methods/classes to allow runtime introspection and modification.

- [ ] 1.4.4.1. Design AST storage in Method objects
  - [ ] 1.4.4.1.1. **DECISION**: Store original AST for every method (accept memory cost)
  - [ ] 1.4.4.1.2. Add `source_ast: Option<Rc<Vec<Statement>>>` to Method struct
  - [ ] 1.4.4.1.3. Compiler populates source_ast when creating methods
  - [ ] 1.4.4.1.4. AST is immutable (wrapped in Rc), clone before modification
  - [ ] 1.4.4.1.5. Document memory tradeoff: ~2x memory per method

- [ ] 1.4.4.2. Design AST storage in Class objects
  - [ ] 1.4.4.2.1. Add `class_definition_ast: Option<Rc<Statement>>` to Class struct
  - [ ] 1.4.4.2.2. Store entire class definition AST (includes all methods)
  - [ ] 1.4.4.2.3. Update when class is modified (method added/removed)
  - [ ] 1.4.4.2.4. **TEST**: Write test for class AST retrieval

- [ ] 1.4.4.3. Add AST retrieval instructions
  - [ ] 1.4.4.3.1. Add `GET_METHOD_AST <method_name>` instruction
  - [ ] 1.4.4.3.2. Add `GET_CLASS_AST <class_name>` instruction
  - [ ] 1.4.4.3.3. Both push AST as Object::AST variant onto stack
  - [ ] 1.4.4.3.4. Document AST object type in object model

- [ ] 1.4.4.4. Add AST modification support
  - [ ] 1.4.4.4.1. **TEST**: Write test for modifying method AST
  - [ ] 1.4.4.4.2. Add `SET_METHOD_AST <method_name>` instruction
  - [ ] 1.4.4.4.3. Pop AST from stack, recompile to bytecode
  - [ ] 1.4.4.4.4. Replace method's bytecode with new version
  - [ ] 1.4.4.4.5. Update method's stored AST to new version
  - [ ] 1.4.4.4.6. Invalidate method cache (trigger recompilation on next call)
  - [ ] 1.4.4.4.7. **TEST**: Verify modified method executes with new behavior

- [ ] 1.4.4.5. Support runtime method removal
  - [ ] 1.4.4.5.1. **TEST**: Write test for removing method from class
  - [ ] 1.4.4.5.2. Add `REMOVE_METHOD <class_name> <method_name>` instruction
  - [ ] 1.4.4.5.3. Remove method from class's method table
  - [ ] 1.4.4.5.4. Update class AST (remove method definition node)
  - [ ] 1.4.4.5.5. Invalidate method cache
  - [ ] 1.4.4.5.6. Verify method no longer callable

### 1.4.5 Document Meta-Programming Strategy

- [ ] 1.4.5.1. Write META_PROGRAMMING.md design doc
  - [ ] 1.4.5.1.1. **DECISION**: Full AST preservation for all methods/classes
  - [ ] 1.4.5.1.2. Document memory cost: ~2x per method (acceptable tradeoff)
  - [ ] 1.4.5.1.3. Explain AST modification → recompilation workflow
  - [ ] 1.4.5.1.4. Document Object::AST type and manipulation APIs
  - [ ] 1.4.5.1.5. Provide examples of AST introspection patterns

---

## Phase 1.5: Critical Infrastructure - GC, Closures, and Runtime Integration

### 1.5.1 Garbage Collection Integration

**Context**: The bytecode VM must properly integrate with the existing heap/GC system to ensure objects referenced by the stack, call frames, and constant pool are kept alive.

- [ ] 1.5.1.1. Design GC root set for bytecode VM
  - [ ] 1.5.1.1.1. **TEST**: Write test that GC doesn't collect objects on value stack
  - [ ] 1.5.1.1.2. Identify all GC roots: value stack, call frame locals, globals, constant pool
  - [ ] 1.5.1.1.3. Document root scanning strategy
  - [ ] 1.5.1.1.4. Verify test passes with manual GC trigger
- [ ] 1.5.1.2. Implement value stack GC integration
  - [ ] 1.5.1.2.1. **TEST**: Write test for stack-held objects surviving GC
  - [ ] 1.5.1.2.2. Ensure stack holds proper Rc<RefCell<Object>> references
  - [ ] 1.5.1.2.3. Implement stack scanning during GC mark phase
  - [ ] 1.5.1.2.4. Verify test passes with GC pressure
- [ ] 1.5.1.3. Implement call frame locals GC integration
  - [ ] 1.5.1.3.1. **TEST**: Write test for locals surviving across GC
  - [ ] 1.5.1.3.2. Store locals as GC-tracked references in CallFrame
  - [ ] 1.5.1.3.3. Scan all call frames during GC mark
  - [ ] 1.5.1.3.4. Verify test passes
- [ ] 1.5.1.4. Implement constant pool GC integration
  - [ ] 1.5.1.4.1. **TEST**: Write test for constants surviving GC
  - [ ] 1.5.1.4.2. Decide: are constants immortal or GC-tracked?
  - [ ] 1.5.1.4.3. If GC-tracked, scan constant pool during mark
  - [ ] 1.5.1.4.4. Verify test passes
- [ ] 1.5.1.5. Handle GC during bytecode execution
  - [ ] 1.5.1.5.1. **TEST**: Write test that triggers GC mid-execution
  - [ ] 1.5.1.5.2. Ensure VM state is always GC-safe (no dangling pointers)
  - [ ] 1.5.1.5.3. Test allocation-heavy bytecode doesn't crash
  - [ ] 1.5.1.5.4. Verify all tests pass

### 1.5.2 Closure and Upvalue Management

**Context**: Closures capture variables from outer scopes. When those scopes return, captured variables must "close over" - moving from stack to heap. This is one of the most complex parts of bytecode VMs.

- [ ] 1.5.2.1. Design upvalue system
  - [ ] 1.5.2.1.1. **TEST**: Write test for simple closure capturing local variable
  - [ ] 1.5.2.1.2. Design Upvalue type: Open (points to stack) vs Closed (owns value)
  - [ ] 1.5.2.1.3. Document upvalue lifecycle: creation → closing → access
  - [ ] 1.5.2.1.4. Create src/bytecode/upvalue.rs
- [ ] 1.5.2.2. Implement open upvalues
  - [ ] 1.5.2.2.1. **TEST**: Write test where closure accesses variable while frame is active
  - [ ] 1.5.2.2.2. Define OpenUpvalue { stack_index: usize, frame_id: usize }
  - [ ] 1.5.2.2.3. Implement upvalue lookup in active stack frames
  - [ ] 1.5.2.2.4. Verify test passes
- [ ] 1.5.2.3. Implement upvalue closing
  - [ ] 1.5.2.3.1. **TEST**: Write test where closure outlives creating frame
  - [ ] 1.5.2.3.2. On function return, close all upvalues pointing to returned frame
  - [ ] 1.5.2.3.3. Copy stack value to heap-allocated ClosedUpvalue
  - [ ] 1.5.2.3.4. Update all references to point to closed value
  - [ ] 1.5.2.3.5. Verify test passes (closure still works after return)
- [ ] 1.5.2.4. Implement CLOSE_UPVALUE instruction
  - [ ] 1.5.2.4.1. **TEST**: Write test for explicit upvalue closing at scope end
  - [ ] 1.5.2.4.2. Add CLOSE_UPVALUE <stack_index> instruction
  - [ ] 1.5.2.4.3. Close upvalues when local scope ends (not just function return)
  - [ ] 1.5.2.4.4. Verify test passes
- [ ] 1.5.2.5. Implement upvalue sharing
  - [ ] 1.5.2.5.1. **TEST**: Write test with multiple closures sharing same variable
  - [ ] 1.5.2.5.2. Track open upvalues per VM to deduplicate
  - [ ] 1.5.2.5.3. When creating upvalue, check if one already exists for that slot
  - [ ] 1.5.2.5.4. Share single upvalue across multiple closures
  - [ ] 1.5.2.5.5. **TEST**: Verify mutations through one closure visible in another
  - [ ] 1.5.2.5.6. Verify all tests pass
- [ ] 1.5.2.6. Update BUILD_LAMBDA to capture upvalues
  - [ ] 1.5.2.6.1. **TEST**: Write test for nested closures (closure in closure)
  - [ ] 1.5.2.6.2. Modify BUILD_LAMBDA to create upvalue list
  - [ ] 1.5.2.6.3. Store upvalues in Block/Lambda object
  - [ ] 1.5.2.6.4. Test nested closures work correctly
  - [ ] 1.5.2.6.5. Verify 100% coverage for upvalue.rs
- [ ] 1.5.2.7. Implement upvalue GC integration
  - [ ] 1.5.2.7.1. **TEST**: Write test that upvalues survive GC
  - [ ] 1.5.2.7.2. Scan open upvalues during GC mark
  - [ ] 1.5.2.7.3. Scan closed upvalue values during GC mark
  - [ ] 1.5.2.7.4. Verify test passes

### 1.5.3 Block Yield Support

**Context**: Metorex has blocks and yield (based on existing code). Bytecode must support yielding to blocks.

- [ ] 1.5.3.1. Design yield mechanism
  - [ ] 1.5.3.1.1. **TEST**: Write test for method that yields to block
  - [ ] 1.5.3.1.2. Add YIELD <arg_count> instruction
  - [ ] 1.5.3.1.3. Design block calling convention (stored where in frame?)
  - [ ] 1.5.3.1.4. Document yield control flow
- [ ] 1.5.3.2. Implement YIELD instruction handler
  - [ ] 1.5.3.2.1. **TEST**: Write test for yield with arguments
  - [ ] 1.5.3.2.2. Pop N arguments from stack
  - [ ] 1.5.3.2.3. Get current block from call frame
  - [ ] 1.5.3.2.4. Call block with arguments
  - [ ] 1.5.3.2.5. Push block's return value
  - [ ] 1.5.3.2.6. Verify test passes
- [ ] 1.5.3.3. Implement BLOCK_GIVEN instruction
  - [ ] 1.5.3.3.1. **TEST**: Write test for block_given? conditional
  - [ ] 1.5.3.3.2. Add BLOCK_GIVEN instruction (pushes bool)
  - [ ] 1.5.3.3.3. Check if current call frame has a block
  - [ ] 1.5.3.3.4. Verify test passes
- [ ] 1.5.3.4. Implement implicit block passing
  - [ ] 1.5.3.4.1. **TEST**: Write test for method call with trailing block
  - [ ] 1.5.3.4.2. Modify CALL_METHOD to accept optional block
  - [ ] 1.5.3.4.3. Store block in new call frame
  - [ ] 1.5.3.4.4. Verify block accessible via yield
- [ ] 1.5.3.5. Handle block return semantics
  - [ ] 1.5.3.5.1. **TEST**: Write test that return in block returns from method
  - [ ] 1.5.3.5.2. Distinguish block return from method return
  - [ ] 1.5.3.5.3. Implement non-local return (return from enclosing method)
  - [ ] 1.5.3.5.4. Verify Ruby-like block return behavior

### 1.5.4 Method Resolution and Caching

**Context**: Dynamic method lookup is expensive. Need efficient resolution with support for method_missing.

- [ ] 1.5.4.1. Implement basic method lookup
  - [ ] 1.5.4.1.1. **TEST**: Write test for method lookup in class hierarchy
  - [ ] 1.5.4.1.2. Search instance methods in class
  - [ ] 1.5.4.1.3. Walk superclass chain if not found
  - [ ] 1.5.4.1.4. Verify test passes
- [ ] 1.5.4.2. Implement method_missing support
  - [ ] 1.5.4.2.1. **TEST**: Write test that method_missing gets called
  - [ ] 1.5.4.2.2. If method not found, look for method_missing
  - [ ] 1.5.4.2.3. Call method_missing with original method name + args
  - [ ] 1.5.4.2.4. Verify test passes
- [ ] 1.5.4.3. Implement inline caching (basic)
  - [ ] 1.5.4.3.1. **TEST**: Write benchmark for repeated method calls
  - [ ] 1.5.4.3.2. Add cache slot to CALL_METHOD instruction
  - [ ] 1.5.4.3.3. Cache last lookup result (class → method mapping)
  - [ ] 1.5.4.3.4. Check cache before full lookup
  - [ ] 1.5.4.3.5. Measure performance improvement (should be significant)
- [ ] 1.5.4.4. Implement cache invalidation
  - [ ] 1.5.4.4.1. **TEST**: Write test that redefines method, calls again
  - [ ] 1.5.4.4.2. Add global method cache version counter
  - [ ] 1.5.4.4.3. Increment version on any method definition
  - [ ] 1.5.4.4.4. Invalidate cached entries if version changed
  - [ ] 1.5.4.4.5. Verify test shows new behavior
- [ ] 1.5.4.5. Handle visibility (private/protected/public)
  - [ ] 1.5.4.5.1. **TEST**: Write test for private method enforcement
  - [ ] 1.5.4.5.2. Check method visibility during lookup
  - [ ] 1.5.4.5.3. Raise error if calling private method from outside
  - [ ] 1.5.4.5.4. Verify test passes

### 1.5.5 Operator Overloading Support

**Context**: Metorex allows custom +, -, [], etc. Bytecode must call these methods.

- [ ] 1.5.5.1. Map operators to method calls
  - [ ] 1.5.5.1.1. **TEST**: Write test for custom + operator
  - [ ] 1.5.5.1.2. ADD instruction: try native add first, fall back to calling + method
  - [ ] 1.5.5.1.3. Implement for all binary operators
  - [ ] 1.5.5.1.4. Verify test passes
- [ ] 1.5.5.2. Implement index operator overloading
  - [ ] 1.5.5.2.1. **TEST**: Write test for custom [] method
  - [ ] 1.5.5.2.2. INDEX_GET: call [] method if defined
  - [ ] 1.5.5.2.3. INDEX_SET: call []= method if defined
  - [ ] 1.5.5.2.4. Verify test passes
- [ ] 1.5.5.3. Optimize built-in type operators
  - [ ] 1.5.5.3.1. **TEST**: Benchmark int + int performance
  - [ ] 1.5.5.3.2. Fast path for Int, Float, String native operations
  - [ ] 1.5.5.3.3. Only call method for custom classes
  - [ ] 1.5.5.3.4. Verify performance is good

---

## Phase 2: Compiler - AST to Bytecode Translation

### 2.1 Implement Basic Compiler Infrastructure

- [ ] 2.1.1. Create compiler module structure
  - [ ] 2.1.1.1. **TEST**: Write tests/bytecode_compiler_tests.rs - test compiling empty program
  - [ ] 2.1.1.2. Create `src/bytecode/compiler.rs`
  - [ ] 2.1.1.3. Define Compiler struct with current_chunk, scopes, constant_pool
  - [ ] 2.1.1.4. Implement Compiler::new() and compile() skeleton to make test pass
  - [ ] 2.1.1.5. Verify test passes
- [ ] 2.1.2. Implement local variable scope tracking
  - [ ] 2.1.2.1. **TEST**: Write test for single-scope variable resolution
  - [ ] 2.1.2.2. Define Scope struct { locals: HashMap&lt;String, u16&gt;, depth: usize }
  - [ ] 2.1.2.3. Implement push_scope() / pop_scope() / resolve_local()
  - [ ] 2.1.2.4. **TEST**: Write test for nested scope variable shadowing
  - [ ] 2.1.2.5. Fix implementation to make test pass
  - [ ] 2.1.2.6. **TEST**: Write test for variable declaration order (indices)
  - [ ] 2.1.2.7. Verify all scope tests pass, 100% coverage
- [ ] 2.1.3. Implement jump patching mechanism
  - [ ] 2.1.3.1. **TEST**: Write test for forward jump patching
  - [ ] 2.1.3.2. Create JumpPatch struct and emit_jump() skeleton
  - [ ] 2.1.3.3. Implement patch_jump() to make test pass
  - [ ] 2.1.3.4. **TEST**: Write test for backward jump
  - [ ] 2.1.3.5. Implement get_current_position(), verify all jump tests pass

### 2.2 Compile Expressions

- [ ] 2.2.1. Compile literal expressions
  - [ ] 2.2.1.1. **TEST**: Write test for compiling integer literal (5 → PUSH_CONST 0)
  - [ ] 2.2.1.2. Implement IntLiteral → PUSH_CONST (add to constant pool)
  - [ ] 2.2.1.3. **TEST**: Write test for float literal
  - [ ] 2.2.1.4. Implement FloatLiteral → PUSH_CONST
  - [ ] 2.2.1.5. **TEST**: Write test for string literal
  - [ ] 2.2.1.6. Implement StringLiteral → PUSH_CONST
  - [ ] 2.2.1.7. **TEST**: Write test for boolean literals (true → PUSH_TRUE)
  - [ ] 2.2.1.8. Implement BoolLiteral → PUSH_TRUE / PUSH_FALSE
  - [ ] 2.2.1.9. **TEST**: Write test for nil literal
  - [ ] 2.2.1.10. Implement NilLiteral → PUSH_NIL
  - [ ] 2.2.1.11. Verify all literal tests pass
- [ ] 2.2.2. Compile variable access
  - [ ] 2.2.2.1. Identifier → LOAD_LOCAL or LOAD_GLOBAL
  - [ ] 2.2.2.2. InstanceVariable → LOAD_IVAR
  - [ ] 2.2.2.3. ClassVariable → LOAD_CVAR
  - [ ] 2.2.2.4. SelfExpr → LOAD_SELF
  - [ ] 2.2.2.5. Write tests for variable resolution
- [ ] 2.2.3. Compile unary operations
  - [ ] 2.2.3.1. Compile operand, then emit NEG / NOT
  - [ ] 2.2.3.2. Handle operator precedence
  - [ ] 2.2.3.3. Write tests for unary operators
- [ ] 2.2.4. Compile binary operations
  - [ ] 2.2.4.1. Compile left operand, compile right operand, emit operation
  - [ ] 2.2.4.2. ADD, SUB, MUL, DIV, MOD
  - [ ] 2.2.4.3. EQ, NE, LT, LE, GT, GE
  - [ ] 2.2.4.4. AND, OR (short-circuit evaluation with jumps)
  - [ ] 2.2.4.5. Write tests for each operator
- [ ] 2.2.5. Compile collection literals
  - [ ] 2.2.5.1. Array → compile elements, emit BUILD_ARRAY &lt;count&gt;
  - [ ] 2.2.5.2. Dictionary → compile key-value pairs, emit BUILD_DICT &lt;count&gt;
  - [ ] 2.2.5.3. Range → compile start/end, emit BUILD_RANGE &lt;exclusive&gt;
  - [ ] 2.2.5.4. Write tests for collection compilation
- [ ] 2.2.6. Compile index operations
  - [ ] 2.2.6.1. Index access → compile collection, compile index, emit INDEX_GET
  - [ ] 2.2.6.2. Index assignment → compile collection, compile index, compile value, emit INDEX_SET
  - [ ] 2.2.6.3. Write tests for indexing
- [ ] 2.2.7. Compile method calls
  - [ ] 2.2.7.1. MethodCall → compile receiver, compile args, emit CALL_METHOD &lt;name&gt; &lt;argc&gt;
  - [ ] 2.2.7.2. Handle trailing blocks (compile block, pass as last arg)
  - [ ] 2.2.7.3. Write tests for method calls with/without blocks
- [ ] 2.2.8. Compile function calls
  - [ ] 2.2.8.1. Call → compile callee, compile args, emit CALL &lt;argc&gt;
  - [ ] 2.2.8.2. Write tests for function calls
- [ ] 2.2.9. Compile super calls
  - [ ] 2.2.9.1. Super → compile args, emit CALL_SUPER &lt;method_name&gt; &lt;argc&gt;
  - [ ] 2.2.9.2. Write tests for super invocation
- [ ] 2.2.10. Compile string interpolation
  - [ ] 2.2.10.1. InterpolatedString → compile parts, emit string concatenation
  - [ ] 2.2.10.2. Optimize: if all parts are constants, fold into single string
  - [ ] 2.2.10.3. Write tests for interpolation
- [ ] 2.2.11. Compile lambda/block expressions
  - [ ] 2.2.11.1. Lambda → compile body into separate chunk, emit BUILD_LAMBDA &lt;chunk_index&gt;
  - [ ] 2.2.11.2. Implement closure capture (emit CAPTURE_VAR for each captured variable)
  - [ ] 2.2.11.3. Store lambda chunks in constant pool
  - [ ] 2.2.11.4. Write tests for lambdas and closures

### 2.3 Compile Statements

- [ ] 2.3.1. Compile expression statements
  - [ ] 2.3.1.1. Compile expression, emit POP (discard result if not used)
  - [ ] 2.3.1.2. Handle auto-call for method references
  - [ ] 2.3.1.3. Write tests for expression statements
- [ ] 2.3.2. Compile assignments
  - [ ] 2.3.2.1. Variable assignment → compile value, emit STORE_LOCAL/STORE_GLOBAL
  - [ ] 2.3.2.2. Instance variable → emit STORE_IVAR
  - [ ] 2.3.2.3. Class variable → emit STORE_CVAR
  - [ ] 2.3.2.4. Index assignment → compile target, index, value, emit INDEX_SET
  - [ ] 2.3.2.5. Write tests for each assignment type
- [ ] 2.3.3. Compile return statements
  - [ ] 2.3.3.1. Return → compile value (or push nil), emit RETURN
  - [ ] 2.3.3.2. Write tests for return statements
- [ ] 2.3.4. Compile break/continue
  - [ ] 2.3.4.1. Break → emit BREAK (requires loop context tracking)
  - [ ] 2.3.4.2. Continue → emit CONTINUE
  - [ ] 2.3.4.3. Track loop start/end for jump resolution
  - [ ] 2.3.4.4. Write tests for break/continue
- [ ] 2.3.5. Compile if/elsif/else statements
  - [ ] 2.3.5.1. Compile condition, emit JUMP_IF_FALSE to else/elsif
  - [ ] 2.3.5.2. Compile then-branch, emit JUMP to end
  - [ ] 2.3.5.3. Patch else/elsif jump targets
  - [ ] 2.3.5.4. Handle elsif chain with multiple jumps
  - [ ] 2.3.5.5. Write tests for if statements
- [ ] 2.3.6. Compile unless statements
  - [ ] 2.3.6.1. Compile condition, emit JUMP_IF_TRUE to else
  - [ ] 2.3.6.2. Compile then-branch
  - [ ] 2.3.6.3. Write tests for unless statements
- [ ] 2.3.7. Compile while loops
  - [ ] 2.3.7.1. Mark loop start position
  - [ ] 2.3.7.2. Compile condition, emit JUMP_IF_FALSE to end
  - [ ] 2.3.7.3. Compile body with break/continue support
  - [ ] 2.3.7.4. Emit JUMP back to loop start
  - [ ] 2.3.7.5. Patch break/continue jumps
  - [ ] 2.3.7.6. Write tests for while loops
- [ ] 2.3.8. Compile for loops
  - [ ] 2.3.8.1. Compile iterable, emit iterator setup
  - [ ] 2.3.8.2. Mark loop start, emit iterator check
  - [ ] 2.3.8.3. Compile body with loop variable binding
  - [ ] 2.3.8.4. Emit iterator advance, JUMP to start
  - [ ] 2.3.8.5. Write tests for for loops
- [ ] 2.3.9. Compile blocks
  - [ ] 2.3.9.1. Push scope, compile statements, pop scope
  - [ ] 2.3.9.2. Write tests for block scoping
- [ ] 2.3.10. Compile class definitions
  - [ ] 2.3.10.1. Emit DEFINE_CLASS &lt;name&gt; &lt;has_super&gt;
  - [ ] 2.3.10.2. Compile class body (methods, attr_accessor, class vars)
  - [ ] 2.3.10.3. Emit DEFINE_METHOD for each method
  - [ ] 2.3.10.4. Handle initialize method specially
  - [ ] 2.3.10.5. Write tests for class compilation
- [ ] 2.3.11. Compile function definitions
  - [ ] 2.3.11.1. Compile function body into separate chunk
  - [ ] 2.3.11.2. Emit DEFINE_FUNCTION &lt;name&gt; &lt;chunk_index&gt;
  - [ ] 2.3.11.3. Write tests for function definitions

### 2.4 Compile Advanced Features

- [ ] 2.4.1. Compile exception handling (begin/rescue/ensure)
  - [ ] 2.4.1.1. Emit BEGIN_TRY &lt;rescue_offset&gt; before try block
  - [ ] 2.4.1.2. Compile body, emit END_TRY
  - [ ] 2.4.1.3. For each rescue clause, emit RESCUE &lt;exception_type&gt;
  - [ ] 2.4.1.4. Compile rescue body, emit JUMP to after-ensure
  - [ ] 2.4.1.5. Emit ENSURE_START, compile ensure block, emit ENSURE_END
  - [ ] 2.4.1.6. Write tests for exception handling compilation
- [ ] 2.4.2. Compile raise statements
  - [ ] 2.4.2.1. Compile exception value, emit RAISE
  - [ ] 2.4.2.2. Write tests for raise compilation
- [ ] 2.4.3. Compile pattern matching (case/when)
  - [ ] 2.4.3.1. Emit MATCH_START
  - [ ] 2.4.3.2. Compile expression to match
  - [ ] 2.4.3.3. For each case, emit MATCH_CASE &lt;pattern&gt; &lt;jump_if_no_match&gt;
  - [ ] 2.4.3.4. Compile bindings with MATCH_BIND
  - [ ] 2.4.3.5. Compile case body, emit JUMP to end
  - [ ] 2.4.3.6. Emit MATCH_END
  - [ ] 2.4.3.7. Write tests for pattern matching compilation

### 2.5 Optimization Pass (Optional, can be deferred)

- [ ] 2.5.1. Implement constant folding
  - [ ] 2.5.1.1. Detect constant expressions (2 + 3)
  - [ ] 2.5.1.2. Evaluate at compile time, emit single PUSH_CONST
  - [ ] 2.5.1.3. Write tests for constant folding
- [ ] 2.5.2. Implement dead code elimination
  - [ ] 2.5.2.1. Detect unreachable code after return/break
  - [ ] 2.5.2.2. Don't emit bytecode for unreachable statements
  - [ ] 2.5.2.3. Write tests for dead code elimination
- [ ] 2.5.3. Implement peephole optimization
  - [ ] 2.5.3.1. Detect PUSH_CONST + POP sequences, eliminate both
  - [ ] 2.5.3.2. Optimize JUMP to JUMP (jump chain collapsing)
  - [ ] 2.5.3.3. Write tests for peephole patterns

---

## Phase 3: Stack-Based Virtual Machine

### 3.1 Implement VM Core Infrastructure

- [ ] 3.1.1. Create bytecode VM structure
  - [ ] 3.1.1.1. Create `src/bytecode/vm.rs`
  - [ ] 3.1.1.2. Define BytecodeVM struct { stack: Vec&lt;Object&gt;, call_frames: Vec&lt;CallFrame&gt;, globals: HashMap, heap: Heap }
  - [ ] 3.1.1.3. Implement BytecodeVM::new()
  - [ ] 3.1.1.4. Write tests for VM initialization
- [ ] 3.1.2. Implement value stack operations
  - [ ] 3.1.2.1. Implement push(value: Object)
  - [ ] 3.1.2.2. Implement pop() → Object
  - [ ] 3.1.2.3. Implement peek(offset: usize) → &Object
  - [ ] 3.1.2.4. Add stack overflow/underflow checks
  - [ ] 3.1.2.5. Write tests for stack operations
- [ ] 3.1.3. Implement call frame management
  - [ ] 3.1.3.1. Define BytecodeCallFrame { chunk: Rc&lt;Chunk&gt;, ip: usize, stack_base: usize, locals: Vec&lt;Object&gt; }
  - [ ] 3.1.3.2. Implement push_frame(frame: BytecodeCallFrame)
  - [ ] 3.1.3.3. Implement pop_frame() → BytecodeCallFrame
  - [ ] 3.1.3.4. Implement current_frame() → &BytecodeCallFrame
  - [ ] 3.1.3.5. Add call stack depth limit
  - [ ] 3.1.3.6. Write tests for call frame management
- [ ] 3.1.4. Implement instruction pointer (IP) management
  - [ ] 3.1.4.1. Implement read_byte() → u8 (from current chunk at IP)
  - [ ] 3.1.4.2. Implement read_short() → u16 (two bytes)
  - [ ] 3.1.4.3. Implement advance_ip(offset: isize)
  - [ ] 3.1.4.4. Write tests for IP operations
- [ ] 3.1.5. Implement main execution loop
  - [ ] 3.1.5.1. Implement run(chunk: Chunk) → Result&lt;Object&gt;
  - [ ] 3.1.5.2. Fetch-decode-execute loop structure
  - [ ] 3.1.5.3. Add instruction tracing for debugging (optional flag)
  - [ ] 3.1.5.4. Write tests for execution loop

### 3.2 Implement Instruction Handlers

- [ ] 3.2.1. Implement stack manipulation handlers
  - [ ] 3.2.1.1. PUSH_CONST → read index, push from constant pool
  - [ ] 3.2.1.2. PUSH_NIL → push Object::Nil
  - [ ] 3.2.1.3. PUSH_TRUE / PUSH_FALSE → push booleans
  - [ ] 3.2.1.4. POP → discard top of stack
  - [ ] 3.2.1.5. DUP → duplicate top value
  - [ ] 3.2.1.6. SWAP → swap top two values
  - [ ] 3.2.1.7. Write tests for each instruction
- [ ] 3.2.2. Implement arithmetic handlers
  - [ ] 3.2.2.1. ADD → pop two values, push sum
  - [ ] 3.2.2.2. SUB, MUL, DIV, MOD → implement binary operations
  - [ ] 3.2.2.3. NEG, NOT → implement unary operations
  - [ ] 3.2.2.4. Handle type checking and coercion
  - [ ] 3.2.2.5. Write tests for each operation
- [ ] 3.2.3. Implement comparison handlers
  - [ ] 3.2.3.1. EQ, NE, LT, LE, GT, GE → pop two, compare, push bool
  - [ ] 3.2.3.2. Handle different object types (int, float, string)
  - [ ] 3.2.3.3. Write tests for comparisons
- [ ] 3.2.4. Implement variable access handlers
  - [ ] 3.2.4.1. LOAD_LOCAL → read index, push from call frame locals
  - [ ] 3.2.4.2. STORE_LOCAL → read index, pop value, store in locals
  - [ ] 3.2.4.3. LOAD_GLOBAL → read name, lookup in globals, push
  - [ ] 3.2.4.4. STORE_GLOBAL → read name, pop value, store in globals
  - [ ] 3.2.4.5. LOAD_IVAR → get self, read instance variable, push
  - [ ] 3.2.4.6. STORE_IVAR → get self, pop value, set instance variable
  - [ ] 3.2.4.7. LOAD_CVAR → get class, read class variable, push
  - [ ] 3.2.4.8. STORE_CVAR → get class, pop value, set class variable
  - [ ] 3.2.4.9. LOAD_SELF → push self from current frame
  - [ ] 3.2.4.10. Write tests for each variable operation
- [ ] 3.2.5. Implement control flow handlers
  - [ ] 3.2.5.1. JUMP → read offset, advance IP
  - [ ] 3.2.5.2. JUMP_IF_FALSE → pop value, jump if falsey
  - [ ] 3.2.5.3. JUMP_IF_TRUE → pop value, jump if truthy
  - [ ] 3.2.5.4. RETURN → pop value, pop call frame, return value
  - [ ] 3.2.5.5. BREAK → unwind to loop end (requires loop tracking)
  - [ ] 3.2.5.6. CONTINUE → jump to loop start
  - [ ] 3.2.5.7. Write tests for control flow
- [ ] 3.2.6. Implement collection handlers
  - [ ] 3.2.6.1. BUILD_ARRAY → pop N values, build array, push
  - [ ] 3.2.6.2. BUILD_DICT → pop N*2 values (key-value pairs), build dict, push
  - [ ] 3.2.6.3. BUILD_RANGE → pop start/end, build range, push
  - [ ] 3.2.6.4. INDEX_GET → pop index, pop collection, get element, push
  - [ ] 3.2.6.5. INDEX_SET → pop value, pop index, pop collection, set element
  - [ ] 3.2.6.6. Write tests for collection operations

### 3.3 Implement Function and Method Calls

- [ ] 3.3.1. Implement CALL instruction handler
  - [ ] 3.3.1.1. Pop N arguments from stack
  - [ ] 3.3.1.2. Pop callable object
  - [ ] 3.3.1.3. Validate callable type (Function, Block, etc.)
  - [ ] 3.3.1.4. Create new call frame with callable's chunk
  - [ ] 3.3.1.5. Initialize locals with arguments
  - [ ] 3.3.1.6. Push call frame and continue execution
  - [ ] 3.3.1.7. Write tests for function calls
- [ ] 3.3.2. Implement CALL_METHOD instruction handler
  - [ ] 3.3.2.1. Read method name from constant pool
  - [ ] 3.3.2.2. Pop N arguments
  - [ ] 3.3.2.3. Pop receiver object
  - [ ] 3.3.2.4. Look up method in receiver's class
  - [ ] 3.3.2.5. Handle built-in methods (native functions)
  - [ ] 3.3.2.6. Handle user-defined methods (bytecode)
  - [ ] 3.3.2.7. Create call frame with self bound
  - [ ] 3.3.2.8. Write tests for method calls
- [ ] 3.3.3. Implement CALL_SUPER instruction handler
  - [ ] 3.3.3.1. Read method name from constant pool
  - [ ] 3.3.3.2. Get current class from call frame
  - [ ] 3.3.3.3. Look up method in superclass
  - [ ] 3.3.3.4. Create call frame with same self
  - [ ] 3.3.3.5. Write tests for super calls
- [ ] 3.3.4. Implement native method bridge
  - [ ] 3.3.4.1. Create NativeMethod wrapper for Rust functions
  - [ ] 3.3.4.2. Extract arguments from stack
  - [ ] 3.3.4.3. Call Rust function, convert result to Object
  - [ ] 3.3.4.4. Push result onto stack
  - [ ] 3.3.4.5. Write tests for native methods

### 3.4 Implement Class and Object Support

- [ ] 3.4.1. Implement DEFINE_CLASS instruction handler
  - [ ] 3.4.1.1. Read class name from constant pool
  - [ ] 3.4.1.2. Check for superclass flag
  - [ ] 3.4.1.3. Pop superclass if present
  - [ ] 3.4.1.4. Create Class object
  - [ ] 3.4.1.5. Store in globals
  - [ ] 3.4.1.6. Push class onto stack for method definitions
  - [ ] 3.4.1.7. Write tests for class definition
- [ ] 3.4.2. Implement DEFINE_METHOD instruction handler
  - [ ] 3.4.2.1. Read method name from constant pool
  - [ ] 3.4.2.2. Pop method body (chunk or block)
  - [ ] 3.4.2.3. Peek class from stack
  - [ ] 3.4.2.4. Add method to class
  - [ ] 3.4.2.5. Write tests for method definition
- [ ] 3.4.3. Implement NEW_INSTANCE instruction handler
  - [ ] 3.4.3.1. Read class index
  - [ ] 3.4.3.2. Get class from constant pool
  - [ ] 3.4.3.3. Create instance object
  - [ ] 3.4.3.4. Call initialize if present
  - [ ] 3.4.3.5. Push instance onto stack
  - [ ] 3.4.3.6. Write tests for instance creation
- [ ] 3.4.4. Implement DEFINE_FUNCTION instruction handler
  - [ ] 3.4.4.1. Read function name
  - [ ] 3.4.4.2. Pop function body chunk
  - [ ] 3.4.4.3. Create Function object
  - [ ] 3.4.4.4. Store in globals
  - [ ] 3.4.4.5. Write tests for function definition

### 3.5 Implement Exception Handling

- [ ] 3.5.1. Implement exception handler stack
  - [ ] 3.5.1.1. Define ExceptionHandler { rescue_ip: usize, ensure_ip: Option&lt;usize&gt;, stack_depth: usize }
  - [ ] 3.5.1.2. Add exception_handlers: Vec&lt;ExceptionHandler&gt; to VM
  - [ ] 3.5.1.3. Implement push_exception_handler()
  - [ ] 3.5.1.4. Implement pop_exception_handler()
  - [ ] 3.5.1.5. Write tests for handler stack
- [ ] 3.5.2. Implement BEGIN_TRY instruction handler
  - [ ] 3.5.2.1. Read rescue offset from instruction
  - [ ] 3.5.2.2. Push exception handler onto stack
  - [ ] 3.5.2.3. Continue execution
  - [ ] 3.5.2.4. Write tests for begin blocks
- [ ] 3.5.3. Implement END_TRY instruction handler
  - [ ] 3.5.3.1. Pop exception handler
  - [ ] 3.5.3.2. If no exception, continue
  - [ ] 3.5.3.3. Write tests for successful try blocks
- [ ] 3.5.4. Implement RESCUE instruction handler
  - [ ] 3.5.4.1. Read exception type from constant pool
  - [ ] 3.5.4.2. Peek exception from stack (set by unwinding)
  - [ ] 3.5.4.3. Check if exception matches type
  - [ ] 3.5.4.4. If match, clear exception and execute rescue body
  - [ ] 3.5.4.5. If no match, jump to next rescue or re-raise
  - [ ] 3.5.4.6. Write tests for rescue clauses
- [ ] 3.5.5. Implement RAISE instruction handler
  - [ ] 3.5.5.1. Pop exception object from stack
  - [ ] 3.5.5.2. Begin stack unwinding
  - [ ] 3.5.5.3. Search for matching exception handler
  - [ ] 3.5.5.4. If found, restore stack and jump to rescue
  - [ ] 3.5.5.5. If not found, propagate to caller
  - [ ] 3.5.5.6. Write tests for raise
- [ ] 3.5.6. Implement ENSURE_START / ENSURE_END handlers
  - [ ] 3.5.6.1. Mark ensure block boundaries
  - [ ] 3.5.6.2. Execute ensure block on normal exit or exception
  - [ ] 3.5.6.3. Re-raise exception after ensure if needed
  - [ ] 3.5.6.4. Write tests for ensure blocks
- [ ] 3.5.7. Implement stack unwinding
  - [ ] 3.5.7.1. Save current exception
  - [ ] 3.5.7.2. Walk call frames backwards
  - [ ] 3.5.7.3. Execute ensure blocks during unwinding
  - [ ] 3.5.7.4. Find matching rescue clause
  - [ ] 3.5.7.5. Restore stack to handler depth
  - [ ] 3.5.7.6. Write tests for stack unwinding

### 3.6 Implement Meta-Programming Support

- [ ] 3.6.1. Implement EVAL_STRING instruction handler
  - [ ] 3.6.1.1. **TEST**: Write test for eval("x = 5; x * 2")
  - [ ] 3.6.1.2. Pop string value from stack
  - [ ] 3.6.1.3. Create parser instance with string source
  - [ ] 3.6.1.4. Parse string → AST
  - [ ] 3.6.1.5. Create compiler instance with current VM context
  - [ ] 3.6.1.6. Compile AST → bytecode chunk
  - [ ] 3.6.1.7. Execute chunk in current scope (access to locals/globals)
  - [ ] 3.6.1.8. Push return value onto stack
  - [ ] 3.6.1.9. **TEST**: Verify eval can access existing variables
  - [ ] 3.6.1.10. **TEST**: Verify eval can define new variables
  - [ ] 3.6.1.11. Write tests for eval with syntax errors
  - [ ] 3.6.1.12. Verify all tests pass

- [ ] 3.6.2. Implement BUILD_LAMBDA instruction handler
  - [ ] 3.6.2.1. Read chunk index from constant pool
  - [ ] 3.6.2.2. Get lambda chunk
  - [ ] 3.6.2.3. Capture variables (read capture list)
  - [ ] 3.6.2.4. Create Block object with chunk and captures
  - [ ] 3.6.2.5. Push block onto stack
  - [ ] 3.6.2.6. Write tests for lambda creation
- [ ] 3.6.3. Implement CAPTURE_VAR instruction handler
  - [ ] 3.6.3.1. Read variable name from constant pool
  - [ ] 3.6.3.2. Look up variable in current scope
  - [ ] 3.6.3.3. Store reference in closure's capture map
  - [ ] 3.6.3.4. Write tests for variable capture
- [ ] 3.6.4. Implement CALL_BLOCK instruction handler
  - [ ] 3.6.4.1. Pop block object
  - [ ] 3.6.4.2. Pop arguments
  - [ ] 3.6.4.3. Create call frame with block's chunk
  - [ ] 3.6.4.4. Bind captured variables in new frame
  - [ ] 3.6.4.5. Execute block
  - [ ] 3.6.4.6. Write tests for block calls
- [ ] 3.6.5. Implement DEFINE_METHOD_RUNTIME instruction handler
  - [ ] 3.6.5.1. Read method name
  - [ ] 3.6.5.2. Pop block from stack (method body)
  - [ ] 3.6.5.3. Get target class (from self or explicit)
  - [ ] 3.6.5.4. Add method to class at runtime
  - [ ] 3.6.5.5. Write tests for define_method

- [ ] 3.6.6. Implement GET_METHOD_AST instruction handler
  - [ ] 3.6.6.1. **TEST**: Write test for method.get_ast()
  - [ ] 3.6.6.2. Read method name from constant pool
  - [ ] 3.6.6.3. Look up method in current class/global scope
  - [ ] 3.6.6.4. Retrieve source_ast from Method object
  - [ ] 3.6.6.5. Wrap AST in Object::AST variant
  - [ ] 3.6.6.6. Push onto stack
  - [ ] 3.6.6.7. **TEST**: Verify AST structure matches original definition
  - [ ] 3.6.6.8. Handle case where AST is None (error or return nil)

- [ ] 3.6.7. Implement GET_CLASS_AST instruction handler
  - [ ] 3.6.7.1. **TEST**: Write test for MyClass.get_ast()
  - [ ] 3.6.7.2. Read class name from constant pool
  - [ ] 3.6.7.3. Look up class in globals
  - [ ] 3.6.7.4. Retrieve class_definition_ast from Class object
  - [ ] 3.6.7.5. Wrap in Object::AST, push onto stack
  - [ ] 3.6.7.6. Verify test passes

- [ ] 3.6.8. Implement SET_METHOD_AST instruction handler
  - [ ] 3.6.8.1. **TEST**: Write test for modifying method via AST
  - [ ] 3.6.8.2. Read method name from constant pool
  - [ ] 3.6.8.3. Pop Object::AST from stack
  - [ ] 3.6.8.4. Unwrap to get Vec<Statement>
  - [ ] 3.6.8.5. Invoke compiler to generate new bytecode chunk
  - [ ] 3.6.8.6. Look up existing method in class
  - [ ] 3.6.8.7. Replace method's bytecode chunk with new version
  - [ ] 3.6.8.8. Update method's source_ast to new AST
  - [ ] 3.6.8.9. Invalidate method cache (increment global version)
  - [ ] 3.6.8.10. **TEST**: Call modified method, verify new behavior
  - [ ] 3.6.8.11. Verify all tests pass

- [ ] 3.6.9. Implement REMOVE_METHOD instruction handler
  - [ ] 3.6.9.1. **TEST**: Write test for removing method from class
  - [ ] 3.6.9.2. Read class name and method name from constant pool
  - [ ] 3.6.9.3. Look up class in globals
  - [ ] 3.6.9.4. Remove method from class's method table
  - [ ] 3.6.9.5. Update class_definition_ast (remove method node)
  - [ ] 3.6.9.6. Invalidate method cache
  - [ ] 3.6.9.7. **TEST**: Verify method no longer callable (NoMethodError)
  - [ ] 3.6.9.8. Verify test passes

- [ ] 3.6.10. Add Object::AST variant to object model
  - [ ] 3.6.10.1. Add `AST(Rc<Vec<Statement>>)` to Object enum
  - [ ] 3.6.10.2. Implement Display for Object::AST (show AST structure)
  - [ ] 3.6.10.3. Implement Clone for Object::AST
  - [ ] 3.6.10.4. Add AST manipulation methods (modify nodes, traverse, etc.)
  - [ ] 3.6.10.5. **TEST**: Write tests for AST object operations
  - [ ] 3.6.10.6. Document Object::AST API in code comments

### 3.7 Implement Pattern Matching

- [ ] 3.7.1. Implement MATCH_START instruction handler
  - [ ] 3.7.1.1. Initialize pattern matching context
  - [ ] 3.7.1.2. Keep expression value on stack
  - [ ] 3.7.1.3. Write tests for match initialization
- [ ] 3.7.2. Implement MATCH_CASE instruction handler
  - [ ] 3.7.2.1. Read pattern type and jump offset
  - [ ] 3.7.2.2. Pop pattern value from stack
  - [ ] 3.7.2.3. Peek expression value
  - [ ] 3.7.2.4. Test pattern (literal, type, destructure)
  - [ ] 3.7.2.5. If match, continue; if no match, jump
  - [ ] 3.7.2.6. Write tests for case matching
- [ ] 3.7.3. Implement MATCH_BIND instruction handler
  - [ ] 3.7.3.1. Read variable name from constant pool
  - [ ] 3.7.3.2. Pop/peek matched value
  - [ ] 3.7.3.3. Bind to local variable
  - [ ] 3.7.3.4. Write tests for bindings
- [ ] 3.7.4. Implement MATCH_END instruction handler
  - [ ] 3.7.4.1. Clean up match context
  - [ ] 3.7.4.2. Pop expression value
  - [ ] 3.7.4.3. Write tests for match cleanup

---

## Phase 4: Integration and Migration

### 4.1 Dual-Mode Support

- [ ] 4.1.1. Add compilation mode selection
  - [ ] 4.1.1.1. Add `--ast` flag to main.rs for legacy AST interpreter mode
  - [ ] 4.1.1.2. Make bytecode mode the default execution mode
  - [ ] 4.1.1.3. Route execution based on flag (default: bytecode, --ast: tree-walking)
  - [ ] 4.1.1.4. Write tests for mode selection
- [ ] 4.1.2. Update REPL for bytecode mode
  - [ ] 4.1.2.1. Default REPL to bytecode mode
  - [ ] 4.1.2.2. Add toggle command (e.g., `:ast` to switch to AST mode)
  - [ ] 4.1.2.3. Compile and execute each line in bytecode mode by default
  - [ ] 4.1.2.4. Show compilation errors clearly
  - [ ] 4.1.2.5. Test REPL in both modes
- [ ] 4.1.3. Handle REPL-specific challenges
  - [ ] 4.1.3.1. **TEST**: Write test for incremental global definitions in REPL
  - [ ] 4.1.3.2. Design persistent state across REPL inputs (global registry, classes)
  - [ ] 4.1.3.3. Allow redefining classes/methods in REPL without restart
  - [ ] 4.1.3.4. **TEST**: Write test for class redefinition in REPL
  - [ ] 4.1.3.5. Implement constant pool growth across REPL lines
  - [ ] 4.1.3.6. **TEST**: Write test for multi-line input compilation (begin...end across prompts)
  - [ ] 4.1.3.7. Verify all REPL tests pass
- [ ] 4.1.4. Ensure feature parity
  - [ ] 4.1.4.1. Run all existing tests in both modes
  - [ ] 4.1.4.2. Document features only available in one mode (if any)
  - [ ] 4.1.4.3. Add warning message when using deprecated --ast flag
  - [ ] 4.1.4.4. Create migration checklist for eventual AST mode removal

### 4.2 Test Migration

- [ ] 4.2.1. Create bytecode test harness
  - [ ] 4.2.1.1. Copy tests/examples_runner.rs to tests/bytecode_examples_runner.rs
  - [ ] 4.2.1.2. Modify to use bytecode VM
  - [ ] 4.2.1.3. Run all examples in bytecode mode
  - [ ] 4.2.1.4. Track which examples pass/fail
- [ ] 4.2.2. Fix failing tests
  - [ ] 4.2.2.1. Identify root causes of failures
  - [ ] 4.2.2.2. Fix compiler bugs
  - [ ] 4.2.2.3. Fix VM bugs
  - [ ] 4.2.2.4. Update tests if behavior intentionally differs
- [ ] 4.2.3. Run full test suite in both modes
  - [ ] 4.2.3.1. Ensure tests/ directory works in bytecode mode
  - [ ] 4.2.3.2. Add CI checks for bytecode mode
  - [ ] 4.2.3.3. Document test coverage differences

### 4.3 Performance Validation

- [ ] 4.3.1. Create benchmark suite
  - [ ] 4.3.1.1. Create benchmarks/ directory
  - [ ] 4.3.1.2. Write benchmark for arithmetic (fib, factorial)
  - [ ] 4.3.1.3. Write benchmark for method calls
  - [ ] 4.3.1.4. Write benchmark for loops
  - [ ] 4.3.1.5. Write benchmark for collections
  - [ ] 4.3.1.6. Write benchmark for exception handling
- [ ] 4.3.2. Run benchmarks in both modes
  - [ ] 4.3.2.1. Measure execution time for each benchmark
  - [ ] 4.3.2.2. Measure memory usage
  - [ ] 4.3.2.3. Compare AST interpreter vs bytecode VM
  - [ ] 4.3.2.4. Document performance improvements
- [ ] 4.3.3. Optimize hot paths
  - [ ] 4.3.3.1. Profile bytecode VM execution
  - [ ] 4.3.3.2. Identify bottlenecks
  - [ ] 4.3.3.3. Optimize critical instruction handlers
  - [ ] 4.3.3.4. Re-run benchmarks to verify improvements

### 4.4 Error Reporting and Debugging

- [ ] 4.4.1. Implement source line tracking
  - [ ] 4.4.1.1. Emit SET_LINE instructions during compilation
  - [ ] 4.4.1.2. Update VM to track current source line
  - [ ] 4.4.1.3. Include line numbers in stack traces
  - [ ] 4.4.1.4. Write tests for error line reporting
- [ ] 4.4.2. Improve error messages
  - [ ] 4.4.2.1. Map bytecode errors back to source code
  - [ ] 4.4.2.2. Show code snippet with error location
  - [ ] 4.4.2.3. Include call stack in errors
  - [ ] 4.4.2.4. Test error message quality
- [ ] 4.4.3. Implement stack trace generation
  - [ ] 4.4.3.1. Record call frames with source positions
  - [ ] 4.4.3.2. Format stack traces like AST interpreter
  - [ ] 4.4.3.3. Include method names and line numbers
  - [ ] 4.4.3.4. Write tests for stack traces
- [ ] 4.4.4. Add VM tracing mode
  - [ ] 4.4.4.1. Add `--trace` flag to print each instruction (works in bytecode mode)
  - [ ] 4.4.4.2. Show stack state after each instruction
  - [ ] 4.4.4.3. Show call frame changes
  - [ ] 4.4.4.4. Use for debugging compiler/VM issues
  - [ ] 4.4.4.5. Add `--trace-ast` for AST mode tracing (if needed)

### 4.5 Documentation

- [ ] 4.5.1. Document bytecode instruction set
  - [ ] 4.5.1.1. Create BYTECODE_SPEC.md
  - [ ] 4.5.1.2. Describe each instruction with examples
  - [ ] 4.5.1.3. Show encoding format
  - [ ] 4.5.1.4. Explain operand types
- [ ] 4.5.2. Document compilation process
  - [ ] 4.5.2.1. Create COMPILER.md
  - [ ] 4.5.2.2. Explain AST to bytecode transformation
  - [ ] 4.5.2.3. Document optimization passes
  - [ ] 4.5.2.4. Show example compilations
- [ ] 4.5.3. Document VM architecture
  - [ ] 4.5.3.1. Create VM.md
  - [ ] 4.5.3.2. Explain stack-based execution model
  - [ ] 4.5.3.3. Document call frame structure
  - [ ] 4.5.3.4. Explain exception handling mechanism
- [ ] 4.5.4. Update README
  - [ ] 4.5.4.1. Update architecture section to show bytecode as primary execution mode
  - [ ] 4.5.4.2. Add performance comparison (bytecode vs AST interpreter)
  - [ ] 4.5.4.3. Update roadmap to mark Phase 2 complete
  - [ ] 4.5.4.4. Document --ast flag as legacy/debugging option
- [ ] 4.5.5. Create migration guide
  - [ ] 4.5.5.1. Create MIGRATION.md
  - [ ] 4.5.5.2. Explain that bytecode is now default (AST mode via --ast flag only)
  - [ ] 4.5.5.3. Document any breaking changes or behavioral differences
  - [ ] 4.5.5.4. Provide examples of using --ast flag for debugging
  - [ ] 4.5.5.5. Explain when AST mode might eventually be removed

---

## Phase 4.6: Module and Import System (Future-Proofing)

**Context**: While not implemented yet, the bytecode design should support a future module/import system without requiring major refactoring.

### 4.6.1 Design Module Loading Strategy

- [ ] 4.6.1.1. Design module representation
  - [ ] 4.6.1.1.1. Document module concept: separate namespace, exports
  - [ ] 4.6.1.1.2. Decide: module = separate bytecode chunk or namespace in globals?
  - [ ] 4.6.1.1.3. Design Module object structure
  - [ ] 4.6.1.1.4. Write design doc for future implementation
- [ ] 4.6.1.2. Reserve bytecode instructions for modules
  - [ ] 4.6.1.2.1. Add LOAD_MODULE <path> instruction to opcode enum (unimplemented)
  - [ ] 4.6.1.2.2. Add IMPORT <module> <name> instruction (unimplemented)
  - [ ] 4.6.1.2.3. Add EXPORT <name> instruction (unimplemented)
  - [ ] 4.6.1.2.4. Document intended semantics in comments
  - [ ] 4.6.1.2.5. Emit error if these instructions are encountered in Phase 2

### 4.6.2 Cross-Module Constant Pool Strategy

- [ ] 4.6.2.1. Design constant pool sharing
  - [ ] 4.6.2.1.1. Document options: shared global pool vs per-module pools
  - [ ] 4.6.2.1.2. Consider implications for bytecode caching
  - [ ] 4.6.2.1.3. Decide on approach for Phase 3 implementation
- [ ] 4.6.2.2. Ensure bytecode format supports module references
  - [ ] 4.6.2.2.1. Add module ID field to chunk metadata
  - [ ] 4.6.2.2.2. Add module version to serialization format
  - [ ] 4.6.2.2.3. Test round-trip with module metadata

### 4.6.3 Global Resolution Scope

- [ ] 4.6.3.1. Document current global resolution behavior
  - [ ] 4.6.3.1.1. **TEST**: Write test for undefined global access
  - [ ] 4.6.3.1.2. Implement error for undefined globals in bytecode mode
  - [ ] 4.6.3.1.3. Verify consistent with AST mode behavior
- [ ] 4.6.3.2. Design module-aware global lookup
  - [ ] 4.6.3.2.1. Document how LOAD_GLOBAL will search module scope
  - [ ] 4.6.3.2.2. Plan for module import aliases
  - [ ] 4.6.3.2.3. Write design doc for future work

---

## Phase 5: Advanced Bytecode Features

### 5.1 Code Serialization and Caching

- [ ] 5.1.1. Implement bytecode file format (.mxc)
  - [ ] 5.1.1.1. Design file header (magic number, version, metadata)
  - [ ] 5.1.1.2. Implement chunk serialization
  - [ ] 5.1.1.3. Implement constant pool serialization
  - [ ] 5.1.1.4. Add compression (optional)
  - [ ] 5.1.1.5. Write tests for serialization/deserialization
- [ ] 5.1.2. Add bytecode cache
  - [ ] 5.1.2.1. Check for .mxc file when loading .mx
  - [ ] 5.1.2.2. Compare timestamps (recompile if source newer)
  - [ ] 5.1.2.3. Load cached bytecode if valid
  - [ ] 5.1.2.4. Write .mxc after compilation
  - [ ] 5.1.2.5. Test cache invalidation
- [ ] 5.1.3. Add precompilation tool
  - [ ] 5.1.3.1. Create `metorex compile` subcommand
  - [ ] 5.1.3.2. Compile .mx files to .mxc
  - [ ] 5.1.3.3. Support batch compilation
  - [ ] 5.1.3.4. Test precompilation workflow

### 5.2 Bytecode Verification

- [ ] 5.2.1. Implement bytecode verifier
  - [ ] 5.2.1.1. Create `src/bytecode/verifier.rs`
  - [ ] 5.2.1.2. Verify stack depth correctness
  - [ ] 5.2.1.3. Verify jump targets are valid
  - [ ] 5.2.1.4. Verify local variable indices
  - [ ] 5.2.1.5. Verify constant pool references
  - [ ] 5.2.1.6. Write tests for verification
- [ ] 5.2.2. Add verification pass before execution
  - [ ] 5.2.2.1. Run verifier on loaded bytecode
  - [ ] 5.2.2.2. Report verification errors clearly
  - [ ] 5.2.2.3. Skip verification in release mode (optional)
  - [ ] 5.2.2.4. Test invalid bytecode rejection

### 5.3 Reflection and Introspection

- [ ] 5.3.1. Implement bytecode introspection API
  - [ ] 5.3.1.1. Add method.bytecode() to get chunk
  - [ ] 5.3.1.2. Add method.disassemble() to get text
  - [ ] 5.3.1.3. Add class.methods() to list methods
  - [ ] 5.3.1.4. Write tests for introspection
- [ ] 5.3.2. Preserve AST for meta-programming
  - [ ] 5.3.2.1. Store original AST alongside bytecode
  - [ ] 5.3.2.2. Allow AST retrieval for code-as-object features
  - [ ] 5.3.2.3. Balance memory usage vs. functionality
  - [ ] 5.3.2.4. Test meta-programming features work in bytecode mode
- [ ] 5.3.3. Implement runtime code generation
  - [ ] 5.3.3.1. Allow compiling AST to bytecode at runtime
  - [ ] 5.3.3.2. Support eval-like functionality
  - [ ] 5.3.3.3. Ensure security (sandboxing if needed)
  - [ ] 5.3.3.4. Test dynamic code generation

### 5.4 Advanced Optimizations

- [ ] 5.4.1. Implement inline caching for method calls
  - [ ] 5.4.1.1. Cache method lookup results
  - [ ] 5.4.1.2. Invalidate cache on class modification
  - [ ] 5.4.1.3. Measure performance improvement
  - [ ] 5.4.1.4. Write tests for cache correctness
- [ ] 5.4.2. Implement type specialization
  - [ ] 5.4.2.1. Generate specialized bytecode for common types
  - [ ] 5.4.2.2. E.g., int + int vs. generic +
  - [ ] 5.4.2.3. Measure performance improvement
  - [ ] 5.4.2.4. Write tests for specialized operations
- [ ] 5.4.3. Implement loop optimization
  - [ ] 5.4.3.1. Detect loop invariants
  - [ ] 5.4.3.2. Hoist constant computations out of loops
  - [ ] 5.4.3.3. Unroll small loops (optional)
  - [ ] 5.4.3.4. Test optimization correctness
- [ ] 5.4.4. Implement tail call optimization
  - [ ] 5.4.4.1. Detect tail calls in compiler
  - [ ] 5.4.4.2. Replace CALL + RETURN with tail call instruction
  - [ ] 5.4.4.3. Implement tail call handler in VM
  - [ ] 5.4.4.4. Test recursive tail calls don't overflow stack

---

## Phase 6: Preparation for JIT (Phase 3 Preview)

### 6.1 Profiling Infrastructure

- [ ] 6.1.1. Implement bytecode profiler
  - [ ] 6.1.1.1. Track instruction execution counts
  - [ ] 6.1.1.2. Track method call frequencies
  - [ ] 6.1.1.3. Identify hot methods and loops
  - [ ] 6.1.1.4. Output profiling data in readable format
  - [ ] 6.1.1.5. Write tests for profiler
- [ ] 6.1.2. Add profiling mode to CLI
  - [ ] 6.1.2.1. Add `--profile` flag
  - [ ] 6.1.2.2. Run with profiling enabled
  - [ ] 6.1.2.3. Output profiling report
  - [ ] 6.1.2.4. Test profiling overhead is acceptable

### 6.2 Intermediate Representation (IR) Design

- [ ] 6.2.1. Design IR for JIT compilation
  - [ ] 6.2.1.1. Define IR instruction set (SSA-based)
  - [ ] 6.2.1.2. Map bytecode instructions to IR
  - [ ] 6.2.1.3. Document IR specification
  - [ ] 6.2.1.4. Create examples of bytecode → IR
- [ ] 6.2.2. Implement bytecode to IR converter
  - [ ] 6.2.2.1. Create `src/jit/ir.rs`
  - [ ] 6.2.2.2. Translate bytecode to IR
  - [ ] 6.2.2.3. Perform IR optimizations (constant prop, DCE)
  - [ ] 6.2.2.4. Write tests for IR conversion

### 6.3 LLVM Integration Preparation

- [ ] 6.3.1. Add LLVM dependency
  - [ ] 6.3.1.1. Add inkwell crate to Cargo.toml
  - [ ] 6.3.1.2. Create `src/jit/llvm.rs` module
  - [ ] 6.3.1.3. Set up LLVM context and module
  - [ ] 6.3.1.4. Write basic LLVM integration test
- [ ] 6.3.2. Implement simple IR to LLVM translation
  - [ ] 6.3.2.1. Translate arithmetic operations to LLVM
  - [ ] 6.3.2.2. Translate function calls to LLVM
  - [ ] 6.3.2.3. Generate machine code
  - [ ] 6.3.2.4. Test simple compiled functions
- [ ] 6.3.3. Design hot path compilation strategy
  - [ ] 6.3.3.1. Define threshold for JIT compilation
  - [ ] 6.3.3.2. Implement method/loop hotness detection
  - [ ] 6.3.3.3. Trigger compilation on hot code
  - [ ] 6.3.3.4. Test JIT trigger mechanism

---

## Phase 4.7: Comprehensive Testing for Complex Features

**Context**: Closures, GC integration, and yield semantics are notoriously bug-prone. Comprehensive testing beyond basic functionality is critical.

### 4.7.1 Closure Edge Case Testing

- [ ] 4.7.1.1. **TEST**: Closure capturing multiple variables at different nesting levels
- [ ] 4.7.1.2. **TEST**: Closure returned from closure (nested closures)
- [ ] 4.7.1.3. **TEST**: Multiple closures sharing same captured variable
- [ ] 4.7.1.4. **TEST**: Closure accessing variables after frame has returned
- [ ] 4.7.1.5. **TEST**: Closure modifying captured variable, visible in outer scope
- [ ] 4.7.1.6. **TEST**: Closure in loop capturing loop variable (classic closure bug)
- [ ] 4.7.1.7. **TEST**: Recursive function creating closures
- [ ] 4.7.1.8. Verify all closure edge cases pass

### 4.7.2 GC Stress Testing

- [ ] 4.7.2.1. **TEST**: Create many objects during bytecode execution, trigger GC
- [ ] 4.7.2.2. **TEST**: Deep call stack with GC in middle of execution
- [ ] 4.7.2.3. **TEST**: Closures surviving GC cycles
- [ ] 4.7.2.4. **TEST**: Exceptions thrown during GC don't corrupt VM state
- [ ] 4.7.2.5. **TEST**: Large constant pool with GC
- [ ] 4.7.2.6. Create GC stress test suite (runs with frequent GC triggers)
- [ ] 4.7.2.7. Verify no memory leaks with valgrind/ASAN

### 4.7.3 Yield and Block Testing

- [ ] 4.7.3.1. **TEST**: Yield with 0, 1, many arguments
- [ ] 4.7.3.2. **TEST**: Nested yields (method yields to block that yields to another block)
- [ ] 4.7.3.3. **TEST**: Return from within block returns from enclosing method
- [ ] 4.7.3.4. **TEST**: Exception in block propagates correctly
- [ ] 4.7.3.5. **TEST**: Block with block_given? checks
- [ ] 4.7.3.6. **TEST**: Iterator methods (.each, .map) using yield
- [ ] 4.7.3.7. Verify all yield semantics match Ruby behavior

### 4.7.4 Exception Handling Stress Testing

- [ ] 4.7.4.1. **TEST**: Deeply nested try/rescue blocks
- [ ] 4.7.4.2. **TEST**: Exception in ensure block
- [ ] 4.7.4.3. **TEST**: Re-raise from rescue
- [ ] 4.7.4.4. **TEST**: Exception during method lookup
- [ ] 4.7.4.5. **TEST**: Exception in native method propagates correctly
- [ ] 4.7.4.6. **TEST**: Stack unwinding through multiple frames
- [ ] 4.7.4.7. **TEST**: Ensure blocks run during unwinding
- [ ] 4.7.4.8. Verify all exception tests pass

### 4.7.5 Property-Based Testing

- [ ] 4.7.5.1. Set up property-based testing framework (proptest crate)
- [ ] 4.7.5.2. **PROPERTY**: Bytecode mode output == AST mode output for all valid programs
- [ ] 4.7.5.3. **PROPERTY**: All bytecode instructions maintain valid stack depth
- [ ] 4.7.5.4. **PROPERTY**: Constant pool indices are always in bounds
- [ ] 4.7.5.5. **PROPERTY**: GC never collects reachable objects
- [ ] 4.7.5.6. Run property tests with 10000+ generated cases
- [ ] 4.7.5.7. Fix any failures discovered

### 4.7.6 Fuzzing

- [ ] 4.7.6.1. Set up cargo-fuzz for compiler
- [ ] 4.7.6.2. Fuzz AST → bytecode compiler (should never crash)
- [ ] 4.7.6.3. Fuzz bytecode verifier
- [ ] 4.7.6.4. Fuzz bytecode deserializer (.mxc file loading)
- [ ] 4.7.6.5. Run fuzzer for 24 hours, fix all crashes
- [ ] 4.7.6.6. Add regression tests for all fuzzer-discovered issues

---

## Test-Driven Development Notes

**All phases above follow TDD**:
1. Write failing test first (**TEST**: marked items)
2. Implement minimum code to make test pass
3. Verify test passes and run `cargo test && cargo clippy && cargo fmt`
4. Check coverage with `cargo tarpaulin` - must remain at 100%
5. Run `scripts/misplaced_tests.sh` to ensure no tests in src/

**Test organization**:
- All tests go in `tests/` directory (never in src/)
- One test file per module (e.g., tests/bytecode_compiler_tests.rs)
- Use descriptive test names (test_compile_integer_literal_emits_push_const)
- Create tests/bytecode_test_utils.rs for shared helpers

**Coverage requirement**:
- Every line of new bytecode code must be covered by tests
- Run `cargo tarpaulin` after each implementation
- Fix any coverage gaps before moving to next task

---

## Post-Implementation Validation

### Final Integration Testing (After All Phases Complete)

- [ ] Run all existing examples/ in bytecode mode vs AST mode, compare outputs
- [ ] Run full test suite: `cargo test` - all tests must pass
- [ ] Verify 100% code coverage: `cargo tarpaulin`
- [ ] Run clippy: `cargo clippy` - zero warnings
- [ ] Run formatter: `cargo fmt --check`
- [ ] Run misplaced tests check: `scripts/misplaced_tests.sh`
- [ ] Performance benchmarks show 5-10x improvement
- [ ] All documentation updated (README, BYTECODE_SPEC.md, VM.md, COMPILER.md, MIGRATION.md)

**Remember**: The TDD cycle is:
1. **Red**: Write failing test
2. **Green**: Write minimum code to pass
3. **Refactor**: Clean up while keeping tests green
4. **Verify**: Run full suite + coverage + clippy + fmt

---

## Validation and Completion

### Final Checklist

- [ ] All existing tests pass in bytecode mode
- [ ] Performance is at least 5x faster than AST interpreter
- [ ] Code coverage remains at 100%
- [ ] No clippy warnings in new code
- [ ] All documentation is updated
- [ ] Examples work in bytecode mode
- [ ] REPL works in bytecode mode
- [ ] Error messages are clear and helpful
- [ ] Stack traces are accurate
- [ ] Meta-programming features work correctly
- [ ] README.md Phase 2 is marked complete

---

## Success Criteria

1. ✅ All 150+ existing tests pass in bytecode mode
2. ✅ Bytecode VM is 5-10x faster than AST interpreter
3. ✅ 100% code coverage maintained
4. ✅ Meta-programming (code-as-object) still works
5. ✅ Exception handling with stack traces works correctly
6. ✅ Disassembler shows clear, readable bytecode
7. ✅ Documentation explains bytecode system comprehensively
8. ✅ Performance benchmarks show clear improvement
9. ✅ REPL works seamlessly in both modes
10. ✅ Ready for Phase 3 (JIT compilation)
