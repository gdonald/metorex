// How two values order against each other.

use super::*;

impl VirtualMachine {
    /// Evaluate comparison operations on numeric operands.
    pub(crate) fn evaluate_comparison(
        &mut self,
        op: &BinaryOp,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if matches!(left, Object::Class(_) | Object::Module(_)) {
            return self.evaluate_module_comparison(op, left, right, position);
        }

        // Two exact integers compare exactly, without the rounding a Float
        // conversion would introduce at large magnitudes.
        if matches!(left, Object::Int(_) | Object::BigInt(_))
            && matches!(right, Object::Int(_) | Object::BigInt(_))
        {
            let a = left.as_big_integer().expect("integer-kinded");
            let b = right.as_big_integer().expect("integer-kinded");
            let result = match op {
                BinaryOp::Less => a < b,
                BinaryOp::Greater => a > b,
                BinaryOp::LessEqual => a <= b,
                BinaryOp::GreaterEqual => a >= b,
                _ => unreachable!("caller restricts op to the comparison set"),
            };
            return Ok(Object::Bool(result));
        }

        // A bignum against a Float is ordered exactly, since rounding the
        // bignum to a Float would lose the digits the answer turns on.
        let exact = match (&left, &right) {
            (Object::BigInt(value), Object::Float(float)) => {
                compare_integer_to_float(value, *float)
            }
            (Object::Float(float), Object::BigInt(value)) => {
                compare_integer_to_float(value, *float).map(|order| order.reverse())
            }
            _ => None,
        };
        if matches!(
            (&left, &right),
            (Object::BigInt(_), Object::Float(_)) | (Object::Float(_), Object::BigInt(_))
        ) {
            use std::cmp::Ordering;
            // Nothing compares against NaN, which every ordering reports as
            // false rather than as a failure.
            let result = match exact {
                None => false,
                Some(order) => match op {
                    BinaryOp::Less => order == Ordering::Less,
                    BinaryOp::Greater => order == Ordering::Greater,
                    BinaryOp::LessEqual => order != Ordering::Greater,
                    BinaryOp::GreaterEqual => order != Ordering::Less,
                    _ => unreachable!("caller restricts op to the comparison set"),
                },
            };
            return Ok(Object::Bool(result));
        }

        // Numeric comparisons
        let numeric_result = match (&left, &right) {
            (Object::Int(a), Object::Int(b)) => Some((*a as f64, *b as f64)),
            (Object::Float(a), Object::Float(b)) => Some((*a, *b)),
            (Object::Int(a), Object::Float(b)) => Some((*a as f64, *b)),
            (Object::Float(a), Object::Int(b)) => Some((*a, *b as f64)),
            // String comparison
            (Object::String(a), Object::String(b)) => {
                let result = match op {
                    BinaryOp::Less => **a < **b,
                    BinaryOp::Greater => **a > **b,
                    BinaryOp::LessEqual => **a <= **b,
                    BinaryOp::GreaterEqual => **a >= **b,
                    _ => unreachable!(),
                };
                return Ok(Object::Bool(result));
            }
            _ => None,
        };

        if let Some((lhs, rhs)) = numeric_result {
            let result = match op {
                BinaryOp::Less => lhs < rhs,
                BinaryOp::Greater => lhs > rhs,
                BinaryOp::LessEqual => lhs <= rhs,
                BinaryOp::GreaterEqual => lhs >= rhs,
                _ => unreachable!(),
            };
            return Ok(Object::Bool(result));
        }

        // A user-defined comparison operator wins over the Comparable
        // protocol, the way `==` already dispatches to its own definition.
        if matches!(left, Object::Instance(_))
            && let Some(operator) = comparison_operator_name(op)
            && let Some((class, method)) = self.lookup_method(&left, operator)
            && !method.is_undefined
        {
            return self.invoke_method(class, method, left, vec![right], position);
        }

        // An object with neither the operator nor `<=>` still gets the call
        // offered to `method_missing`, as any other missing method would.
        if matches!(left, Object::Instance(_))
            && let Some(operator) = comparison_operator_name(op)
            && self.lookup_method(&left, "<=>").is_none()
            && let Some((class, method)) = self.lookup_method(&left, "method_missing")
        {
            let name = Object::symbol(operator.to_string());
            return self.invoke_method(class, method, left, vec![name, right], position);
        }

        // Object defines no comparison operators; Comparable adds them on top
        // of `<=>`. An object of the program's own that is not Comparable has
        // no such method at all.
        if matches!(left, Object::Instance(_))
            && let Some(operator) = comparison_operator_name(op)
            && !self.is_comparable(&left)
        {
            let wording = self.receiver_wording_for(&left, position);
            return Err(crate::vm::errors::undefined_method_error_worded(
                operator,
                &left,
                std::slice::from_ref(&right),
                wording,
                position,
            ));
        }

        // For instances, try dispatching to <=> method (Comparable protocol)
        if let Some((class, method)) = self.lookup_method(&left, "<=>") {
            let left_type = self.class_name_for_comparison(&left, position);
            let right_type = self.comparison_subject(&right, position);
            let cmp_result = self.invoke_method(class, method, left, vec![right], position)?;
            let cmp_value: Option<f64> = match &cmp_result {
                Object::Int(n) => Some(*n as f64),
                Object::Float(f) => Some(*f),
                _ => None,
            };
            if let Some(n) = cmp_value {
                let result = match op {
                    BinaryOp::Less => n < 0.0,
                    BinaryOp::Greater => n > 0.0,
                    BinaryOp::LessEqual => n <= 0.0,
                    BinaryOp::GreaterEqual => n >= 0.0,
                    _ => unreachable!(),
                };
                return Ok(Object::Bool(result));
            }
            // nil or other: raise ArgumentError
            let exc = Object::exception(
                "ArgumentError",
                format!("comparison of {} with {} failed", left_type, right_type),
            );
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: "comparison failed".to_string(),
            });
        }

        // Comparison type mismatch is ArgumentError in Ruby, not TypeError.
        // Ruby's format: "comparison of <LeftClass> with <right_value> failed"
        let right_repr = match &right {
            Object::Int(n) => n.to_string(),
            Object::Float(f) => f.to_string(),
            Object::Nil => "nil".to_string(),
            Object::Bool(true) => "true".to_string(),
            Object::Bool(false) => "false".to_string(),
            _ => right.type_name().to_string(),
        };
        let msg = format!(
            "comparison of {} with {} failed",
            left.type_name(),
            right_repr
        );
        Err(MetorexError::UncaughtException {
            exception: Object::exception("ArgumentError", msg.clone()),
            location: position_to_location(position),
            message: msg,
        })
    }

    /// Order two arrays element by element, with the shorter one first when
    /// every shared element is equal. A pair already being ordered further up
    /// the stack is taken as equal, so two arrays that reach themselves answer
    /// rather than recursing forever.
    pub(crate) fn order_arrays(
        &mut self,
        left: &Rc<std::cell::RefCell<Vec<Object>>>,
        right: &Rc<std::cell::RefCell<Vec<Object>>>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let pair = (Rc::as_ptr(left) as usize, Rc::as_ptr(right) as usize);
        if pair.0 == pair.1 {
            return Ok(Object::Int(0));
        }
        let entered = ORDERING.with(|active| {
            let mut active = active.borrow_mut();
            if active.contains(&pair) {
                return false;
            }
            active.push(pair);
            true
        });
        if !entered {
            return Ok(Object::Int(0));
        }
        let outcome = self.order_array_elements(left, right, position);
        ORDERING.with(|active| {
            active.borrow_mut().pop();
        });
        outcome
    }

    pub(crate) fn order_array_elements(
        &mut self,
        left: &Rc<std::cell::RefCell<Vec<Object>>>,
        right: &Rc<std::cell::RefCell<Vec<Object>>>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let left_elements = left.borrow().clone();
        let right_elements = right.borrow().clone();
        for (one, other) in left_elements.iter().zip(right_elements.iter()) {
            let order = self.evaluate_binary_operation(
                &BinaryOp::Spaceship,
                one.clone(),
                other.clone(),
                position,
            )?;
            // The first result that is not zero is what the whole comparison
            // answers, whatever kind of value it is.
            match order {
                Object::Int(0) => continue,
                other => return Ok(other),
            }
        }
        Ok(Object::Int(
            left_elements.len().cmp(&right_elements.len()) as i64
        ))
    }

    /// Evaluate the spaceship operator (<=>), returning -1, 0, or 1.
    pub(crate) fn evaluate_spaceship(
        &self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Two exact integers order exactly, whatever their magnitude.
        if matches!(left, Object::Int(_) | Object::BigInt(_))
            && matches!(right, Object::Int(_) | Object::BigInt(_))
        {
            let a = left.as_big_integer().expect("integer-kinded");
            let b = right.as_big_integer().expect("integer-kinded");
            return Ok(Object::Int(a.cmp(&b) as i64));
        }
        // Nothing compares against a value that is not a number, which is
        // what a NaN on either side makes the answer.
        if matches!(&left, Object::Float(held) if held.is_nan())
            || matches!(&right, Object::Float(held) if held.is_nan())
        {
            return Ok(Object::Nil);
        }
        match (&left, &right) {
            (Object::Int(a), Object::Int(b)) => Ok(Object::Int(a.cmp(b) as i64)),
            // A bignum carries more digits than a Float has, so the two are
            // ordered exactly rather than by rounding the bignum.
            (Object::BigInt(a), Object::Float(b)) => {
                Ok(compare_or_nil(compare_integer_to_float(a, *b)))
            }
            (Object::Float(a), Object::BigInt(b)) => Ok(compare_or_nil(
                compare_integer_to_float(b, *a).map(std::cmp::Ordering::reverse),
            )),
            (Object::Float(a), Object::Float(b)) => Ok(compare_or_nil(a.partial_cmp(b))),
            (Object::Int(a), Object::Float(b)) => {
                let a = *a as f64;
                Ok(compare_or_nil(a.partial_cmp(b)))
            }
            (Object::Float(a), Object::Int(b)) => {
                let b = *b as f64;
                Ok(Object::Int(a.partial_cmp(&b).map_or(0, |o| o as i64)))
            }
            (Object::String(a), Object::String(b)) => Ok(Object::Int(a.cmp(b) as i64)),
            // A Symbol orders by its name, the way its String does.
            (Object::Symbol(a), Object::Symbol(b)) => Ok(Object::Int(a.cmp(b) as i64)),
            // Module#<=>: compares the ancestry relationship of two modules or
            // classes. -1 when the left is a descendant/includer of the right,
            // +1 when it's an ancestor/included-by, 0 when they're the same,
            // and nil when they're unrelated.
            (Object::Class(a) | Object::Module(a), Object::Class(b) | Object::Module(b)) => {
                let left_below_right = self.builtins().is_subclass_of(a, b);
                let right_below_left = self.builtins().is_subclass_of(b, a);
                Ok(match (left_below_right, right_below_left) {
                    (true, true) => Object::Int(0),
                    (true, false) => Object::Int(-1),
                    (false, true) => Object::Int(1),
                    (false, false) => Object::Nil,
                })
            }
            // Module#<=> against a non-module argument returns nil rather than
            // raising.
            (Object::Class(_) | Object::Module(_), _) => Ok(Object::Nil),
            // Two values of unrelated kinds have no ordering, and Ruby reports
            // that as nil rather than as an error. `sort` and friends turn it
            // into the ArgumentError a caller sees.
            _ => {
                let _ = position;
                Ok(Object::Nil)
            }
        }
    }

    /// How a String orders against another object, or None when the ordering
    /// is left to the general rules.
    fn order_string_against(
        &mut self,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::String(text) = left else {
            return Ok(None);
        };
        // An instance of a String subclass orders by the characters behind it,
        // so a subclass compares equal to the string it was built from.
        let other = match right {
            Object::String(other) => Some(Rc::clone(other)),
            held => match crate::vm::native_methods::string_subclass_value(held) {
                Some(Object::String(other)) => Some(other),
                _ => None,
            },
        };
        if let Some(other) = other {
            return Ok(Some(order_two_strings(text, &other)));
        }
        // Anything that reads as a String is compared as the String it reads
        // as, which is what `to_str` answers.
        if self.responds_to(right, "to_str") {
            let read = self.send_to_object(right.clone(), "to_str", vec![], position)?;
            if let Object::String(other) = read {
                return Ok(Some(order_two_strings(text, &other)));
            }
            return Ok(Some(Object::Nil));
        }
        // Anything that orders itself is asked the other way round, and the
        // answer is turned around to match.
        if !self.responds_to(right, "<=>") {
            return Ok(Some(Object::Nil));
        }
        let pair = (object_identity(left), object_identity(right));
        if INVERSE_COMPARISONS.with(|held| held.borrow().contains(&pair)) {
            return Ok(Some(Object::Nil));
        }
        INVERSE_COMPARISONS.with(|held| held.borrow_mut().push(pair));
        let answered = self.send_to_object(right.clone(), "<=>", vec![left.clone()], position);
        INVERSE_COMPARISONS.with(|held| {
            held.borrow_mut().pop();
        });
        Ok(Some(match answered? {
            Object::Int(order) => Object::Int(-order),
            _ => Object::Nil,
        }))
    }

    /// How two values order against each other, or nil where they do not.
    pub(crate) fn evaluate_ordering(
        &mut self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        use BinaryOp::*;
        // Object#<=> answers 0 for the same object or when #== says
        // so, and nil otherwise. It never consults #eql?. A class that
        // defines its own #<=> is dispatched before reaching here.
        if let Object::Instance(inst_rc) = &left {
            // A class of the program's own answers for itself, which
            // is what a walk over elements reaches here.
            if let Some((class, method)) = self.lookup_method(&left, "<=>")
                && !method.is_undefined
                && !method.body.is_empty()
            {
                return self.invoke_method(class, method, left.clone(), vec![right], position);
            }
            if let Object::Instance(rhs) = &right
                && Rc::ptr_eq(inst_rc, rhs)
            {
                return Ok(Object::Int(0));
            }
            let equal = self.evaluate_binary_operation(&Equal, left, right.clone(), position)?;
            return Ok(if equal.is_truthy() {
                Object::Int(0)
            } else {
                Object::Nil
            });
        }
        // Two arrays order element by element, and each element is
        // sent `<=>` so one of the program's own answers for itself.
        if let Object::Array(left_elements) = &left {
            let left_elements = Rc::clone(left_elements);
            if let Some(right_elements) = self.as_array_operand(&right, position)? {
                return self.order_arrays(&left_elements, &right_elements, position);
            }
            return Ok(Object::Nil);
        }
        // An endless Float orders against anything that reports
        // which end it stands at, and against a finite value it is
        // simply greater.
        if let Object::Float(held) = &left
            && held.is_infinite()
            && matches!(right, Object::Instance(_))
            && self.responds_to(&right, "infinite?")
        {
            let reported = self.send_to_object(right.clone(), "infinite?", Vec::new(), position)?;
            let theirs = match reported {
                Object::Int(held) => held,
                _ => 0,
            };
            let mine = if *held > 0.0 { 1 } else { -1 };
            return Ok(Object::Int(mine.cmp(&theirs) as i64));
        }
        // A String orders against anything that reads as one, then
        // against anything that orders itself, and has no order at
        // all with anything else.
        if matches!(&left, Object::String(_))
            && let Some(order) = self.order_string_against(&left, &right, position)?
        {
            return Ok(order);
        }
        let ordered = self.evaluate_spaceship(left.clone(), right.clone(), position)?;
        if !matches!(ordered, Object::Nil) {
            return Ok(ordered);
        }
        // Object#<=> answers 0 for two values `==` calls the same and
        // nil for everything else, which is what two of a kind with no
        // order of their own fall back on. A value that orders itself
        // has already answered, and asking it again would run an `==`
        // written for an operand of another kind.
        if is_number(&left)
            || matches!(
                left,
                Object::String(_) | Object::Symbol(_) | Object::Array(_)
            )
        {
            return Ok(Object::Nil);
        }
        let equal = self.evaluate_binary_operation(&Equal, left, right, position)?;
        Ok(if equal.is_truthy() {
            Object::Int(0)
        } else {
            Object::Nil
        })
    }
}

