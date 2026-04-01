// Tests for the OpCode enum

use metorex::bytecode::opcode::OpCode;

// ── Round-trip: to_byte → from_byte ─────────────────────────────────────

#[test]
fn all_opcodes_round_trip() {
    let opcodes = [
        OpCode::Constant,
        OpCode::ConstantLong,
        OpCode::Nil,
        OpCode::True,
        OpCode::False,
        OpCode::Pop,
        OpCode::GetLocal,
        OpCode::SetLocal,
        OpCode::GetGlobal,
        OpCode::SetGlobal,
        OpCode::DefineGlobal,
        OpCode::GetInstance,
        OpCode::SetInstance,
        OpCode::GetUpvalue,
        OpCode::SetUpvalue,
        OpCode::CloseUpvalue,
        OpCode::Add,
        OpCode::Subtract,
        OpCode::Multiply,
        OpCode::Divide,
        OpCode::Modulo,
        OpCode::Negate,
        OpCode::Equal,
        OpCode::NotEqual,
        OpCode::Greater,
        OpCode::Less,
        OpCode::GreaterEqual,
        OpCode::LessEqual,
        OpCode::Not,
        OpCode::Jump,
        OpCode::JumpIfFalse,
        OpCode::Loop,
        OpCode::Call,
        OpCode::Invoke,
        OpCode::Return,
        OpCode::Closure,
        OpCode::Class,
        OpCode::Method,
        OpCode::Array,
        OpCode::Hash,
        OpCode::IndexGet,
        OpCode::IndexSet,
        OpCode::Try,
        OpCode::Catch,
        OpCode::Finally,
        OpCode::EndTry,
        OpCode::Raise,
        OpCode::Match,
        OpCode::MatchPattern,
    ];

    for op in opcodes {
        let byte = op.to_byte();
        let decoded = OpCode::from_byte(byte);
        assert_eq!(
            decoded,
            Some(op),
            "Round-trip failed for {:?} (byte {})",
            op,
            byte
        );
    }
}

// ── from_byte returns None for invalid bytes ────────────────────────────

#[test]
fn invalid_byte_returns_none() {
    assert_eq!(OpCode::from_byte(255), None);
    assert_eq!(OpCode::from_byte(5), None);
    assert_eq!(OpCode::from_byte(99), None);
    assert_eq!(OpCode::from_byte(200), None);
}

// ── name() returns OP_ prefixed names ───────────────────────────────────

