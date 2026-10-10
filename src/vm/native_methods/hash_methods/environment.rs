// ENV, the hash the process shares with the operating system.

use super::*;

impl VirtualMachine {
    /// The text an argument to an environment method names, refusing anything
    /// that names none.
    pub(crate) fn environment_text(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let Object::String(text) = value {
            return Ok(text.to_text());
        }
        if self.responds_to(value, "to_str")
            && let Object::String(text) =
                self.send_to_object(value.clone(), "to_str", vec![], position)?
        {
            return Ok(text.to_text());
        }
        let message = format!(
            "no implicit conversion of {} into String",
            crate::vm::errors::conversion_subject(value)
        );
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }

    /// The encoding the environment is read as: the one a program named, or
    /// the one the system is set to.
    pub(crate) fn environment_reading_encoding(&mut self) -> Option<String> {
        if let Some(Object::Class(held)) = self.globals().get("__Encoding_default_internal") {
            return Some(held.name().to_string());
        }
        // With nothing named, the environment reads as the locale does,
        // which is what the C library hands the program its variables in.
        match crate::vm::locale_charmap_name().as_str() {
            "" => None,
            "ANSI_X3.4-1968" => Some("US-ASCII".to_string()),
            other => Some(other.to_string()),
        }
    }

    /// Refuse a name the environment could never hold.
    pub(crate) fn refuse_bad_variable_name(
        &self,
        key: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        if key.is_empty() || key.contains('=') {
            let message = format!("Invalid argument - setenv({})", key);
            return Err(crate::vm::errors::simple_exception(
                "Errno::EINVAL",
                &message,
                position,
            ));
        }
        Ok(())
    }

    /// Put one variable into the environment, where the C library sees it too.
    pub(crate) fn store_variable(
        &mut self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
        key: String,
        value: String,
    ) {
        let named = Object::string(key.clone());
        // What the environment holds is read back in the encoding the locale
        // names, so it is kept in that encoding from the moment it is set.
        let reading = self.environment_reading_encoding();
        let held = retagged(Object::string(value), &reading);
        // A name that reads back as some other kind of value is kept beside
        // the entry, so `ENV["1"]` stays a name rather than a number.
        if !crate::vm::utils::is_primitive_key(&named) {
            let slot = "__MX_KEY_OBJECTS__".to_string();
            let mut objects = match dict_rc.borrow().get(&slot) {
                Some(Object::Dict(held)) => held.borrow().clone(),
                _ => indexmap::IndexMap::new(),
            };
            objects.insert(key.clone(), named.clone());
            dict_rc
                .borrow_mut()
                .insert(slot, Object::Dict(Rc::new(RefCell::new(objects))));
        }
        dict_rc.borrow_mut().insert(key, held.clone());
        self.record_environment_change(dict_rc, &named, &held);
    }

    /// Whether a hash is the one ENV names, which a few methods answer
    /// differently on.
    pub(crate) fn dict_is_environment(
        &self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
    ) -> bool {
        matches!(self.globals().get("ENV"), Some(Object::Dict(held)) if Rc::ptr_eq(&held, dict_rc))
    }

    /// A write to ENV reaches the process environment too, so the C library
    /// sees it. Anything the C library reads, such as the time zone, follows
    /// what the program set.
    pub(crate) fn record_environment_change(
        &mut self,
        dict_rc: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
        key: &Object,
        value: &Object,
    ) {
        let Some(Object::Dict(environment)) = self.globals().get("ENV") else {
            return;
        };
        if !Rc::ptr_eq(&environment, dict_rc) {
            return;
        }
        let Object::String(name) = key else {
            return;
        };
        let Ok(name) = std::ffi::CString::new(name.as_str().as_bytes().to_vec()) else {
            return;
        };
        // Ruby removes the entry outright when a name is set to nil, so the
        // name stops being one the environment holds at all.
        if matches!(value, Object::Nil) {
            let mut held = dict_rc.borrow_mut();
            if let Some(key_text) = crate::vm::utils::object_to_dict_key(key) {
                held.shift_remove(&key_text);
            }
        }
        unsafe {
            match value {
                Object::Nil => {
                    libc::unsetenv(name.as_ptr());
                }
                Object::String(text) => {
                    let Ok(setting) = std::ffi::CString::new(text.as_str().as_bytes().to_vec())
                    else {
                        return;
                    };
                    libc::setenv(name.as_ptr(), setting.as_ptr(), 1);
                }
                _ => {}
            }
        }
    }
}

/// The environment methods whose one argument names a variable or a value,
/// which is written as text however it arrives.
pub(crate) const ENVIRONMENT_TEXT_ARGUMENT: &[&str] = &[
    "[]",
    "delete",
    "has_key?",
    "key?",
    "include?",
    "member?",
    "key",
    "assoc",
    "has_value?",
    "value?",
    "rassoc",
];

/// The subset of those whose argument is a value rather than a name.
pub(crate) const ENVIRONMENT_VALUE_ARGUMENT: &[&str] = &["has_value?", "value?", "rassoc"];