thread_local! {
    // The pairs of objects an inverse comparison is already running for, so a
    // `<=>` that turns around and asks the other object back is answered with
    // no order at all rather than running forever.
    static INVERSE_COMPARISONS: std::cell::RefCell<Vec<(usize, usize)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// The address an object is known by while an inverse comparison runs. A value
/// with no address of its own stands for itself.
fn object_identity(held: &Object) -> usize {
    match held {
        Object::String(text) => Rc::as_ptr(text) as usize,
        Object::Instance(instance) => Rc::as_ptr(instance) as usize,
        Object::Array(elements) => Rc::as_ptr(elements) as usize,
        Object::Dict(entries) => Rc::as_ptr(entries) as usize,
        _ => 0,
    }
}

/// How one string orders against another. Ruby compares the bytes, and two
/// strings whose bytes are the same but whose encodings are not are ordered by
/// where those encodings sit in the list Ruby keeps.
fn order_two_strings(
    left: &Rc<crate::object::StringValue>,
    right: &Rc<crate::object::StringValue>,
) -> Object {
    let left_bytes = crate::vm::native_methods::string_methods::binary_bytes(left);
    let right_bytes = crate::vm::native_methods::string_methods::binary_bytes(right);
    match left_bytes.cmp(&right_bytes) {
        std::cmp::Ordering::Less => return Object::Int(-1),
        std::cmp::Ordering::Greater => return Object::Int(1),
        std::cmp::Ordering::Equal => (),
    }
    let (held, theirs) = (left.encoding_name(), right.encoding_name());
    // Text that is nothing but ASCII reads the same under either encoding, so
    // the two are equal whatever they are tagged with.
    if held == theirs || left_bytes.is_ascii() {
        return Object::Int(0);
    }
    let place = |named: &str| {
        crate::vm::init::ENCODING_NAMES
            .iter()
            .position(|(_, canonical, _)| *canonical == named)
            .unwrap_or(usize::MAX)
    };
    Object::Int(place(&held).cmp(&place(&theirs)) as i64)
}

/// The Ruby method name for an ordering operator.
fn comparison_operator_name(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Less => Some("<"),
        BinaryOp::Greater => Some(">"),
        BinaryOp::LessEqual => Some("<="),
        BinaryOp::GreaterEqual => Some(">="),
        _ => None,
    }
}

