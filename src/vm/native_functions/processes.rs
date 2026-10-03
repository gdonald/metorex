// Running a program to completion, and finding one to run.

use super::*;

impl crate::vm::core::VirtualMachine {
    /// A name written at the top level names a method of Object, since that
    /// is where a `def` outside any class or module is written. The top-level
    /// `self` is the Object class itself, so an ordinary send looks for a
    /// class method of that name and finds nothing.
    pub(crate) fn top_level_method(&mut self, receiver: &Object, name: &str) -> Option<Object> {
        let Object::Class(held) = receiver else {
            return None;
        };
        if held.name() != "Object" {
            return None;
        }
        let (owner, method) = held.find_method_with_owner(name)?;
        let mut bound = (*method).clone();
        bound.receiver = Some(Box::new(receiver.clone()));
        bound.owner = Some(owner.ruby_name());
        bound.owner_class = Some(owner);
        Some(Object::Method(std::rc::Rc::new(bound)))
    }
}

/// Whether a command written as one string holds a character the shell reads,
/// which is what decides between running it through `sh` and running it as
/// the program it names.
pub(crate) fn needs_a_shell(command: &str) -> bool {
    const READ_BY_THE_SHELL: &[u8] = b"*?{}[]<>()~&|\\$;'`\"\n#";
    command
        .bytes()
        .any(|byte| READ_BY_THE_SHELL.contains(&byte))
}

/// Run a program to completion, answering how it ended and the process id it
/// ran under. A program that cannot be reached ends with status 127 and says
/// nothing, which is what the shell reports for one.
pub(crate) fn run_to_completion(
    reached: &str,
    words: &[String],
    redirects: &[(i32, String)],
) -> (std::process::ExitStatus, i64) {
    use std::os::unix::process::ExitStatusExt as _;
    let named = std::ffi::CString::new(reached).unwrap_or_default();
    let spelled: Vec<std::ffi::CString> = words
        .iter()
        .map(|held| std::ffi::CString::new(held.as_str()).unwrap_or_default())
        .collect();
    let opened: Vec<(i32, std::fs::File)> = redirects
        .iter()
        .filter_map(|(slot, path)| {
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(false)
                .open(path)
                .ok()
                .map(|held| (*slot, held))
        })
        .collect();
    // SAFETY: the child does nothing but point its streams where it was told
    // and hand itself over to the program, so nothing the parent holds is
    // read or written there.
    let child = unsafe { libc::fork() };
    if child == 0 {
        use std::os::unix::io::AsRawFd as _;
        for (slot, file) in &opened {
            // SAFETY: both numbers name descriptors this process holds.
            unsafe { libc::dup2(file.as_raw_fd(), *slot) };
        }
        let mut pointers: Vec<*const libc::c_char> =
            spelled.iter().map(|held| held.as_ptr()).collect();
        pointers.push(std::ptr::null());
        // SAFETY: the argument list is null-terminated and outlives the call.
        unsafe {
            libc::execvp(named.as_ptr(), pointers.as_ptr());
            libc::_exit(127);
        }
    }
    if child < 0 {
        return (std::process::ExitStatus::from_raw(127 << 8), 0);
    }
    let mut held: libc::c_int = 0;
    // SAFETY: `child` is a process this one started.
    unsafe { libc::waitpid(child, &mut held, 0) };
    (std::process::ExitStatus::from_raw(held), child as i64)
}

/// The files a `require` of `named` may mean, in the order Ruby tries them.
/// A path already ending in `.rb` is taken as written rather than having
/// another `.rb` added to it.
pub(crate) fn require_candidates(named: &str) -> Vec<std::path::PathBuf> {
    if named.ends_with(".rb") {
        return vec![std::path::PathBuf::from(named)];
    }
    let mut candidates = vec![std::path::PathBuf::from(format!("{}.rb", named))];
    // A name with no ending of its own can name a C extension built for
    // this platform, which Ruby looks for after the Ruby file.
    if std::path::Path::new(named).extension().is_none() {
        candidates.push(std::path::PathBuf::from(format!(
            "{}.{}",
            named, PLATFORM_EXTENSION
        )));
    }
    candidates.push(std::path::PathBuf::from(named));
    candidates
}

/// The ending a C extension built for this platform carries.
pub(crate) const PLATFORM_EXTENSION: &str = if cfg!(target_os = "macos") {
    "bundle"
} else {
    "so"
};

/// Whether a path names a file built for the machine rather than one written
/// in Ruby.
pub(crate) fn names_a_native_extension(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|held| held.to_str())
        .is_some_and(|held| NATIVE_EXTENSIONS.contains(&held))
}

/// The endings of a file built for the machine rather than written in Ruby.
pub(crate) const NATIVE_EXTENSIONS: [&str; 4] = ["so", "bundle", "dylib", "dll"];

/// The path a feature entry names, with a relative one read from the working
/// directory and any `.` or `..` in it taken out.
pub(crate) fn expanded_feature_path(path: &std::path::Path) -> std::path::PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    crate::vm::loading::without_dot_components(&absolute)
}
