// The settings a socket carries, and the services a port names.

use super::*;

impl VirtualMachine {
    pub(crate) fn socket_service_port(&mut self, text: String) -> Result<Object, MetorexError> {
        let (service, protocol) = match text.split_once('/') {
            Some((named, kind)) => (named.to_string(), kind.to_string()),
            None => (text.to_string(), "tcp".to_string()),
        };
        let name = std::ffi::CString::new(service).unwrap_or_default();
        let kind = std::ffi::CString::new(protocol).unwrap_or_default();
        let found = unsafe { libc::getservbyname(name.as_ptr(), kind.as_ptr()) };
        if found.is_null() {
            return Ok(Object::Nil);
        }
        let port = unsafe { (*found).s_port };
        Ok(Object::Int(i64::from(u16::from_be(port as u16))))
    }

    pub(crate) fn socket_service_name(
        &mut self,
        text: String,
        port: i64,
    ) -> Result<Object, MetorexError> {
        let kind = std::ffi::CString::new(text.to_string()).unwrap_or_default();
        let found = unsafe { libc::getservbyport((port as u16).to_be() as i32, kind.as_ptr()) };
        if found.is_null() {
            return Ok(Object::Nil);
        }
        let named = unsafe { std::ffi::CStr::from_ptr((*found).s_name) };
        Ok(Object::string(named.to_string_lossy().to_string()))
    }

    /// What a socket setting holds right now, as the bytes the
    /// operating system keeps it in. `text` names the level and the
    /// option, and `port` how many bytes to read back.
    pub(crate) fn socket_option(
        &mut self,
        handle: u64,
        text: String,
        port: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut parts = text.split('/');
        let level: libc::c_int = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        let name: libc::c_int = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        let Some(descriptor) = self.socket_descriptor(handle) else {
            return Err(refused(position, "option of a closed socket".to_string()));
        };
        let wanted = if port > 0 { port as usize } else { 4 };
        let mut buffer = vec![0u8; wanted];
        let mut size = wanted as libc::socklen_t;
        let answered = unsafe {
            libc::getsockopt(
                descriptor,
                level,
                name,
                buffer.as_mut_ptr() as *mut libc::c_void,
                &mut size,
            )
        };
        if answered < 0 {
            let problem = std::io::Error::last_os_error();
            return Err(crate::vm::errors::simple_exception(
                errno_class(&problem),
                &format!(
                    "{} - getsockopt(2)",
                    crate::vm::init::errno_description(problem.raw_os_error().unwrap_or(0))
                ),
                position,
            ));
        }
        buffer.truncate(size as usize);
        Ok(crate::vm::native_methods::pack_format::bytes_to_string(
            &buffer,
        ))
    }

    /// Change a socket setting, with `text` naming the level, the
    /// option, and the bytes to write, separated by slashes.
    pub(crate) fn socket_set_option(
        &mut self,
        handle: u64,
        text: String,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut parts = text.splitn(3, '/');
        let level: libc::c_int = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        let name: libc::c_int = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        let bytes =
            crate::vm::native_methods::pack_format::string_to_bytes(parts.next().unwrap_or(""));
        let Some(descriptor) = self.socket_descriptor(handle) else {
            return Err(refused(position, "option of a closed socket".to_string()));
        };
        let answered = unsafe {
            libc::setsockopt(
                descriptor,
                level,
                name,
                bytes.as_ptr() as *const libc::c_void,
                bytes.len() as libc::socklen_t,
            )
        };
        if answered < 0 {
            let problem = std::io::Error::last_os_error();
            return Err(crate::vm::errors::simple_exception(
                errno_class(&problem),
                &format!(
                    "{} - setsockopt(2)",
                    crate::vm::init::errno_description(problem.raw_os_error().unwrap_or(0))
                ),
                position,
            ));
        }
        Ok(Object::Int(0))
    }

    /// Every network interface this machine has, one line per entry,
    /// written as name/flags/index/address/netmask/broadcast.
    pub(crate) fn socket_interfaces(&mut self) -> Result<Object, MetorexError> {
        let mut held: *mut libc::ifaddrs = std::ptr::null_mut();
        if unsafe { libc::getifaddrs(&mut held) } != 0 {
            return Ok(Object::array(Vec::new()));
        }
        let mut found = Vec::new();
        let mut walking = held;
        while !walking.is_null() {
            let entry = unsafe { &*walking };
            walking = entry.ifa_next;
            if entry.ifa_name.is_null() {
                continue;
            }
            let name = unsafe { std::ffi::CStr::from_ptr(entry.ifa_name) }
                .to_string_lossy()
                .to_string();
            let index = unsafe {
                let spelled = std::ffi::CString::new(name.clone()).unwrap_or_default();
                libc::if_nametoindex(spelled.as_ptr())
            };
            found.push(Object::array(vec![
                Object::string(name),
                Object::Int(i64::from(entry.ifa_flags)),
                Object::Int(i64::from(index)),
                address_of_sockaddr(entry.ifa_addr),
                address_of_sockaddr(entry.ifa_netmask),
                address_of_sockaddr(point_to_point_address(entry)),
            ]));
        }
        unsafe { libc::freeifaddrs(held) };
        Ok(Object::array(found))
    }
}
