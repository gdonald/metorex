// The instance methods a module holds.

use super::*;

impl VirtualMachine {
    /// The instance methods a module holds, as names and as unbound methods.
    pub(crate) fn call_instance_method_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        match method_name {
            // Module#instance_method / Class#instance_method: returns the
            // bound `Method` object so `parameters` and friends work on it.
            "instance_method" | "public_instance_method" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name_str = self.coerce_method_name(&arguments[0], method_name, position)?;
                // A refinement in force here is what the name stands for
                // while it lasts, so the unbound method answers to it.
                let refined = crate::vm::method_lookup::refinement_target_name(
                    &Object::Class(Rc::clone(class_rc)),
                    self,
                )
                .or_else(|| {
                    Some(format!(
                        "__refine__{}@{:p}",
                        class_rc.name(),
                        Rc::as_ptr(class_rc)
                    ))
                })
                .and_then(|target| self.find_refined_method(&target, &name_str));
                if let Some(refined) = refined {
                    // The refinement the method was written in owns it.
                    let mut unbound = (*refined).clone();
                    if unbound.owner_class.is_none() {
                        unbound.owner = Some(class_rc.name().to_string());
                        unbound.owner_class = Some(Rc::clone(class_rc));
                    }
                    unbound.origin_class = Some(Rc::clone(class_rc));
                    return Ok(Answered(Object::Method(Rc::new(unbound))));
                }
                if let Some((owner, method)) = class_rc.find_method_with_owner(&name_str) {
                    // `public_instance_method` only hands out public methods.
                    if method.is_undefined
                        || (method_name == "public_instance_method"
                            && (owner.is_method_restricted(&name_str)
                                || self.method_is_restricted(
                                    &Object::Class(Rc::clone(class_rc)),
                                    &name_str,
                                )))
                    {
                        return Err(undefined_instance_method_error(
                            &name_str, class_rc, position,
                        ));
                    }
                    let mut unbound = (*method).clone();
                    // A `def self.name` method records the class it was
                    // written in, while the singleton class is what owns it.
                    if unbound.owner_class.is_none() || owner.is_singleton_class() {
                        unbound.owner = Some(owner.name().to_string());
                        unbound.owner_class = Some(owner);
                    }
                    unbound.origin_class = Some(Rc::clone(class_rc));
                    return Ok(Answered(Object::Method(Rc::new(unbound))));
                }
                // Synthesize a stub for well-known Module-private mixin
                // hooks so `Module.instance_method(:append_features)` works
                // for spec patterns that bind/call them.
                if class_rc.name() == "Module" && MODULE_PRIVATE_HOOKS.contains(&name_str.as_str())
                {
                    let stub = Method::with_owner(
                        name_str.clone(),
                        vec!["target".to_string()],
                        vec![],
                        "Module".to_string(),
                    );
                    return Ok(Answered(Object::Method(Rc::new(stub))));
                }
                // The rest of Module's own methods are implemented natively,
                // so hand out a stub carrying the right parameter list.
                if matches!(class_rc.name(), "Module" | "Class")
                    && let Some(stub) = native_module_method_stub(&name_str)
                {
                    return Ok(Answered(Object::Method(Rc::new(stub))));
                }
                // `Class#new` is answered natively too, and belongs to Class
                // rather than to Module.
                if class_rc.name() == "Class" && name_str == "new" {
                    let mut stub = Method::with_owner(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                        "Class".to_string(),
                    );
                    stub.variadic_param = Some((0, "args".to_string()));
                    stub.native_alias = Some(name_str.clone());
                    stub.original_name = Some(name_str.clone());
                    stub.owner_class = Some(Rc::clone(class_rc));
                    return Ok(Answered(Object::Method(Rc::new(stub))));
                }
                // Kernel methods are implemented natively rather than living
                // in Object's method table. A body-less stub reaches the same
                // native implementation when invoked, so `Object` can hand out
                // an UnboundMethod for them.
                if matches!(class_rc.name(), "Object" | "Kernel")
                    && is_native_kernel_method(&name_str)
                {
                    let mut stub = Method::with_owner(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                        "Kernel".to_string(),
                    );
                    stub.variadic_param = Some((0, "args".to_string()));
                    return Ok(Answered(Object::Method(Rc::new(stub))));
                }
                // BasicObject answers its own handful natively rather than
                // holding them in a method table, so a body-less stub stands
                // for each of them.
                if class_rc.name() == "BasicObject"
                    && (NATIVE_BASIC_OBJECT_METHODS.contains(&name_str.as_str())
                        || BASIC_OBJECT_PRIVATE_METHODS.contains(&name_str.as_str()))
                {
                    let mut stub = Method::with_owner(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                        "BasicObject".to_string(),
                    );
                    stub.variadic_param = Some((0, "args".to_string()));
                    stub.native_alias = Some(name_str.clone());
                    stub.original_name = Some(name_str.clone());
                    stub.owner_class = Some(Rc::clone(class_rc));
                    return Ok(Answered(Object::Method(Rc::new(stub))));
                }
                // A builtin class answers many of its methods natively. A
                // body-less stub carrying the name reaches the same one.
                if let Some(probe) = sample_of_class(class_rc.name())
                    && self.responds_to(&probe, &name_str)
                {
                    let mut stub = Method::with_owner(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                        class_rc.name().to_string(),
                    );
                    stub.variadic_param = Some((0, "args".to_string()));
                    // Two names for one native method stand for that one
                    // method, which is what makes them equal and alike.
                    let named = crate::vm::native_methods::object_methods::native_alias_target(
                        class_rc.name(),
                        name_str.as_str(),
                    )
                    .unwrap_or(name_str.as_str())
                    .to_string();
                    stub.native_alias = Some(named.clone());
                    stub.original_name = Some(named);
                    return Ok(Answered(Object::Method(Rc::new(stub))));
                }
                // An exception class takes its message, and whatever else a
                // subclass reads, through a native `initialize`. A body-less
                // stub carries the variadic arity Ruby reports for it.
                if name_str == "initialize"
                    && crate::vm::method_invocation::descends_from(class_rc, "Exception")
                {
                    let mut stub = Method::with_owner(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                        class_rc.name().to_string(),
                    );
                    stub.variadic_param = Some((0, "args".to_string()));
                    return Ok(Answered(Object::Method(Rc::new(stub))));
                }
                return Err(undefined_instance_method_error(
                    &name_str, class_rc, position,
                ));
            }
            // Module#undefined_instance_methods: the names this class itself
            // has undefined with `undef_method`, not those its ancestors did.
            "undefined_instance_methods" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let undefined: Vec<Object> = class_rc
                    .method_names()
                    .into_iter()
                    .filter(|name| {
                        class_rc
                            .find_own_method(name)
                            .is_some_and(|method| method.is_undefined)
                    })
                    .map(Object::symbol)
                    .collect();
                return Ok(Answered(Object::Array(Rc::new(std::cell::RefCell::new(
                    undefined,
                )))));
            }
            // Module#instance_methods / public_/private_/protected_ variants.
            // The `false` argument restricts to methods defined directly on
            // this class (excluding inherited and mixin methods).
            "instance_methods"
            | "public_instance_methods"
            | "private_instance_methods"
            | "protected_instance_methods" => {
                let include_super = match arguments.first() {
                    Some(Object::Bool(b)) => *b,
                    _ => true,
                };
                // A refinement's body names the class it refines, and it
                // answers to everything that class answers to as well as to
                // its own names, so the class is asked the same question and
                // what it says is listed behind them.
                if let Some(Object::Class(refined) | Object::Module(refined)) = class_rc
                    .get_class_var(crate::vm::native_methods::module_methods::REFINEMENT_TARGET_KEY)
                {
                    let mut named: Vec<Object> = class_rc
                        .method_names()
                        .into_iter()
                        .map(Object::symbol)
                        .collect();
                    if let Some(Object::Array(held)) =
                        self.call_class_methods(&refined, method_name, arguments, position)?
                    {
                        for one in held.borrow().iter() {
                            if !named.contains(one) {
                                named.push(one.clone());
                            }
                        }
                    }
                    return Ok(Answered(Object::array(named)));
                }
                let mut method_list: Vec<String> = class_rc.method_names();
                // A `private`/`public` naming an inherited method marks the
                // visibility here without defining anything, and Ruby counts
                // that name among this class's own methods.
                let object_class = match self.globals().get("Object") {
                    Some(Object::Class(object_class)) => Some(object_class),
                    _ => None,
                };
                for name in class_rc.visibility_marked_names() {
                    let resolves = class_rc.find_method(&name).is_some()
                        || object_class
                            .as_ref()
                            .is_some_and(|oc| oc.find_method(&name).is_some());
                    if !method_list.contains(&name) && resolves {
                        method_list.push(name);
                    }
                }
                if include_super {
                    // The ancestor walk already covers mixins of mixins and
                    // each superclass's mixins.
                    let mut chain: Vec<Object> = Vec::new();
                    let mut seen: Vec<*const Class> = Vec::new();
                    push_class_ancestors(class_rc, &mut chain, &mut seen);
                    for ancestor in &chain {
                        let (Object::Class(c) | Object::Module(c)) = ancestor else {
                            continue;
                        };
                        for n in c.method_names() {
                            if !method_list.contains(&n) {
                                method_list.push(n);
                            }
                        }
                    }
                }
                // For the `Module` / `Class` receiver, advertise the native
                // mutation methods we actually implement so mspec matchers
                // (e.g. `have_public_instance_method(:alias_method, false)`)
                // recognize them as public instance methods.
                if matches!(class_rc.name(), "Module" | "Class") {
                    for (n, _, _) in NATIVE_MODULE_METHODS {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                    }
                }
                // A method removed with `undef_method` stays in the table as
                // a tombstone so lookups stop at it; it is not an instance
                // method any more.
                method_list.retain(|n| {
                    class_rc
                        .find_method(n)
                        .is_none_or(|method| !method.is_undefined)
                });
                // A name's visibility comes from the nearest ancestor that
                // defines it: an ancestor further along may mark its own copy
                // private without that reaching the one in front.
                let mut priv_set: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                let mut protected_set: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                // Module-private mixin hooks: append_features and friends
                // are private instance methods on Module. Class is *also* a
                // Module subclass — but `append_features` is undefined on
                // Class (Ruby sets it to undef), so only surface them when
                // the receiver is Module itself.
                if class_rc.name() == "Module" {
                    for n in MODULE_PRIVATE_HOOKS
                        .iter()
                        .chain(MODULE_PRIVATE_DECLARATIONS.iter())
                    {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                        priv_set.insert((*n).to_string());
                    }
                }
                // BasicObject's own methods are native too, and its table is
                // empty, so they are listed here the way Kernel's are.
                if class_rc.name() == "BasicObject" {
                    for n in NATIVE_BASIC_OBJECT_METHODS.iter() {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                    }
                    for n in BASIC_OBJECT_PRIVATE_METHODS.iter() {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                        priv_set.insert((*n).to_string());
                    }
                }
                // Kernel's methods are implemented natively rather than in
                // its table, so they are listed here. The public ones come
                // first; the pass below marks the private ones.
                if class_rc.name() == "Kernel" {
                    for (n, _, _) in NATIVE_KERNEL_METHODS.iter() {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                    }
                    for n in
                        crate::vm::native_methods::kernel_conversion::KERNEL_CONVERSION_FUNCTIONS
                            .iter()
                            .chain(KERNEL_PRIVATE_FUNCTIONS.iter())
                    {
                        if !method_list.iter().any(|m| m == n) {
                            method_list.push((*n).to_string());
                        }
                        priv_set.insert((*n).to_string());
                    }
                }
                let visibility_chain: Vec<Rc<Class>> = if include_super {
                    let mut chain: Vec<Object> = Vec::new();
                    let mut seen: Vec<*const Class> = Vec::new();
                    push_class_ancestors(class_rc, &mut chain, &mut seen);
                    chain
                        .iter()
                        .filter_map(|ancestor| match ancestor {
                            Object::Class(c) | Object::Module(c) => Some(Rc::clone(c)),
                            _ => None,
                        })
                        .collect()
                } else {
                    vec![Rc::clone(class_rc)]
                };
                for name in &method_list {
                    for ancestor in &visibility_chain {
                        if ancestor.has_public_override(name) {
                            break;
                        }
                        if ancestor.is_method_private(name) {
                            priv_set.insert(name.clone());
                            break;
                        }
                        if ancestor.is_method_protected(name) {
                            protected_set.insert(name.clone());
                            break;
                        }
                        if ancestor.find_own_method(name).is_some() {
                            break;
                        }
                    }
                }
                let filtered: Vec<Object> = method_list
                    .into_iter()
                    .filter(|n| !is_internal_method_key(n))
                    .filter(|n| match method_name {
                        "private_instance_methods" => priv_set.contains(n),
                        "protected_instance_methods" => protected_set.contains(n),
                        "public_instance_methods" => {
                            !priv_set.contains(n) && !protected_set.contains(n)
                        }
                        // Ruby's `instance_methods` covers public and
                        // protected alike.
                        "instance_methods" => !priv_set.contains(n),
                        _ => true,
                    })
                    .map(Object::symbol)
                    .collect();
                return Ok(Answered(Object::Array(Rc::new(std::cell::RefCell::new(
                    filtered,
                )))));
            }
            _ => {}
        }
        Ok(Unclaimed)
    }
}