#[test]
fn opcode_names() {
    assert_eq!(OpCode::Constant.name(), "OP_CONSTANT");
    assert_eq!(OpCode::ConstantLong.name(), "OP_CONSTANT_LONG");
    assert_eq!(OpCode::Nil.name(), "OP_NIL");
    assert_eq!(OpCode::True.name(), "OP_TRUE");
    assert_eq!(OpCode::False.name(), "OP_FALSE");
    assert_eq!(OpCode::Pop.name(), "OP_POP");
    assert_eq!(OpCode::GetLocal.name(), "OP_GET_LOCAL");
    assert_eq!(OpCode::SetLocal.name(), "OP_SET_LOCAL");
    assert_eq!(OpCode::GetGlobal.name(), "OP_GET_GLOBAL");
    assert_eq!(OpCode::SetGlobal.name(), "OP_SET_GLOBAL");
    assert_eq!(OpCode::DefineGlobal.name(), "OP_DEFINE_GLOBAL");
    assert_eq!(OpCode::GetInstance.name(), "OP_GET_INSTANCE");
    assert_eq!(OpCode::SetInstance.name(), "OP_SET_INSTANCE");
    assert_eq!(OpCode::GetUpvalue.name(), "OP_GET_UPVALUE");
    assert_eq!(OpCode::SetUpvalue.name(), "OP_SET_UPVALUE");
    assert_eq!(OpCode::CloseUpvalue.name(), "OP_CLOSE_UPVALUE");
    assert_eq!(OpCode::Add.name(), "OP_ADD");
    assert_eq!(OpCode::Subtract.name(), "OP_SUBTRACT");
    assert_eq!(OpCode::Multiply.name(), "OP_MULTIPLY");
    assert_eq!(OpCode::Divide.name(), "OP_DIVIDE");
    assert_eq!(OpCode::Modulo.name(), "OP_MODULO");
    assert_eq!(OpCode::Negate.name(), "OP_NEGATE");
    assert_eq!(OpCode::Equal.name(), "OP_EQUAL");
    assert_eq!(OpCode::NotEqual.name(), "OP_NOT_EQUAL");
    assert_eq!(OpCode::Greater.name(), "OP_GREATER");
    assert_eq!(OpCode::Less.name(), "OP_LESS");
    assert_eq!(OpCode::GreaterEqual.name(), "OP_GREATER_EQUAL");
    assert_eq!(OpCode::LessEqual.name(), "OP_LESS_EQUAL");
    assert_eq!(OpCode::Not.name(), "OP_NOT");
    assert_eq!(OpCode::Jump.name(), "OP_JUMP");
    assert_eq!(OpCode::JumpIfFalse.name(), "OP_JUMP_IF_FALSE");
    assert_eq!(OpCode::Loop.name(), "OP_LOOP");
    assert_eq!(OpCode::Call.name(), "OP_CALL");
    assert_eq!(OpCode::Invoke.name(), "OP_INVOKE");
    assert_eq!(OpCode::Return.name(), "OP_RETURN");
    assert_eq!(OpCode::Closure.name(), "OP_CLOSURE");
    assert_eq!(OpCode::Class.name(), "OP_CLASS");
    assert_eq!(OpCode::Method.name(), "OP_METHOD");
    assert_eq!(OpCode::Array.name(), "OP_ARRAY");
    assert_eq!(OpCode::Hash.name(), "OP_HASH");
    assert_eq!(OpCode::IndexGet.name(), "OP_INDEX_GET");
    assert_eq!(OpCode::IndexSet.name(), "OP_INDEX_SET");
    assert_eq!(OpCode::Try.name(), "OP_TRY");
    assert_eq!(OpCode::Catch.name(), "OP_CATCH");
    assert_eq!(OpCode::Finally.name(), "OP_FINALLY");
    assert_eq!(OpCode::EndTry.name(), "OP_END_TRY");
    assert_eq!(OpCode::Raise.name(), "OP_RAISE");
    assert_eq!(OpCode::Match.name(), "OP_MATCH");
    assert_eq!(OpCode::MatchPattern.name(), "OP_MATCH_PATTERN");
}

// ── Display trait ───────────────────────────────────────────────────────

#[test]
fn opcode_display() {
    assert_eq!(format!("{}", OpCode::Nil), "OP_NIL");
    assert_eq!(format!("{}", OpCode::Closure), "OP_CLOSURE");
}

// ── operand_size() ──────────────────────────────────────────────────────

#[test]
fn no_operand_opcodes() {
    assert_eq!(OpCode::Nil.operand_size(), 0);
    assert_eq!(OpCode::True.operand_size(), 0);
    assert_eq!(OpCode::False.operand_size(), 0);
    assert_eq!(OpCode::Pop.operand_size(), 0);
    assert_eq!(OpCode::Add.operand_size(), 0);
    assert_eq!(OpCode::Subtract.operand_size(), 0);
    assert_eq!(OpCode::Return.operand_size(), 0);
    assert_eq!(OpCode::Not.operand_size(), 0);
    assert_eq!(OpCode::Negate.operand_size(), 0);
    assert_eq!(OpCode::Equal.operand_size(), 0);
    assert_eq!(OpCode::IndexGet.operand_size(), 0);
    assert_eq!(OpCode::Raise.operand_size(), 0);
    assert_eq!(OpCode::Match.operand_size(), 0);
    assert_eq!(OpCode::CloseUpvalue.operand_size(), 0);
    assert_eq!(OpCode::Catch.operand_size(), 0);
    assert_eq!(OpCode::Finally.operand_size(), 0);
    assert_eq!(OpCode::EndTry.operand_size(), 0);
}

