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
                block_from_ampersand: false,
                keywords_given: false,
                method: None,
            },
            || init(),
        )
    }
}

unsafe extern "C-unwind" {
    fn Init_nkf();
}

/// The C extensions build.rs compiles into the binary, by the feature name
/// `require` finds each under, with its `Init` function.
const STATIC_EXTENSIONS: &[(&str, unsafe extern "C-unwind" fn())] = &[("nkf", Init_nkf)];

/// The `Init` function of a C extension compiled into the binary, for a
/// name such as `nkf` or `nkf.so`.
pub(crate) fn static_extension(name: &str) -> Option<unsafe extern "C-unwind" fn()> {
    let stem = [".so", ".bundle"]
        .iter()
        .find_map(|ending| name.strip_suffix(ending))
        .unwrap_or(name);
    STATIC_EXTENSIONS
        .iter()
        .find(|(named, _)| *named == stem)
        .map(|(_, init)| *init)
}

impl VirtualMachine {
    /// Run the `Init` function of a C extension compiled into the binary,
    /// once, and answer whether this was the first time.
    pub(crate) fn run_static_extension(
        &mut self,
        name: &str,
        init: unsafe extern "C-unwind" fn(),
        position: Position,
    ) -> Result<bool, MetorexError> {
        let stem = name.split('.').next().unwrap_or(name);
        let marker = std::path::PathBuf::from(format!("<metorex>/{stem}.so"));
        if self.is_file_loaded(&marker) {
            return Ok(false);
        }
        self.mark_file_loaded(marker);
        self.publish_core_classes();
        super::enter(
            self,
            super::Caller {
                position,
                block: None,
                block_from_ampersand: false,
                keywords_given: false,
                method: None,
            },
            // SAFETY: an extension's Init function takes nothing and returns
            // nothing.
            || unsafe { init() },
        )?;
        Ok(true)
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
