//! Loading a C extension: opening the shared library and running the
//! `Init_<name>` function named after its file.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::vm::VirtualMachine;
use std::ffi::{CStr, CString};
use std::path::Path;

impl VirtualMachine {
    /// Opens the extension at `path` and runs its `Init` function.
    pub(crate) fn load_native_extension(
        &mut self,
        path: &Path,
        position: Position,
    ) -> Result<(), MetorexError> {
        // Ruby reports what the loader said, and a file that is there names
        // no missing path.
        let raise_load_error = |message: String| MetorexError::UncaughtException {
            exception: crate::vm::errors::load_error_without_path(message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        };
        let file_name = CString::new(path.as_os_str().as_encoded_bytes())
            .expect("a path read from the file system holds no NUL");
        // SAFETY: the path is NUL-terminated, and a handle that is never
        // closed keeps the library's functions in place for its methods.
        let library =
            unsafe { libc::dlopen(file_name.as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL) };
        if library.is_null() {
            return Err(raise_load_error(loader_message()));
        }
        let stem = path
            .file_stem()
            .map(|held| held.to_string_lossy().into_owned())
            .unwrap_or_default();
        let init_name = CString::new(format!("Init_{stem}")).unwrap_or_default();
        // SAFETY: the handle came from `dlopen` and the name is NUL-terminated.
        let init = unsafe { libc::dlsym(library, init_name.as_ptr()) };
        if init.is_null() {
            return Err(raise_load_error(loader_message()));
        }
        self.publish_core_classes();
        // SAFETY: an extension's Init function takes nothing and returns
        // nothing.
        let init: extern "C-unwind" fn() = unsafe { std::mem::transmute(init) };
        super::enter(
            self,
            super::Caller {
                position,
                block: None,
                keywords_given: false,
                method: None,
            },
            || init(),
        )
    }
}

/// What the dynamic loader said about the call into it that just failed.
fn loader_message() -> String {
    // SAFETY: called right after `dlopen` or `dlsym` failed, when `dlerror`
    // answers the NUL-terminated message describing the failure.
    unsafe { CStr::from_ptr(libc::dlerror()) }
        .to_string_lossy()
        .into_owned()
}