#[test]
fn one_byte_operand_opcodes() {
    assert_eq!(OpCode::Constant.operand_size(), 1);
    assert_eq!(OpCode::GetLocal.operand_size(), 1);
    assert_eq!(OpCode::SetLocal.operand_size(), 1);
    assert_eq!(OpCode::GetGlobal.operand_size(), 1);
    assert_eq!(OpCode::SetGlobal.operand_size(), 1);
    assert_eq!(OpCode::DefineGlobal.operand_size(), 1);
    assert_eq!(OpCode::GetInstance.operand_size(), 1);
    assert_eq!(OpCode::SetInstance.operand_size(), 1);
    assert_eq!(OpCode::GetUpvalue.operand_size(), 1);
    assert_eq!(OpCode::SetUpvalue.operand_size(), 1);
    assert_eq!(OpCode::Call.operand_size(), 1);
    assert_eq!(OpCode::Closure.operand_size(), 1);
    assert_eq!(OpCode::Class.operand_size(), 1);
    assert_eq!(OpCode::Method.operand_size(), 1);
    assert_eq!(OpCode::Array.operand_size(), 1);
    assert_eq!(OpCode::Hash.operand_size(), 1);
}

#[test]
fn two_byte_operand_opcodes() {
    assert_eq!(OpCode::ConstantLong.operand_size(), 2);
    assert_eq!(OpCode::Jump.operand_size(), 2);
    assert_eq!(OpCode::JumpIfFalse.operand_size(), 2);
    assert_eq!(OpCode::Loop.operand_size(), 2);
    assert_eq!(OpCode::Invoke.operand_size(), 2);
    assert_eq!(OpCode::Try.operand_size(), 2);
    assert_eq!(OpCode::MatchPattern.operand_size(), 2);
}

// ── Distinct byte values ────────────────────────────────────────────────

#[test]
fn all_opcodes_have_distinct_byte_values() {
    let opcodes = [
        OpCode::Constant,
        OpCode::ConstantLong,
        OpCode::Nil,
        OpCode::True,
        OpCode::False,
        OpCode::Pop,
        OpCode::GetLocal,
        OpCode::SetLocal,
        OpCode::GetGlobal,
        OpCode::SetGlobal,
        OpCode::DefineGlobal,
        OpCode::GetInstance,
        OpCode::SetInstance,
        OpCode::GetUpvalue,
        OpCode::SetUpvalue,
        OpCode::CloseUpvalue,
        OpCode::Add,
        OpCode::Subtract,
        OpCode::Multiply,
        OpCode::Divide,
        OpCode::Modulo,
        OpCode::Negate,
        OpCode::Equal,
        OpCode::NotEqual,
        OpCode::Greater,
        OpCode::Less,
        OpCode::GreaterEqual,
        OpCode::LessEqual,
        OpCode::Not,
        OpCode::Jump,
        OpCode::JumpIfFalse,
        OpCode::Loop,
        OpCode::Call,
        OpCode::Invoke,
        OpCode::Return,
        OpCode::Closure,
        OpCode::Class,
        OpCode::Method,
        OpCode::Array,
        OpCode::Hash,
        OpCode::IndexGet,
        OpCode::IndexSet,
        OpCode::Try,
        OpCode::Catch,
        OpCode::Finally,
        OpCode::EndTry,
        OpCode::Raise,
        OpCode::Match,
        OpCode::MatchPattern,
    ];

    let mut seen = std::collections::HashSet::new();
    for op in opcodes {
        let byte = op.to_byte();
        assert!(
            seen.insert(byte),
            "Duplicate byte value {} for {:?}",
            byte,
            op
        );
    }
}
