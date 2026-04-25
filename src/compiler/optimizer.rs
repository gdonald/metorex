// Optimization passes for compiled bytecode.
//
// These operate on a Chunk after compilation, rewriting instruction
// sequences for better performance.

use crate::bytecode::chunk::Chunk;
use crate::bytecode::opcode::OpCode;
use crate::object::Object;

/// Run all optimization passes on a chunk.
/// Runs constant folding and peephole in a fixed-point loop (max 3 iterations)
/// since one pass may create new opportunities for another.
pub fn optimize(chunk: &mut Chunk) {
    for _ in 0..3 {
        let before = chunk.code().to_vec();
        constant_fold(chunk);
        peephole_optimize(chunk);
        if chunk.code() == before.as_slice() {
            break;
        }
    }
    dead_code_eliminate(chunk);
}

// ── Constant folding (12.9.1) ──────────────────────────────────────────
//
// Folds sequences like:
//   Constant(a), Constant(b), Add  →  Constant(a+b)
//   Constant(a), Constant(b), Subtract  →  Constant(a-b)
//   Constant(a), Constant(b), Multiply  →  Constant(a*b)
//   Constant(a), Negate  →  Constant(-a)
//
// We rebuild the chunk rather than patching in place, since instruction
// sizes may change.

pub fn constant_fold(chunk: &mut Chunk) {
    let code = chunk.code().to_vec();
    let mut new_chunk = Chunk::new();

    // Copy over all existing constants so indices stay valid
    for i in 0..chunk.constants_count() {
        new_chunk.add_constant(chunk.get_constant(i).clone());
    }

    let mut i = 0;
    while i < code.len() {
        let Some(op) = OpCode::from_byte(code[i]) else {
            new_chunk.write_byte(code[i], chunk.get_line(i));
            i += 1;
            continue;
        };

        // Try: Constant(a), Negate → Constant(-a)
        if op == OpCode::Constant
            && i + 2 < code.len()
            && let Some(OpCode::Negate) = OpCode::from_byte(code[i + 2])
        {
            let idx = code[i + 1] as usize;
            let val = chunk.get_constant(idx);
            if let Some(folded) = negate_constant(val) {
                let new_idx = new_chunk.add_constant(folded);
                if let Some(new_idx) = new_idx {
                    let line = chunk.get_line(i);
                    new_chunk.write_constant(new_idx, line);
                    i += 3; // skip Constant + operand + Negate
                    continue;
                }
            }
        }

        // Try: Constant(a), Constant(b), BinOp → Constant(result)
        if op == OpCode::Constant
            && i + 4 < code.len()
            && let Some(OpCode::Constant) = OpCode::from_byte(code[i + 2])
            && let Some(bin_op) = OpCode::from_byte(code[i + 4])
            && is_foldable_binop(bin_op)
        {
            let a_idx = code[i + 1] as usize;
            let b_idx = code[i + 3] as usize;
            let a = chunk.get_constant(a_idx);
            let b = chunk.get_constant(b_idx);
            if let Some(folded) = fold_binary(a, b, bin_op) {
                let new_idx = new_chunk.add_constant(folded);
                if let Some(new_idx) = new_idx {
                    let line = chunk.get_line(i);
                    new_chunk.write_constant(new_idx, line);
                    i += 5; // skip Const+op + Const+op + BinOp
                    continue;
                }
            }
        }

        // No folding — copy instruction as-is
        let size = 1 + op.operand_size();
        for j in 0..size {
            new_chunk.write_byte(code[i + j], chunk.get_line(i + j));
        }
        i += size;
    }

    *chunk = new_chunk;
}

fn is_foldable_binop(op: OpCode) -> bool {
    matches!(
        op,
        OpCode::Add | OpCode::Subtract | OpCode::Multiply | OpCode::Divide | OpCode::Modulo
    )
}

fn fold_binary(a: &Object, b: &Object, op: OpCode) -> Option<Object> {
    match (a, b, op) {
        (Object::Int(a), Object::Int(b), OpCode::Add) => Some(Object::Int(a + b)),
        (Object::Int(a), Object::Int(b), OpCode::Subtract) => Some(Object::Int(a - b)),
        (Object::Int(a), Object::Int(b), OpCode::Multiply) => Some(Object::Int(a * b)),
        (Object::Int(a), Object::Int(b), OpCode::Divide) if *b != 0 => Some(Object::Int(a / b)),
        (Object::Int(a), Object::Int(b), OpCode::Modulo) if *b != 0 => Some(Object::Int(a % b)),
        (Object::Float(a), Object::Float(b), OpCode::Add) => Some(Object::Float(a + b)),
        (Object::Float(a), Object::Float(b), OpCode::Subtract) => Some(Object::Float(a - b)),
        (Object::Float(a), Object::Float(b), OpCode::Multiply) => Some(Object::Float(a * b)),
        (Object::Float(a), Object::Float(b), OpCode::Divide) if *b != 0.0 => {
            Some(Object::Float(a / b))
        }
        _ => None,
    }
}

fn negate_constant(val: &Object) -> Option<Object> {
    match val {
        Object::Int(n) => Some(Object::Int(-n)),
        Object::Float(f) => Some(Object::Float(-f)),
        _ => None,
    }
}

// ── Dead code elimination (12.9.2) ────────────────────────────────────
//
// Removes unreachable instructions after an unconditional Return.
// Stops removing when it hits a jump target (another instruction could
// jump to here). Since we don't track jump targets precisely, we stop
// at the next instruction that follows a Return within the same basic
// block — we only eliminate simple sequences like `Return, Pop, ...`.

