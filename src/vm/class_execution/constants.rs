// Finding the constant a name stands for, from the scope the name
// was written in outwards.

use super::*;

impl VirtualMachine {
    /// Whether an unqualified constant may fall through to the top level.
    /// Ruby reaches top-level constants because they are Object's, so a class
    /// body whose class does not descend from Object never sees them.
    pub(crate) fn lexical_scope_reaches_top_level(&self) -> bool {
        let innermost = self.def_scope_stack.last().cloned().or_else(|| {
            // Inside a method body the lexical chain is the nesting captured
            // where the method was defined, innermost first.
            self.method_nesting_stack
                .last()
                .and_then(|nesting| nesting.first())
                .cloned()
        });
        let Some(innermost) = innermost else {
            return true;
        };
        // A module has no superclass chain to judge by, so it stays open.
        let mut cursor = Some(innermost);
        while let Some(current) = cursor {
            if current.name() == "Object" {
                return true;
            }
            if current.name() == "BasicObject" {
                return false;
            }
            cursor = current.superclass();
        }
        true
    }

    /// Resolve a bare constant name using Ruby-like lexical scoping: walk
    /// the enclosing class/module stack (innermost first), then check the
    /// local environment, then globals. Used by `include`/`extend` inside
    /// class bodies so nested modules can reference siblings without
    /// fully-qualifying them. Also handles qualified names (`A::B::C`).
    pub(crate) fn resolve_constant_in_scope(&self, name: &str) -> Option<Object> {
        if name.contains("::") {
            let mut parts = name.split("::");
            let head = parts.next()?;
            let mut current = self.resolve_constant_in_scope(head)?;
            for part in parts {
                let class_rc = match &current {
                    Object::Class(c) | Object::Module(c) => Rc::clone(c),
                    _ => return None,
                };
                current = class_rc.get_class_var(part)?;
            }
            return Some(current);
        }
        for enclosing in self.def_scope_stack.iter().rev() {
            if let Some(val) = enclosing.get_class_var(name) {
                return Some(val);
            }
            // A still-being-defined enclosing scope resolves to itself when
            // referenced by its own simple name (e.g. `ModuleSpecs::Alias`
            // inside `module ModuleSpecs; class Allonym; include ... end; end`
            // before ModuleSpecs has been bound in globals).
            if enclosing.name() == name {
                return Some(scope_object(enclosing));
            }
        }
        // Inside a method body the lexical chain is the nesting captured
        // where the method was written, innermost first, which is what lets a
        // name written there reach a sibling of the class holding it.
        if let Some(nesting) = self.method_nesting_stack.last() {
            for enclosing in nesting {
                if let Some(value) = enclosing.get_class_var(name) {
                    return Some(value);
                }
            }
        }
        self.environment()
            .get(name)
            .or_else(|| self.globals().constant(name))
    }

    /// Like `resolve_constant_in_scope`, but triggers autoload on any
    /// segment along a qualified `A::B::C` walk where the next part is only
    /// registered as an autoload. Used by class/module statement positions
    /// that should fire pending autoloads (e.g. `class X < A::B::Auto`).
    pub(crate) fn resolve_constant_with_autoload(
        &mut self,
        name: &str,
    ) -> Result<Option<Object>, MetorexError> {
        if name.contains("::") {
            let mut parts = name.split("::");
            let head = match parts.next() {
                Some(h) => h,
                None => return Ok(None),
            };
            let mut current = match self.resolve_constant_with_autoload(head)? {
                Some(v) => v,
                None => return Ok(None),
            };
            for part in parts {
                let class_rc = match &current {
                    Object::Class(c) | Object::Module(c) => Rc::clone(c),
                    _ => return Ok(None),
                };
                if let Some(v) = class_rc.get_class_var(part) {
                    current = v;
                } else if let Some(v) = self.try_autoload_constant(&class_rc, part)? {
                    current = v;
                } else {
                    return Ok(None);
                }
            }
            return Ok(Some(current));
        }
        Ok(self.resolve_constant_in_scope(name))
    }

    /// Resolve a `Foo::Bar::Baz` qualified constant by walking from the
    /// outermost name through nested module/class constants.
    pub(crate) fn resolve_qualified_constant(&self, qualified: &str) -> Option<Object> {
        let mut parts = qualified.split("::");
        let head = parts.next()?;
        let mut current = self
            .environment()
            .get(head)
            .or_else(|| self.globals().get(head))?;
        for part in parts {
            let class_rc = match &current {
                Object::Class(c) | Object::Module(c) => Rc::clone(c),
                _ => return None,
            };
            current = class_rc.get_class_var(part)?;
        }
        Some(current)
    }
}

/// A class or module body's scope as the value its name stands for.
pub(crate) fn scope_object(scope: &Rc<Class>) -> Object {
    if scope.is_module() {
        Object::Module(Rc::clone(scope))
    } else {
        Object::Class(Rc::clone(scope))
    }
}
