// The operators that report ancestry, and the elements an operand
// stands for.

use super::*;

impl VirtualMachine {
    /// Whether `elements` holds a value `eql?` to `wanted`, which is how Ruby
    /// matches for array difference, intersection, and union.
    pub(crate) fn contains_eql(
        &mut self,
        elements: &[Object],
        wanted: &Object,
        position: Position,
    ) -> Result<bool, MetorexError> {
        // The value being looked up is the one asked, so an object of the
        // program's own answers `eql?` for itself. The same object counts
        // whatever its `eql?` says, which is what a hash lookup amounts to.
        for element in elements {
            if same_object(element, wanted) || self.values_eql(wanted, element, position)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Evaluate `Module#<`, `#<=`, `#>`, and `#>=`. These report ancestry:
    /// true when the relationship holds, false when the opposite relationship
    /// holds, and nil when the two are unrelated. A non class/module argument
    /// raises TypeError.
    pub(crate) fn evaluate_module_comparison(
        &mut self,
        op: &BinaryOp,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (Object::Class(left_rc) | Object::Module(left_rc)) = &left else {
            unreachable!("caller checks the receiver is a class or module")
        };
        let (Object::Class(right_rc) | Object::Module(right_rc)) = &right else {
            let msg = "compared with non class/module".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("TypeError", msg.clone()),
                location: position_to_location(position),
                message: msg,
            });
        };

        let left_is_descendant = self.builtins().is_subclass_of(left_rc, right_rc);
        let right_is_descendant = self.builtins().is_subclass_of(right_rc, left_rc);
        let same = left_is_descendant && right_is_descendant;

        let descendant_side = match op {
            BinaryOp::Less | BinaryOp::LessEqual => left_is_descendant,
            BinaryOp::Greater | BinaryOp::GreaterEqual => right_is_descendant,
            _ => unreachable!("only ordering operators reach module comparison"),
        };
        let ancestor_side = match op {
            BinaryOp::Less | BinaryOp::LessEqual => right_is_descendant,
            BinaryOp::Greater | BinaryOp::GreaterEqual => left_is_descendant,
            _ => unreachable!("only ordering operators reach module comparison"),
        };

        if same {
            let inclusive = matches!(op, BinaryOp::LessEqual | BinaryOp::GreaterEqual);
            return Ok(Object::Bool(inclusive));
        }
        if descendant_side {
            return Ok(Object::Bool(true));
        }
        if ancestor_side {
            return Ok(Object::Bool(false));
        }
        Ok(Object::Nil)
    }

    /// The elements of an operand that stands for an array: an Array itself,
    /// an instance of an Array subclass, or an object answering `to_ary`.
    /// A subclass is taken directly, which is why `to_ary` is not called on
    /// one.
    pub(crate) fn as_array_operand(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Option<Rc<std::cell::RefCell<Vec<Object>>>>, MetorexError> {
        if let Object::Array(elements) = value {
            return Ok(Some(Rc::clone(elements)));
        }
        if let Some(Object::Array(elements)) =
            crate::vm::native_methods::array_subclass_value(value)
        {
            return Ok(Some(elements));
        }
        if !self.responds_to(value, "to_ary") {
            return Ok(None);
        }
        // An error raised inside `to_ary` belongs to the caller, so it travels
        // out rather than reading as an operand that is not an array.
        match self.send_to_object(value.clone(), "to_ary", vec![], position)? {
            Object::Array(elements) => Ok(Some(elements)),
            _ => Ok(None),
        }
    }

    /// How many times `"ab" * count` repeats the string: a Float loses its
    /// fraction, an object gives its `to_int`, and a number too wide to hold
    /// is a RangeError the way Ruby reports one.
    pub(crate) fn repeat_count(
        &mut self,
        count: &Object,
        position: crate::lexer::Position,
    ) -> Result<i64, MetorexError> {
        match count {
            Object::Int(number) => Ok(*number),
            Object::Float(number) => Ok(*number as i64),
            Object::BigInt(_) => Err(crate::vm::errors::simple_exception(
                "RangeError",
                "bignum too big to convert into `long'",
                position,
            )),
            other if !self.responds_to(other, "to_int") => {
                Err(crate::vm::errors::integer_conversion_error(other, position))
            }
            other => match self.send_to_object(other.clone(), "to_int", vec![], position)? {
                Object::Int(number) => Ok(number),
                Object::BigInt(_) => Err(crate::vm::errors::simple_exception(
                    "RangeError",
                    "bignum too big to convert into `long'",
                    position,
                )),
                converted => Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &format!(
                        "no implicit conversion of {} into Integer",
                        crate::vm::errors::conversion_subject(&converted)
                    ),
                    position,
                )),
            },
        }
    }
}

/// Whether two values are the same object, which is what `equal?` reports.
pub(crate) fn same_object(left: &Object, right: &Object) -> bool {
    match (left, right) {
        (Object::Instance(one), Object::Instance(other)) => Rc::ptr_eq(one, other),
        (Object::Array(one), Object::Array(other)) => Rc::ptr_eq(one, other),
        (Object::Dict(one), Object::Dict(other)) => Rc::ptr_eq(one, other),
        _ => false,
    }
}

/// The Set method an operator spells, if it names one.
pub(crate) fn set_operator_name(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add | BinaryOp::BitwiseOr => Some("union"),
        BinaryOp::Subtract => Some("difference"),
        BinaryOp::BitwiseAnd => Some("intersection"),
        BinaryOp::Xor => Some("^"),
        BinaryOp::LessEqual => Some("subset?"),
        BinaryOp::GreaterEqual => Some("superset?"),
        BinaryOp::Less => Some("proper_subset?"),
        BinaryOp::Greater => Some("proper_superset?"),
        BinaryOp::Spaceship => Some("<=>"),
        _ => None,
    }
}
