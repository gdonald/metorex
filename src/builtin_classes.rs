// Built-in base classes for Metorex runtime
// Defines the hierarchy and methods for core classes like Object, String, Integer, etc.

use crate::class::Class;
use crate::object::{Method, Object};
use std::collections::HashMap;
use std::rc::Rc;

/// Registry for built-in classes
pub struct BuiltinClasses {
    /// Base Object class (all classes inherit from this)
    pub object_class: Rc<Class>,
    /// String class
    pub string_class: Rc<Class>,
    /// Integer class
    pub integer_class: Rc<Class>,
    /// Float class
    pub float_class: Rc<Class>,
    /// Array class
    pub array_class: Rc<Class>,
    /// Hash/Dictionary class
    pub hash_class: Rc<Class>,
    /// Set class
    pub set_class: Rc<Class>,
    /// Range class
    pub range_class: Rc<Class>,
    /// Base Exception class
    pub exception_class: Rc<Class>,
    /// StandardError class (inherits from Exception)
    pub standard_error_class: Rc<Class>,
    /// RuntimeError class (inherits from StandardError)
    pub runtime_error_class: Rc<Class>,
    /// TypeError class (inherits from StandardError)
    pub type_error_class: Rc<Class>,
    /// ValueError class (inherits from StandardError)
    pub value_error_class: Rc<Class>,
}

impl BuiltinClasses {
    /// Create and initialize all built-in classes
    pub fn new() -> Self {
        // Create the base Object class
        let object_class = Rc::new(Class::new("Object", None));

        // Create primitive type classes
        let string_class = Rc::new(Class::new("String", Some(Rc::clone(&object_class))));
        let integer_class = Rc::new(Class::new("Integer", Some(Rc::clone(&object_class))));
        let float_class = Rc::new(Class::new("Float", Some(Rc::clone(&object_class))));

        // Create collection classes
        let array_class = Rc::new(Class::new("Array", Some(Rc::clone(&object_class))));
        let hash_class = Rc::new(Class::new("Hash", Some(Rc::clone(&object_class))));
        let set_class = Rc::new(Class::new("Set", Some(Rc::clone(&object_class))));
        let range_class = Rc::new(Class::new("Range", Some(Rc::clone(&object_class))));

        // Create exception hierarchy
        let exception_class = Rc::new(Class::new("Exception", Some(Rc::clone(&object_class))));
        let standard_error_class = Rc::new(Class::new(
            "StandardError",
            Some(Rc::clone(&exception_class)),
        ));
        let runtime_error_class = Rc::new(Class::new(
            "RuntimeError",
            Some(Rc::clone(&standard_error_class)),
        ));
        let type_error_class = Rc::new(Class::new(
            "TypeError",
            Some(Rc::clone(&standard_error_class)),
        ));
        let value_error_class = Rc::new(Class::new(
            "ValueError",
            Some(Rc::clone(&standard_error_class)),
        ));

        Self {
            object_class,
            string_class,
            integer_class,
            float_class,
            array_class,
            hash_class,
            set_class,
            range_class,
            exception_class,
            standard_error_class,
            runtime_error_class,
            type_error_class,
            value_error_class,
        }
    }

    /// Get the class for a given object
    pub fn class_of(&self, obj: &Object) -> Rc<Class> {
        match obj {
            Object::Nil => Rc::clone(&self.object_class),
            Object::Bool(_) => Rc::clone(&self.object_class),
            Object::Int(_) => Rc::clone(&self.integer_class),
            Object::Float(_) => Rc::clone(&self.float_class),
            Object::String(_) => Rc::clone(&self.string_class),
            Object::Array(_) => Rc::clone(&self.array_class),
            Object::Dict(_) => Rc::clone(&self.hash_class),
            Object::Set(_) => Rc::clone(&self.set_class),
            Object::Instance(inst) => Rc::clone(&inst.borrow().class),
            Object::Class(_) => Rc::clone(&self.object_class),
            Object::Method(_) => Rc::clone(&self.object_class),
            Object::Block(_) => Rc::clone(&self.object_class),
            Object::Exception(_) => Rc::clone(&self.exception_class),
            Object::Result(_) => Rc::clone(&self.object_class),
            Object::NativeFunction(_) => Rc::clone(&self.object_class),
            Object::Range { .. } => Rc::clone(&self.range_class),
        }
    }

    /// Check if an object is an instance of a class (or its subclasses)
    pub fn is_instance_of(&self, obj: &Object, class: &Class) -> bool {
        let obj_class = self.class_of(obj);
        self.is_subclass_of(&obj_class, class)
    }

    /// Check if class_a is a subclass of class_b
    pub fn is_subclass_of(&self, class_a: &Class, class_b: &Class) -> bool {
        Self::is_subclass_of_static(class_a, class_b)
    }

    /// Static helper for is_subclass_of
    fn is_subclass_of_static(class_a: &Class, class_b: &Class) -> bool {
        if class_a.name() == class_b.name() {
            return true;
        }

        if let Some(superclass) = class_a.superclass() {
            return Self::is_subclass_of_static(&superclass, class_b);
        }

        false
    }

