// Expression evaluation, split by category.
//
// `dispatch` holds the small `match` over `Expression` variants in
// `evaluate_expression_inner`. Each variant delegates to a helper that lives
// in one of the sibling modules below:
//
//   - identifier: Identifier resolution (vars, methods, classes, bare `new`)
//   - calls:      Call (with bare-id self-method fallback)
//   - super_expr: Super dispatch
//   - defined:    `defined?` introspection
//   - yield_expr: yield to block
//   - vars:       SelfExpr / InstanceVariable / ClassVariable / GlobalVariable

mod calls;
mod defined;
mod dispatch;
mod identifier;
mod singleton_class_expr;
mod super_expr;
mod vars;
mod yield_expr;
