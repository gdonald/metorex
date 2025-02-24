use metorex::ast::{BinaryOp, UnaryOp};

#[test]
fn test_binary_op_display() {
    assert_eq!(format!("{}", BinaryOp::Add), "+");
    assert_eq!(format!("{}", BinaryOp::Subtract), "-");
    assert_eq!(format!("{}", BinaryOp::Multiply), "*");
    assert_eq!(format!("{}", BinaryOp::Divide), "/");
    assert_eq!(format!("{}", BinaryOp::Modulo), "%");
    assert_eq!(format!("{}", BinaryOp::Equal), "==");
    assert_eq!(format!("{}", BinaryOp::NotEqual), "!=");
    assert_eq!(format!("{}", BinaryOp::Less), "<");
    assert_eq!(format!("{}", BinaryOp::Greater), ">");
    assert_eq!(format!("{}", BinaryOp::LessEqual), "<=");
    assert_eq!(format!("{}", BinaryOp::GreaterEqual), ">=");
    assert_eq!(format!("{}", BinaryOp::Assign), "=");
    assert_eq!(format!("{}", BinaryOp::AddAssign), "+=");
    assert_eq!(format!("{}", BinaryOp::SubtractAssign), "-=");
    assert_eq!(format!("{}", BinaryOp::MultiplyAssign), "*=");
    assert_eq!(format!("{}", BinaryOp::DivideAssign), "/=");
}

#[test]
fn test_unary_op_display() {
    assert_eq!(format!("{}", UnaryOp::Plus), "+");
    assert_eq!(format!("{}", UnaryOp::Minus), "-");
}
