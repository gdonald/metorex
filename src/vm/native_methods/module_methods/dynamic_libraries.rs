// Opening a shared library and finding a symbol in it, which Fiddle::Handle
// is built on.

use super::*;

/// The text `dlerror` holds for the call that just failed.
fn last_loader_error() -> String {
    // SAFETY: `dlerror` answers a C string the loader owns, or null when no
    // call has failed since it was last asked.
    unsafe {
        let held = libc::dlerror();
        if held.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(held).to_string_lossy().to_string()
        }
    }
}

fn loader_error(message: &str, position: Position) -> MetorexError {
    simple_exception("Fiddle::DLError", message, position)
}

fn null_byte_error(position: Position) -> MetorexError {
    simple_exception("ArgumentError", "string contains null byte", position)
}

impl VirtualMachine {
    /// `Fiddle.__dynamic_library__(command, ...)`: `:open` with a path (or
    /// nil for the program itself) and flags answers the handle's address,
    /// `:symbol` with a handle and a name answers the symbol's address,
    /// `:close` with a handle closes it, and `:default` answers the handle
    /// that searches every library loaded. `:flags` answers RTLD_GLOBAL,
    /// RTLD_LAZY, and RTLD_NOW as the platform numbers them.
    pub(crate) fn dynamic_library_command(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let command = match arguments.first() {
            Some(Object::Symbol(name)) => name.as_str().to_string(),
            _ => String::new(),
        };
        let number = |at: usize| match arguments.get(at) {
            Some(Object::Int(held)) => *held,
            _ => 0,
        };
        let text = |at: usize| match arguments.get(at) {
            Some(Object::String(held)) => Some(held.as_str().to_string()),
            _ => None,
        };
        match command.as_str() {
            "open" => {
                let path = match text(1) {
                    Some(path) => {
                        Some(std::ffi::CString::new(path).map_err(|_| null_byte_error(position))?)
                    }
                    None => None,
                };
                // SAFETY: the path is a C string that lives for the call, or
                // null for the program itself.
                let opened = unsafe {
                    libc::dlopen(
                        path.as_ref().map_or(std::ptr::null(), |held| held.as_ptr()),
                        number(2) as libc::c_int,
                    )
                };
                if opened.is_null() {
                    return Err(loader_error(&last_loader_error(), position));
                }
                Ok(Object::Int(opened as i64))
            }
            "symbol" => {
                let name = text(2).unwrap_or_default();
                let wanted =
                    std::ffi::CString::new(name.clone()).map_err(|_| null_byte_error(position))?;
                // SAFETY: the handle came from `dlopen` or is RTLD_DEFAULT,
                // and the name is a C string that lives for the call.
                let found = unsafe { libc::dlsym(number(1) as *mut libc::c_void, wanted.as_ptr()) };
                if found.is_null() {
                    return Err(loader_error(
                        &format!("unknown symbol \"{name}\""),
                        position,
                    ));
                }
                Ok(Object::Int(found as i64))
            }
            "close" => {
                // SAFETY: the handle came from `dlopen`, and the Ruby side
                // closes each one once.
                let closed = unsafe { libc::dlclose(number(1) as *mut libc::c_void) };
                Ok(Object::Int(closed as i64))
            }
            "read" => {
                let length = number(2).max(0) as usize;
                // SAFETY: Fiddle::Pointer reads the memory at an address the
                // program handed it, which is the use Fiddle is for.
                let bytes = unsafe { std::slice::from_raw_parts(number(1) as *const u8, length) };
                let characters = bytes.iter().map(|byte| *byte as char).collect::<String>();
                Ok(Object::String(std::rc::Rc::new(
                    crate::object::StringValue::from_bytes(characters),
                )))
            }
            "flags" => Ok(Object::array(vec![
                Object::Int(libc::RTLD_GLOBAL as i64),
                Object::Int(libc::RTLD_LAZY as i64),
                Object::Int(libc::RTLD_NOW as i64),
            ])),
            _ => Ok(Object::Int(libc::RTLD_DEFAULT as i64)),
        }
    }
}