pub fn dead_code_eliminate(chunk: &mut Chunk) {
    let code = chunk.code().to_vec();
    let mut new_chunk = Chunk::new();

    // Copy constants
    for i in 0..chunk.constants_count() {
        new_chunk.add_constant(chunk.get_constant(i).clone());
    }

    // Collect all jump targets so we don't eliminate code that is reachable
    let jump_targets = collect_jump_targets(&code);

    let mut i = 0;
    let mut after_return = false;

    while i < code.len() {
        // If this offset is a jump target, code is reachable again
        if jump_targets.contains(&i) {
            after_return = false;
        }

        let Some(op) = OpCode::from_byte(code[i]) else {
            if !after_return {
                new_chunk.write_byte(code[i], chunk.get_line(i));
            }
            i += 1;
            continue;
        };

        let size = 1 + op.operand_size();

        if after_return {
            // Skip this instruction (dead code)
            i += size;
            continue;
        }

        // Copy instruction
        for j in 0..size {
            new_chunk.write_byte(code[i + j], chunk.get_line(i + j));
        }

        if op == OpCode::Return {
            after_return = true;
        }

        i += size;
    }

    *chunk = new_chunk;
}

fn collect_jump_targets(code: &[u8]) -> Vec<usize> {
    let mut targets = Vec::new();
    let mut i = 0;
    while i < code.len() {
        let Some(op) = OpCode::from_byte(code[i]) else {
            i += 1;
            continue;
        };
        let size = 1 + op.operand_size();

        match op {
            OpCode::Jump | OpCode::JumpIfFalse if i + 2 < code.len() => {
                let offset = ((code[i + 1] as usize) << 8) | code[i + 2] as usize;
                let target = i + size + offset;
                if target < code.len() {
                    targets.push(target);
                }
            }
            OpCode::Loop if i + 2 < code.len() => {
                let offset = ((code[i + 1] as usize) << 8) | code[i + 2] as usize;
                if offset <= i + size {
                    targets.push(i + size - offset);
                }
            }
            _ => {}
        }

        i += size;
    }
    targets
}

// ── Peephole optimization (12.9.3) ────────────────────────────────────
//
// Eliminates redundant instruction patterns:
// - Push then immediate Pop: `Constant X, Pop` when X has no side effects
//   (This would eliminate dead expression statements, but we keep those
//    since the Pop is intentional for expression statements.)
// - Double negation: `Negate, Negate` → (nothing)
// - Double not: `Not, Not` → (nothing)

pub fn peephole_optimize(chunk: &mut Chunk) {
    let code = chunk.code().to_vec();
    let mut new_chunk = Chunk::new();

    // Copy constants
    for i in 0..chunk.constants_count() {
        new_chunk.add_constant(chunk.get_constant(i).clone());
    }

    let mut i = 0;
    while i < code.len() {
        let Some(op) = OpCode::from_byte(code[i]) else {
            new_chunk.write_byte(code[i], chunk.get_line(i));
            i += 1;
            continue;
        };

        // Double negate: Negate, Negate → skip both
        if op == OpCode::Negate
            && i + 1 < code.len()
            && let Some(OpCode::Negate) = OpCode::from_byte(code[i + 1])
        {
            i += 2;
            continue;
        }

        // Double not: Not, Not → skip both
        if op == OpCode::Not
            && i + 1 < code.len()
            && let Some(OpCode::Not) = OpCode::from_byte(code[i + 1])
        {
            i += 2;
            continue;
        }

        // Nil, Pop → skip both (pushing nil just to pop it)
        if op == OpCode::Nil
            && i + 1 < code.len()
            && let Some(OpCode::Pop) = OpCode::from_byte(code[i + 1])
        {
            i += 2;
            continue;
        }

        // True, Not → False
        if op == OpCode::True
            && i + 1 < code.len()
            && let Some(OpCode::Not) = OpCode::from_byte(code[i + 1])
        {
            new_chunk.write_byte(OpCode::False.to_byte(), chunk.get_line(i));
            i += 2;
            continue;
        }

        // False, Not → True
        if op == OpCode::False
            && i + 1 < code.len()
            && let Some(OpCode::Not) = OpCode::from_byte(code[i + 1])
        {
            new_chunk.write_byte(OpCode::True.to_byte(), chunk.get_line(i));
            i += 2;
            continue;
        }

        // No optimization — copy as-is
        let size = 1 + op.operand_size();
        for j in 0..size {
            new_chunk.write_byte(code[i + j], chunk.get_line(i + j));
        }
        i += size;
    }

    *chunk = new_chunk;
}

// ── Tail call optimization (12.9.4) ──────────────────────────────────
//
// Detects `Call N, Return` sequences and marks them. In a full
// implementation, the VM would reuse the current call frame instead
// of pushing a new one. For now we just detect and annotate by
// replacing the sequence with `Call N, Return` unchanged (the VM
// can check for this pattern). A proper TCO would require a TailCall
// opcode, which we don't have yet — so this pass is a detection-only
// placeholder that verifies the pattern exists.

pub fn count_tail_calls(chunk: &Chunk) -> usize {
    let code = chunk.code();
    let mut count = 0;
    let mut i = 0;
    while i < code.len() {
        let Some(op) = OpCode::from_byte(code[i]) else {
            i += 1;
            continue;
        };
        let size = 1 + op.operand_size();

        if op == OpCode::Call
            && i + size < code.len()
            && let Some(OpCode::Return) = OpCode::from_byte(code[i + size])
        {
            count += 1;
        }

        i += size;
    }
    count
}
