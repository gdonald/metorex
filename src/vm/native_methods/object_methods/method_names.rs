// The method names reachable on a receiver, by visibility and by
// how far up the lookup path the walk goes.

use super::*;

impl VirtualMachine {
    /// Private method names reachable on `receiver`.
    pub(crate) fn private_method_names_for(
        &self,
        receiver: &Object,
        include_super: bool,
    ) -> Vec<String> {
        self.restricted_method_names_for(receiver, include_super, Class::private_method_names)
    }

    /// Public method names reachable on `receiver`.
    pub(crate) fn public_method_names_for(
        &self,
        receiver: &Object,
        include_super: bool,
    ) -> Vec<String> {
        self.restricted_method_names_for(receiver, include_super, public_method_names)
    }

    /// Protected method names reachable on `receiver`.
    pub(crate) fn protected_method_names_for(
        &self,
        receiver: &Object,
        include_super: bool,
    ) -> Vec<String> {
        self.restricted_method_names_for(receiver, include_super, Class::protected_method_names)
    }

    /// Names of the methods `select` reports along the receiver's lookup path.
    /// The receiver's own singleton layer and its class always count;
    /// `include_super` adds the ancestors and the modules they mix in.
    fn restricted_method_names_for(
        &self,
        receiver: &Object,
        include_super: bool,
        select: fn(&Class) -> Vec<String>,
    ) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        let push = |name: String, names: &mut Vec<String>| {
            if !names.contains(&name) {
                names.push(name);
            }
        };

        // `class << obj` with `private`, and the modules `extend` attached.
        let singleton = match receiver {
            Object::Class(c) | Object::Module(c) => c.singleton_class_slot().clone(),
            Object::Instance(inst) => inst.borrow().singleton_class.borrow().clone(),
            _ => None,
        };
        if let Some(singleton_class) = singleton {
            // A class object's singleton chain stands in for its own class,
            // so it is walked even without `include_super`.
            let walk_chain = matches!(receiver, Object::Class(_) | Object::Module(_));
            let mut current = Some(singleton_class);
            while let Some(class) = current {
                for name in select(&class) {
                    push(name, &mut names);
                }
                for mixin in class.transitive_mixins() {
                    for name in select(&mixin) {
                        push(name, &mut names);
                    }
                }
                current = if walk_chain { class.superclass() } else { None };
            }
        }

        // `def self.name` lands in the class's own table under the
        // `__class__` convention. Those belong to the class object, so they
        // count here and are reported under the name Ruby shows.
        if let Object::Class(class_rc) | Object::Module(class_rc) = receiver {
            let mut current = Some(std::rc::Rc::clone(class_rc));
            while let Some(class) = current {
                for name in select(&class) {
                    if let Some(bare) = name.strip_prefix("__class__") {
                        push(bare.to_string(), &mut names);
                    }
                }
                // A class object's own methods live along this chain the way
                // an instance's live in its class, so it is walked even
                // without `include_super`, matching the singleton chain.
                current = class.superclass();
            }
        }