    /// Get all built-in classes as a map
    pub fn all_classes(&self) -> HashMap<String, Rc<Class>> {
        let mut classes = HashMap::new();
        classes.insert("Object".to_string(), Rc::clone(&self.object_class));
        classes.insert("String".to_string(), Rc::clone(&self.string_class));
        classes.insert("Integer".to_string(), Rc::clone(&self.integer_class));
        classes.insert("Float".to_string(), Rc::clone(&self.float_class));
        classes.insert("Array".to_string(), Rc::clone(&self.array_class));
        classes.insert("Hash".to_string(), Rc::clone(&self.hash_class));
        classes.insert("Set".to_string(), Rc::clone(&self.set_class));
        classes.insert("Exception".to_string(), Rc::clone(&self.exception_class));
        classes.insert(
            "StandardError".to_string(),
            Rc::clone(&self.standard_error_class),
        );
        classes.insert(
            "RuntimeError".to_string(),
            Rc::clone(&self.runtime_error_class),
        );
        classes.insert("TypeError".to_string(), Rc::clone(&self.type_error_class));
        classes.insert("ValueError".to_string(), Rc::clone(&self.value_error_class));
        classes
    }
}

impl Default for BuiltinClasses {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize built-in methods for the Object class
pub fn init_object_methods(object_class: &Class) {
    // Object#to_s - convert to string representation
    let to_s_method = Rc::new(Method::new("to_s".to_string(), vec![], vec![]));
    object_class.define_method("to_s", to_s_method);

    // Object#class - get the class of the object
    let class_method = Rc::new(Method::new("class".to_string(), vec![], vec![]));
    object_class.define_method("class", class_method);

    // Object#respond_to? - check if object responds to a method
    let respond_to_method = Rc::new(Method::new(
        "respond_to?".to_string(),
        vec!["method_name".to_string()],
        vec![],
    ));
    object_class.define_method("respond_to?", respond_to_method);
}

/// Initialize built-in methods for the String class
pub fn init_string_methods(string_class: &Class) {
    // String#length
    let length_method = Rc::new(Method::new("length".to_string(), vec![], vec![]));
    string_class.define_method("length", length_method);

    // String#upcase
    let upcase_method = Rc::new(Method::new("upcase".to_string(), vec![], vec![]));
    string_class.define_method("upcase", upcase_method);

    // String#downcase
    let downcase_method = Rc::new(Method::new("downcase".to_string(), vec![], vec![]));
    string_class.define_method("downcase", downcase_method);

    // String#+
    let concat_method = Rc::new(Method::new(
        "+".to_string(),
        vec!["other".to_string()],
        vec![],
    ));
    string_class.define_method("+", concat_method);

    // String#trim
    let trim_method = Rc::new(Method::new("trim".to_string(), vec![], vec![]));
    string_class.define_method("trim", trim_method);

    // String#reverse
    let reverse_method = Rc::new(Method::new("reverse".to_string(), vec![], vec![]));
    string_class.define_method("reverse", reverse_method);

    // String#chars
    let chars_method = Rc::new(Method::new("chars".to_string(), vec![], vec![]));
    string_class.define_method("chars", chars_method);

    // String#bytes
    let bytes_method = Rc::new(Method::new("bytes".to_string(), vec![], vec![]));
    string_class.define_method("bytes", bytes_method);
}

/// Initialize built-in methods for the Array class
pub fn init_array_methods(array_class: &Class) {
    // Array#length
    let length_method = Rc::new(Method::new("length".to_string(), vec![], vec![]));
    array_class.define_method("length", length_method);

    // Array#push
    let push_method = Rc::new(Method::new(
        "push".to_string(),
        vec!["item".to_string()],
        vec![],
    ));
    array_class.define_method("push", push_method);

    // Array#pop
    let pop_method = Rc::new(Method::new("pop".to_string(), vec![], vec![]));
    array_class.define_method("pop", pop_method);

    // Array#[]
    let index_method = Rc::new(Method::new(
        "[]".to_string(),
        vec!["index".to_string()],
        vec![],
    ));
    array_class.define_method("[]", index_method);
}

/// Initialize built-in methods for the Hash class
pub fn init_hash_methods(hash_class: &Class) {
    // Hash#keys
    let keys_method = Rc::new(Method::new("keys".to_string(), vec![], vec![]));
    hash_class.define_method("keys", keys_method);

    // Hash#values
    let values_method = Rc::new(Method::new("values".to_string(), vec![], vec![]));
    hash_class.define_method("values", values_method);

    // Hash#has_key?
    let has_key_method = Rc::new(Method::new(
        "has_key?".to_string(),
        vec!["key".to_string()],
        vec![],
    ));
    hash_class.define_method("has_key?", has_key_method);

    // Hash#entries
    let entries_method = Rc::new(Method::new("entries".to_string(), vec![], vec![]));
    hash_class.define_method("entries", entries_method);

    // Hash#to_a (alias for entries)
    let to_a_method = Rc::new(Method::new("to_a".to_string(), vec![], vec![]));
    hash_class.define_method("to_a", to_a_method);

    // Hash#length
    let length_method = Rc::new(Method::new("length".to_string(), vec![], vec![]));
    hash_class.define_method("length", length_method);

    // Hash#size (alias for length)
    let size_method = Rc::new(Method::new("size".to_string(), vec![], vec![]));
    hash_class.define_method("size", size_method);

    // Hash#[]
    let index_method = Rc::new(Method::new(
        "[]".to_string(),
        vec!["key".to_string()],
        vec![],
    ));
    hash_class.define_method("[]", index_method);
}