/// Order an arbitrary-width integer against a Float without rounding the
/// integer, which a bignum does not survive. The Float is split at the decimal
/// point so its whole part is compared exactly and its fraction breaks a tie.
fn compare_integer_to_float(value: &num_bigint::BigInt, float: f64) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering;
    if float.is_nan() {
        return None;
    }
    if float == f64::INFINITY {
        return Some(Ordering::Less);
    }
    if float == f64::NEG_INFINITY {
        return Some(Ordering::Greater);
    }
    let whole = float.trunc();
    let whole: num_bigint::BigInt = format!("{:.0}", whole).parse().ok()?;
    match value.cmp(&whole) {
        Ordering::Equal => (0.0).partial_cmp(&(float - float.trunc())),
        other => Some(other),
    }
}

/// One ordering as `<=>` reports it, where an absent ordering is nil.
fn compare_or_nil(order: Option<std::cmp::Ordering>) -> Object {
    match order {
        Some(order) => Object::Int(order as i64),
        None => Object::Nil,
    }
}

thread_local! {
    /// Array pairs currently being ordered, so a pair that reaches itself
    /// answers 0 instead of recursing forever.
    static ORDERING: std::cell::RefCell<Vec<(usize, usize)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

impl VirtualMachine {
    /// Whether the object is a kind of Comparable, through its class or a
    /// module it was extended with.
    pub(crate) fn is_comparable(&self, object: &Object) -> bool {
        let Some(Object::Module(comparable)) = self.globals().get("Comparable") else {
            return false;
        };
        self.builtins().is_instance_of(object, &comparable)
    }

    /// The name of the object's class, as a failed comparison names it.
    fn class_name_for_comparison(&mut self, object: &Object, position: Position) -> String {
        match self.send_to_object(object.clone(), "class", Vec::new(), position) {
            Ok(class) => class.to_string(),
            Err(_) => object.type_name().to_string(),
        }
    }

    /// How a failed comparison names the other side, as MRI's `rb_cmperr`
    /// does: an immediate value or a Float by its `inspect`, anything else by
    /// its class.
    fn comparison_subject(&mut self, object: &Object, position: Position) -> String {
        match object {
            Object::Nil
            | Object::Bool(_)
            | Object::Int(_)
            | Object::Symbol(_)
            | Object::Float(_) => {
                match self.send_to_object(object.clone(), "inspect", Vec::new(), position) {
                    Ok(Object::String(text)) => text.as_str().to_string(),
                    _ => object.to_string(),
                }
            }
            other => self.class_name_for_comparison(other, position),
        }
    }
}