        let own_class = match receiver {
            Object::Instance(inst) => Some(std::rc::Rc::clone(&inst.borrow().class)),
            Object::Class(_) | Object::Module(_) => None,
            other => Some(self.builtins().class_of(other)),
        };
        if let Some(class) = own_class {
            // A class's `__class__` entries describe the class object, not
            // its instances, so they are left out here.
            let instance_level = |class: &Class| -> Vec<String> {
                select(class)
                    .into_iter()
                    .filter(|name| !name.starts_with("__class__"))
                    .collect()
            };
            for name in instance_level(&class) {
                push(name, &mut names);
            }
            if include_super {
                // A mixin is an ancestor, so its methods only count when
                // ancestors do.
                for mixin in class.transitive_mixins() {
                    for name in instance_level(&mixin) {
                        push(name, &mut names);
                    }
                }
                let mut current = class.superclass();
                while let Some(parent) = current {
                    for name in instance_level(&parent) {
                        push(name, &mut names);
                    }
                    for mixin in parent.transitive_mixins() {
                        for name in instance_level(&mixin) {
                            push(name, &mut names);
                        }
                    }
                    current = parent.superclass();
                }
            }
        }
        names
    }

    /// The method `name` names in the receiver's singleton layer: the
    /// singleton class's own table, then the modules attached to it by
    /// `include`, `prepend`, or `extend`. The object's class is not part of
    /// that layer, so a method it defines is not found here.
    pub(crate) fn singleton_layer_method(
        &self,
        receiver: &Object,
        name: &str,
    ) -> Option<(std::rc::Rc<Class>, std::rc::Rc<crate::object::Method>)> {
        // `def self.name` records the method in the class's own table under
        // the `__class__` convention rather than on the singleton class.
        if let Object::Class(class_rc) | Object::Module(class_rc) = receiver
            && let Some(found) = class_rc.find_method(&format!("__class__{}", name))
            && !found.is_undefined
        {
            return Some((std::rc::Rc::clone(class_rc), found));
        }
        let singleton = match receiver {
            Object::Class(c) | Object::Module(c) => c.singleton_class_slot().clone(),
            Object::Instance(inst) => inst.borrow().singleton_class.borrow().clone(),
            _ => None,
        }?;
        if let Some(found) = singleton.find_own_method(name)
            && !found.is_undefined
        {
            return Some((std::rc::Rc::clone(&singleton), found));
        }
        for prepended in singleton.prepend_chain() {
            if let Some(found) = prepended.find_method(name)
                && !found.is_undefined
            {
                return Some((prepended, found));
            }
        }
        for mixin in singleton.transitive_mixins() {
            if let Some(found) = mixin.find_own_method(name)
                && !found.is_undefined
            {
                return Some((mixin, found));
            }
        }
        None
    }

    /// Public method names the receiver's own singleton layer contributes:
    /// `def obj.name`, `class << obj`, `define_singleton_method`, and the
    /// modules `extend` attached. Private ones are left out, matching what
    /// `Object#methods` reports.
    pub(crate) fn singleton_layer_names(&self, receiver: &Object) -> Vec<String> {
        self.singleton_layer_names_with_mixins(receiver, true)
    }

    /// The same walk, with `include_mixins` deciding whether the modules
    /// `extend` attached count. `singleton_methods(false)` leaves them out.
    pub(crate) fn singleton_layer_names_with_mixins(
        &self,
        receiver: &Object,
        include_mixins: bool,
    ) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        // `def obj.name` records the method on the instance itself.
        if let Object::Instance(inst) = receiver {
            for name in inst.borrow().singleton_methods.borrow().keys() {
                names.push(name.clone());
            }
        }
        let singleton = match receiver {
            Object::Class(c) | Object::Module(c) => c.singleton_class_slot().clone(),
            Object::Instance(inst) => inst.borrow().singleton_class.borrow().clone(),
            _ => None,
        };
        let Some(singleton_class) = singleton else {
            return names;
        };
        for name in singleton_class.method_names() {
            // `def self.name` is recorded under the `__class__` convention,
            // and the plain name is reported for it elsewhere. Every other
            // name is one the program wrote, `__value` included.
            if !name.starts_with("__class__")
                && !singleton_class.is_method_private(&name)
                && !names.contains(&name)
            {
                names.push(name);
            }
        }
        // A tombstone `undef_method` left on the singleton class removes the
        // name from the object, however it originally arrived.
        names.retain(|name| {
            singleton_class
                .find_own_method(name)
                .is_none_or(|method| !method.is_undefined)
        });
        // `obj.extend(Mod)` mixes Mod into the singleton class.
        if !include_mixins {
            return names;
        }
        for mixin in singleton_class.transitive_mixins() {
            for name in mixin.method_names() {
                if !name.starts_with("__")
                    && !mixin.is_method_private(&name)
                    && !names.contains(&name)
                {
                    names.push(name);
                }
            }
        }
        names
    }

    /// Whether `receiver` claims `name` through `respond_to_missing?`.
    /// `include_private` is the flag Ruby passes as the second argument:
    /// `method` sends true, `public_method` sends false.
    pub(crate) fn responds_via_missing(
        &mut self,
        receiver: &Object,
        name: &str,
        include_private: bool,
        position: Position,
    ) -> Result<bool, MetorexError> {
        let Some((class, method)) = self.lookup_method(receiver, "respond_to_missing?") else {
            return Ok(false);
        };
        if method.is_undefined {
            return Ok(false);
        }
        let arguments = vec![
            Object::symbol(name.to_string()),
            Object::Bool(include_private),
        ];
        let answer = self.invoke_method(class, method, receiver.clone(), arguments, position)?;
        Ok(!matches!(answer, Object::Nil | Object::Bool(false)))
    }
}
