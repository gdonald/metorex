// The terminal settings io/console reads and changes: a descriptor's termios
// held as a byte string, the window size, and flushing the queues.

use super::*;

/// A descriptor's terminal settings, or the errno the read failed with.
fn terminal_settings(descriptor: i32) -> Result<libc::termios, i32> {
    let mut settings = std::mem::MaybeUninit::<libc::termios>::zeroed();
    if unsafe { libc::tcgetattr(descriptor, settings.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error().raw_os_error().unwrap_or(0));
    }
    Ok(unsafe { settings.assume_init() })
}

/// Terminal settings as the bytes the Ruby side keeps.
fn settings_bytes(settings: &libc::termios) -> Object {
    let bytes = unsafe {
        std::slice::from_raw_parts(
            (settings as *const libc::termios).cast::<u8>(),
            std::mem::size_of::<libc::termios>(),
        )
    };
    crate::vm::native_methods::pack_format::bytes_to_string(bytes)
}

/// Terminal settings read back from the bytes the Ruby side kept.
fn settings_from(argument: Option<&Object>) -> Option<libc::termios> {
    let Some(Object::String(text)) = argument else {
        return None;
    };
    let bytes = crate::vm::native_methods::string_methods::binary_bytes(text);
    if bytes.len() != std::mem::size_of::<libc::termios>() {
        return None;
    }
    let mut settings = std::mem::MaybeUninit::<libc::termios>::zeroed();
    unsafe {
        std::ptr::copy_nonoverlapping(
            bytes.as_ptr(),
            settings.as_mut_ptr().cast::<u8>(),
            bytes.len(),
        );
        Some(settings.assume_init())
    }
}

fn integer_argument(argument: Option<&Object>) -> i64 {
    match argument {
        Some(Object::Int(value)) => *value,
        _ => 0,
    }
}

/// What a call that can fail answers: nil, or the errno it failed with.
fn errno_or_nil(succeeded: bool) -> Object {
    if succeeded {
        Object::Nil
    } else {
        Object::Int(i64::from(
            std::io::Error::last_os_error().raw_os_error().unwrap_or(0),
        ))
    }
}

/// Raw mode as `cfmakeraw` and console.c's `set_rawmode` make it, with the
/// `min:`, `time:` and `intr:` options when they were given.
fn make_raw(settings: &mut libc::termios, vmin: i64, vtime: i64, intr: Option<bool>) {
    unsafe { libc::cfmakeraw(settings) };
    settings.c_lflag &= !(libc::ECHOE | libc::ECHOK);
    if vmin >= 0 {
        settings.c_cc[libc::VMIN] = (vmin & 0xff) as libc::cc_t;
    }
    if vtime >= 0 {
        settings.c_cc[libc::VTIME] = (vtime & 0xff) as libc::cc_t;
    }
    if intr == Some(true) {
        settings.c_iflag |= libc::BRKINT;
        settings.c_lflag |= libc::ISIG;
        settings.c_oflag |= libc::OPOST;
    }
}

impl VirtualMachine {
    /// The native functions io/console calls, by name.
    pub(crate) fn call_console_function(
        &mut self,
        name: &str,
        arguments: &[Object],
    ) -> Result<Object, MetorexError> {
        let descriptor = integer_argument(arguments.first()) as i32;
        Ok(match name {
            // The settings of a descriptor, or the errno reading them failed
            // with, which is ENOTTY for anything but a terminal.
            "__console_mode_get__" => match terminal_settings(descriptor) {
                Ok(settings) => settings_bytes(&settings),
                Err(errno) => Object::Int(i64::from(errno)),
            },
            "__console_mode_set__" => match settings_from(arguments.get(1)) {
                Some(settings) => errno_or_nil(unsafe {
                    libc::tcsetattr(descriptor, libc::TCSANOW, &settings) == 0
                }),
                None => Object::Int(i64::from(libc::EINVAL)),
            },
            // `__console_mode_change__(settings, change, min, time, intr)`
            // answers the settings with console.c's raw, cooked, echo or
            // noecho change made.
            "__console_mode_change__" => {
                let Some(mut settings) = settings_from(arguments.first()) else {
                    return Ok(Object::Nil);
                };
                let change = match arguments.get(1) {
                    Some(Object::Symbol(change)) => change.as_str().to_string(),
                    _ => String::new(),
                };
                let echo = libc::ECHO | libc::ECHOE | libc::ECHOK | libc::ECHONL;
                match change.as_str() {
                    "raw" => {
                        let intr = match arguments.get(4) {
                            Some(Object::Bool(given)) => Some(*given),
                            _ => None,
                        };
                        make_raw(
                            &mut settings,
                            integer_argument(arguments.get(2)),
                            integer_argument(arguments.get(3)),
                            intr,
                        );
                    }
                    "cooked" => {
                        settings.c_iflag |= libc::BRKINT | libc::ISTRIP | libc::ICRNL | libc::IXON;
                        settings.c_oflag |= libc::OPOST;
                        settings.c_lflag |= echo | libc::ICANON | libc::ISIG | libc::IEXTEN;
                    }
                    "echo" => settings.c_lflag |= echo,
                    "noecho" => settings.c_lflag &= !echo,
                    _ => {}
                }
                settings_bytes(&settings)
            }
            // Whether echo is on in the settings.
            "__console_mode_query__" => match settings_from(arguments.first()) {
                Some(settings) => Object::Bool(settings.c_lflag & (libc::ECHO | libc::ECHONL) != 0),
                None => Object::Nil,
            },
            "__console_winsize__" => {
                let mut size = std::mem::MaybeUninit::<libc::winsize>::zeroed();
                if unsafe { libc::ioctl(descriptor, libc::TIOCGWINSZ, size.as_mut_ptr()) } != 0 {
                    return Ok(errno_or_nil(false));
                }
                let size = unsafe { size.assume_init() };
                Object::array(vec![
                    Object::Int(i64::from(size.ws_row)),
                    Object::Int(i64::from(size.ws_col)),
                ])
            }
            "__console_set_winsize__" => {
                let size = libc::winsize {
                    ws_row: integer_argument(arguments.get(1)) as u16,
                    ws_col: integer_argument(arguments.get(2)) as u16,
                    ws_xpixel: integer_argument(arguments.get(3)) as u16,
                    ws_ypixel: integer_argument(arguments.get(4)) as u16,
                };
                errno_or_nil(unsafe { libc::ioctl(descriptor, libc::TIOCSWINSZ, &size) } == 0)
            }
            // `__console_flush__(fd, queue)` discards the input queue (0),
            // the output queue (1) or both (2).
            "__console_flush__" => {
                let queue = match integer_argument(arguments.get(1)) {
                    0 => libc::TCIFLUSH,
                    1 => libc::TCOFLUSH,
                    _ => libc::TCIOFLUSH,
                };
                errno_or_nil(unsafe { libc::tcflush(descriptor, queue) } == 0)
            }
            "__console_beep__" => {
                errno_or_nil(unsafe { libc::write(descriptor, b"\x07".as_ptr().cast(), 1) } >= 0)
            }
            "__console_ttyname__" => {
                if unsafe { libc::isatty(descriptor) } != 1 {
                    return Ok(Object::Nil);
                }
                let name = unsafe { libc::ttyname(descriptor) };
                if name.is_null() {
                    return Ok(Object::Nil);
                }
                Object::string(
                    unsafe { std::ffi::CStr::from_ptr(name) }
                        .to_string_lossy()
                        .into_owned(),
                )
            }
            _ => Object::Nil,
        })
    }
}
