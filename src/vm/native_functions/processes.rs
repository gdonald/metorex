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

/// Whether a command written as one string has to run through `sh`: it
/// holds a character the shell reads or starts with a word the shell answers
/// itself, such as `exit`.
pub(crate) fn needs_a_shell(command: &str) -> bool {
    !command.trim().is_empty() && super::shell::shell_free_words(command).is_none()
}

/// Start a program, answering the process id it runs under, or a negative
/// one when no process could be made. A program that cannot be reached ends
/// with status 127 and says nothing, which is what the shell reports for one.
pub(crate) fn start_program(
    reached: &str,
    words: &[String],
    redirects: &[(i32, String)],
    environment: &[(String, Option<String>)],
) -> libc::pid_t {
    let named = std::ffi::CString::new(reached).unwrap_or_default();
    // The names are spelled before the split, so the child only hands them
    // to the system.
    let settings: Vec<(std::ffi::CString, Option<std::ffi::CString>)> = environment
        .iter()
        .map(|(name, value)| {
            (
                std::ffi::CString::new(name.as_str()).unwrap_or_default(),
                value
                    .as_ref()
                    .map(|held| std::ffi::CString::new(held.as_str()).unwrap_or_default()),
            )
        })
        .collect();
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
        for (name, value) in &settings {
            // SAFETY: both strings are null-terminated and outlive the calls.
            unsafe {
                match value {
                    Some(value) => libc::setenv(name.as_ptr(), value.as_ptr(), 1),
                    None => libc::unsetenv(name.as_ptr()),
                };
            }
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
    child
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

/// A pseudo-terminal pair: the controlling end, the end a program reads and
/// writes as its terminal, and the name of that end.
pub(crate) fn open_terminal_pair() -> std::io::Result<(libc::c_int, libc::c_int, String)> {
    let mut controlling: libc::c_int = -1;
    let mut terminal: libc::c_int = -1;
    // SAFETY: both pointers name ints this call writes, and the name, the
    // settings and the window size are left for the system to choose.
    let made = unsafe {
        libc::openpty(
            &mut controlling,
            &mut terminal,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if made != 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: `terminal` is a descriptor `openpty` just opened, and the name
    // `ttyname` answers is copied before anything else asks for one.
    let name = unsafe {
        let spelled = libc::ttyname(terminal);
        if spelled.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(spelled)
                .to_string_lossy()
                .into_owned()
        }
    };
    for end in [controlling, terminal] {
        // SAFETY: both descriptors came from `openpty` just above.
        unsafe { libc::fcntl(end, libc::F_SETFD, libc::FD_CLOEXEC) };
    }
    Ok((controlling, terminal, name))
}

/// Run `words` in a session of its own with a fresh pseudo-terminal as its
/// controlling terminal and its standard streams, answering the child's
/// process id, the controlling end, a second descriptor for that end, and
/// the terminal's name.
pub(crate) fn start_in_terminal(
    words: &[String],
    environment: &[(String, Option<String>)],
) -> std::io::Result<(libc::pid_t, libc::c_int, libc::c_int, String)> {
    let (controlling, terminal, name) = open_terminal_pair()?;
    let spelled: Vec<std::ffi::CString> = words
        .iter()
        .map(|held| std::ffi::CString::new(held.as_str()).unwrap_or_default())
        .collect();
    let settings: Vec<(std::ffi::CString, Option<std::ffi::CString>)> = environment
        .iter()
        .map(|(named, value)| {
            (
                std::ffi::CString::new(named.as_str()).unwrap_or_default(),
                value
                    .as_ref()
                    .map(|held| std::ffi::CString::new(held.as_str()).unwrap_or_default()),
            )
        })
        .collect();
    let mut pointers: Vec<*const libc::c_char> = spelled.iter().map(|held| held.as_ptr()).collect();
    pointers.push(std::ptr::null());
    // SAFETY: the child only calls functions that touch its own descriptors
    // and environment before handing itself over to the program.
    let child = unsafe { libc::fork() };
    if child < 0 {
        let problem = std::io::Error::last_os_error();
        // SAFETY: both descriptors came from `openpty` and are not used again.
        unsafe {
            libc::close(controlling);
            libc::close(terminal);
        }
        return Err(problem);
    }
    if child == 0 {
        // SAFETY: the argument list is null-terminated and outlives the call,
        // and every descriptor named is one this process holds.
        unsafe {
            libc::setsid();
            libc::ioctl(terminal, libc::TIOCSCTTY as _, 0);
            for slot in 0..3 {
                libc::dup2(terminal, slot);
            }
            if terminal > 2 {
                libc::close(terminal);
            }
            libc::close(controlling);
            for (named, value) in &settings {
                match value {
                    Some(value) => libc::setenv(named.as_ptr(), value.as_ptr(), 1),
                    None => libc::unsetenv(named.as_ptr()),
                };
            }
            libc::execvp(pointers[0], pointers.as_ptr());
            libc::_exit(127);
        }
    }
    // SAFETY: the terminal end belongs to the child now, and `controlling`
    // is a descriptor this process holds.
    let copy = unsafe {
        libc::close(terminal);
        libc::dup(controlling)
    };
    // SAFETY: `copy` came from `dup` just above.
    unsafe { libc::fcntl(copy, libc::F_SETFD, libc::FD_CLOEXEC) };
    Ok((child, controlling, copy, name))
}
