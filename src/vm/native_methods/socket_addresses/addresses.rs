// Reading an address out of the text a program writes it as.

use super::*;

impl VirtualMachine {
    /// `Socket.__address__(action, text, port)` — the one place the socket
    /// library reads and writes addresses, since their shape belongs to the
    /// operating system rather than to Ruby.
    pub(crate) fn socket_address(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(Object::String(action)) = arguments.first() else {
            return Err(crate::vm::errors::argument_count_error(
                crate::vm::errors::Arity::AtLeast(1),
                arguments.len(),
                position,
            ));
        };
        let text = match arguments.get(1) {
            Some(Object::String(held)) => held.as_str().to_string(),
            _ => String::new(),
        };
        let port = match arguments.get(2) {
            Some(Object::Int(held)) => *held,
            _ => 0,
        };
        let refuse =
            |what: String| crate::vm::errors::simple_exception("SocketError", &what, position);
        match &*action.as_str() {
            // Which family an address belongs to, or nothing when the text
            // names no address at all.
            "family" => Ok(match read_address(&text) {
                Some(Written::Four(_)) => Object::Int(4),
                Some(Written::Six(_)) => Object::Int(6),
                None => Object::Nil,
            }),
            // The address written back out in its shortest form.
            "normalize" => match read_address(&text) {
                Some(Written::Four(held)) => {
                    Ok(Object::string(std::net::Ipv4Addr::from(held).to_string()))
                }
                Some(Written::Six(held)) => Ok(Object::string(write_ipv6(held))),
                None => Err(refuse(format!(
                    "getaddrinfo: {text}: Name or service not known"
                ))),
            },
            // The bytes an address stands for, one to a character.
            "bytes" => match read_address(&text) {
                Some(Written::Four(held)) => Ok(
                    crate::vm::native_methods::pack_format::bytes_to_string(&held),
                ),
                Some(Written::Six(held)) => Ok(
                    crate::vm::native_methods::pack_format::bytes_to_string(&held),
                ),
                None => Err(refuse(format!(
                    "getaddrinfo: {text}: Name or service not known"
                ))),
            },
            // The struct the operating system takes an address in, which
            // carries a length byte on some systems and not on others.
            "sockaddr" => {
                let held = read_address(&text).ok_or_else(|| {
                    refuse(format!("getaddrinfo: {text}: Name or service not known"))
                })?;
                let mut out = Vec::new();
                match held {
                    Written::Four(address) => {
                        push_family(&mut out, libc::AF_INET as u16, 16);
                        out.extend_from_slice(&(port as u16).to_be_bytes());
                        out.extend_from_slice(&address);
                        out.extend_from_slice(&[0u8; 8]);
                    }
                    Written::Six(address) => {
                        push_family(&mut out, libc::AF_INET6 as u16, 28);
                        out.extend_from_slice(&(port as u16).to_be_bytes());
                        out.extend_from_slice(&[0u8; 4]);
                        out.extend_from_slice(&address);
                        out.extend_from_slice(&[0u8; 4]);
                    }
                }
                Ok(crate::vm::native_methods::pack_format::bytes_to_string(
                    &out,
                ))
            }
            // The address and port a struct carries, read back out.
            "unpack" => {
                let bytes = crate::vm::native_methods::pack_format::string_to_bytes(&text);
                match read_sockaddr(&bytes) {
                    Some((named, held)) => Ok(Object::array(vec![
                        Object::string(named),
                        Object::Int(i64::from(held)),
                    ])),
                    None => Err(refuse("not an IP address struct".to_string())),
                }
            }
            // The names a host answers to, resolved by the system.
            // The name an address is known by, or nothing when it has none.
            "name_of" => {
                let Ok(address) = text.parse::<std::net::IpAddr>() else {
                    return Ok(Object::Nil);
                };
                let mut buffer = vec![0u8; libc::NI_MAXHOST as usize];
                let answered = match address {
                    std::net::IpAddr::V4(held) => {
                        let mut sockaddr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
                        sockaddr.sin_family = libc::AF_INET as libc::sa_family_t;
                        sockaddr.sin_addr.s_addr = u32::from_ne_bytes(held.octets());
                        unsafe {
                            libc::getnameinfo(
                                &sockaddr as *const libc::sockaddr_in as *const libc::sockaddr,
                                std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
                                buffer.as_mut_ptr() as *mut libc::c_char,
                                buffer.len() as libc::socklen_t,
                                std::ptr::null_mut(),
                                0,
                                0,
                            )
                        }
                    }
                    std::net::IpAddr::V6(_) => -1,
                };
                if answered != 0 {
                    return Ok(Object::Nil);
                }
                let named = unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr() as *const _) };
                Ok(Object::string(named.to_string_lossy().to_string()))
            }
            "resolve" => match std::net::ToSocketAddrs::to_socket_addrs(&(text.as_str(), 0u16)) {
                Ok(found) => Ok(Object::array(
                    found
                        .map(|held| Object::string(held.ip().to_string()))
                        .collect(),
                )),
                Err(_) => Err(refuse(format!(
                    "getaddrinfo: {text}: nodename nor servname provided, or not known"
                ))),
            },
            // The name this machine answers to.
            "hostname" => Ok(Object::string(
                hostname().unwrap_or_else(|| "localhost".to_string()),
            )),
            other => Err(MetorexError::runtime_error(
                format!("unknown address action {other}"),
                crate::vm::utils::position_to_location(position),
            )),
        }
    }
}

/// The four bytes an IPv4 address stands for, or None when the text is not
/// one.
pub(crate) fn ipv4_bytes(text: &str) -> Option<[u8; 4]> {
    let mut held = [0u8; 4];
    let mut pieces = text.split('.');
    for slot in held.iter_mut() {
        let piece = pieces.next()?;
        if piece.is_empty() || piece.len() > 3 || !piece.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        *slot = piece.parse().ok()?;
    }
    if pieces.next().is_some() {
        return None;
    }
    Some(held)
}

/// The sixteen bytes an IPv6 address stands for, `::` included.
pub(crate) fn ipv6_bytes(text: &str) -> Option<[u8; 16]> {
    let held: std::net::Ipv6Addr = text.parse().ok()?;
    Some(held.octets())
}

/// How an address was written, which decides how everything else reads it.
enum Written {
    Four([u8; 4]),
    Six([u8; 16]),
}

fn read_address(text: &str) -> Option<Written> {
    if let Some(held) = ipv4_bytes(text) {
        return Some(Written::Four(held));
    }
    ipv6_bytes(text).map(Written::Six)
}

/// An IPv6 address written the shortest way it can be, which is how Ruby
/// spells one back out.
fn write_ipv6(bytes: [u8; 16]) -> String {
    std::net::Ipv6Addr::from(bytes).to_string()
}

/// The name this machine answers to, as the system reports it.
fn hostname() -> Option<String> {
    let mut buffer = [0u8; 256];
    // SAFETY: `gethostname` writes at most the length given into the buffer,
    // which outlives the call.
    let answered = unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) };
    if answered != 0 {
        return None;
    }
    let end = buffer.iter().position(|byte| *byte == 0).unwrap_or(0);
    String::from_utf8(buffer[..end].to_vec()).ok()
}

/// Write the family into a sockaddr struct. BSD systems put the struct's
/// length in front of it; Linux does not and holds the family in two bytes.
fn push_family(out: &mut Vec<u8>, family: u16, length: u8) {
    if cfg!(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    )) {
        out.push(length);
        out.push(family as u8);
    } else {
        out.extend_from_slice(&family.to_le_bytes());
    }
}

/// The address and port a sockaddr struct carries.
fn read_sockaddr(bytes: &[u8]) -> Option<(String, u16)> {
    if bytes.len() < 8 {
        return None;
    }
    let family = if cfg!(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    )) {
        u16::from(bytes[1])
    } else {
        u16::from_le_bytes([bytes[0], bytes[1]])
    };
    let port = u16::from_be_bytes([bytes[2], bytes[3]]);
    if family == libc::AF_INET as u16 {
        let held: [u8; 4] = bytes.get(4..8)?.try_into().ok()?;
        return Some((std::net::Ipv4Addr::from(held).to_string(), port));
    }
    if family == libc::AF_INET6 as u16 {
        let held: [u8; 16] = bytes.get(8..24)?.try_into().ok()?;
        return Some((write_ipv6(held), port));
    }
    None
}
