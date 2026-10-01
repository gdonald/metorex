// The calls that reach the operating system directly.

/// Send one descriptor over a socket named by a path, alongside a single
/// byte, which is what the message needs to carry the control data.
pub(crate) fn send_descriptor(socket: i32, descriptor: i32) -> isize {
    let mut byte = [0u8; 1];
    let mut piece = libc::iovec {
        iov_base: byte.as_mut_ptr() as *mut libc::c_void,
        iov_len: 1,
    };
    let space = unsafe { libc::CMSG_SPACE(std::mem::size_of::<libc::c_int>() as u32) } as usize;
    let mut control = vec![0u8; space];
    let mut message: libc::msghdr = unsafe { std::mem::zeroed() };
    message.msg_iov = &mut piece;
    message.msg_iovlen = 1;
    message.msg_control = control.as_mut_ptr() as *mut libc::c_void;
    message.msg_controllen = space as _;
    unsafe {
        let header = libc::CMSG_FIRSTHDR(&message);
        (*header).cmsg_level = libc::SOL_SOCKET;
        (*header).cmsg_type = libc::SCM_RIGHTS;
        (*header).cmsg_len = libc::CMSG_LEN(std::mem::size_of::<libc::c_int>() as u32) as _;
        std::ptr::write(libc::CMSG_DATA(header) as *mut libc::c_int, descriptor);
        libc::sendmsg(socket, &message, 0)
    }
}

/// Take one descriptor handed over a socket named by a path, or -1 when none
/// arrived.
pub(crate) fn receive_descriptor(socket: i32) -> i32 {
    let mut byte = [0u8; 1];
    let mut piece = libc::iovec {
        iov_base: byte.as_mut_ptr() as *mut libc::c_void,
        iov_len: 1,
    };
    let space = unsafe { libc::CMSG_SPACE(std::mem::size_of::<libc::c_int>() as u32) } as usize;
    let mut control = vec![0u8; space];
    let mut message: libc::msghdr = unsafe { std::mem::zeroed() };
    message.msg_iov = &mut piece;
    message.msg_iovlen = 1;
    message.msg_control = control.as_mut_ptr() as *mut libc::c_void;
    message.msg_controllen = space as _;
    unsafe {
        if libc::recvmsg(socket, &mut message, 0) < 0 {
            return -1;
        }
        let header = libc::CMSG_FIRSTHDR(&message);
        if header.is_null()
            || (*header).cmsg_level != libc::SOL_SOCKET
            || (*header).cmsg_type != libc::SCM_RIGHTS
        {
            return -1;
        }
        std::ptr::read(libc::CMSG_DATA(header) as *const libc::c_int)
    }
}

/// The number the operating system names a socket constant by. A name this
/// platform does not carry answers None, and the socket library keeps its own
/// number for that one.
pub(crate) fn socket_constant(name: &str) -> Option<i64> {
    let held = match name {
        "AF_UNSPEC" => libc::AF_UNSPEC,
        "AF_UNIX" | "AF_LOCAL" => libc::AF_UNIX,
        "AF_INET" => libc::AF_INET,
        "AF_INET6" => libc::AF_INET6,
        "PF_UNSPEC" => libc::PF_UNSPEC,
        "PF_UNIX" | "PF_LOCAL" => libc::PF_UNIX,
        "PF_INET" => libc::PF_INET,
        "PF_INET6" => libc::PF_INET6,
        "SOCK_STREAM" => libc::SOCK_STREAM,
        "SOCK_DGRAM" => libc::SOCK_DGRAM,
        "SOCK_RAW" => libc::SOCK_RAW,
        "SOCK_RDM" => libc::SOCK_RDM,
        "SOCK_SEQPACKET" => libc::SOCK_SEQPACKET,
        "IPPROTO_IP" => libc::IPPROTO_IP,
        "IPPROTO_ICMP" => libc::IPPROTO_ICMP,
        "IPPROTO_TCP" => libc::IPPROTO_TCP,
        "IPPROTO_UDP" => libc::IPPROTO_UDP,
        "IPPROTO_IPV6" => libc::IPPROTO_IPV6,
        "IPPROTO_RAW" => libc::IPPROTO_RAW,
        "IPPROTO_AH" => libc::IPPROTO_AH,
        "IPPROTO_DSTOPTS" => libc::IPPROTO_DSTOPTS,
        "IPPROTO_EGP" => libc::IPPROTO_EGP,
        "IPPROTO_ESP" => libc::IPPROTO_ESP,
        "IPPROTO_FRAGMENT" => libc::IPPROTO_FRAGMENT,
        "IPPROTO_HOPOPTS" => libc::IPPROTO_HOPOPTS,
        "IPPROTO_ICMPV6" => libc::IPPROTO_ICMPV6,
        "IPPROTO_IDP" => libc::IPPROTO_IDP,
        "IPPROTO_IGMP" => libc::IPPROTO_IGMP,
        "IPPROTO_NONE" => libc::IPPROTO_NONE,
        "IPPROTO_PUP" => libc::IPPROTO_PUP,
        "IPPROTO_ROUTING" => libc::IPPROTO_ROUTING,
        "IPPROTO_TP" => libc::IPPROTO_TP,
        // Linux has raised this between kernel versions, so the number the
        // C library crate carries may not be the one the headers name.
        #[cfg(target_os = "macos")]
        "IPPROTO_MAX" => libc::IPPROTO_MAX,
        #[cfg(target_os = "macos")]
        "IPPROTO_EON" => libc::IPPROTO_EON,
        #[cfg(target_os = "macos")]
        "IPPROTO_GGP" => libc::IPPROTO_GGP,
        #[cfg(target_os = "macos")]
        "IPPROTO_HELLO" => libc::IPPROTO_HELLO,
        #[cfg(target_os = "macos")]
        "IPPROTO_ND" => libc::IPPROTO_ND,
        #[cfg(target_os = "macos")]
        "IPPROTO_XTP" => libc::IPPROTO_XTP,
        "SOL_SOCKET" => libc::SOL_SOCKET,
        "SO_DEBUG" => libc::SO_DEBUG,
        "SO_ACCEPTCONN" => libc::SO_ACCEPTCONN,
        "SO_REUSEADDR" => libc::SO_REUSEADDR,
        "SO_KEEPALIVE" => libc::SO_KEEPALIVE,
        "SO_DONTROUTE" => libc::SO_DONTROUTE,
        "SO_BROADCAST" => libc::SO_BROADCAST,
        "SO_OOBINLINE" => libc::SO_OOBINLINE,
        "SO_REUSEPORT" => libc::SO_REUSEPORT,
        "SO_LINGER" => libc::SO_LINGER,
        "SO_SNDBUF" => libc::SO_SNDBUF,
        "SO_RCVBUF" => libc::SO_RCVBUF,
        "SO_TYPE" => libc::SO_TYPE,
        "SO_ERROR" => libc::SO_ERROR,
        "SO_SNDLOWAT" => libc::SO_SNDLOWAT,
        "SO_RCVLOWAT" => libc::SO_RCVLOWAT,
        "SO_SNDTIMEO" => libc::SO_SNDTIMEO,
        "SO_RCVTIMEO" => libc::SO_RCVTIMEO,
        "MSG_OOB" => libc::MSG_OOB,
        "MSG_PEEK" => libc::MSG_PEEK,
        "MSG_DONTROUTE" => libc::MSG_DONTROUTE,
        "MSG_EOR" => libc::MSG_EOR,
        "MSG_TRUNC" => libc::MSG_TRUNC,
        "MSG_CTRUNC" => libc::MSG_CTRUNC,
        "MSG_WAITALL" => libc::MSG_WAITALL,
        "MSG_DONTWAIT" => libc::MSG_DONTWAIT,
        "TCP_NODELAY" => libc::TCP_NODELAY,
        "TCP_MAXSEG" => libc::TCP_MAXSEG,
        "SCM_RIGHTS" => libc::SCM_RIGHTS,
        "EAI_AGAIN" => libc::EAI_AGAIN,
        "EAI_BADFLAGS" => libc::EAI_BADFLAGS,
        "EAI_FAIL" => libc::EAI_FAIL,
        "EAI_FAMILY" => libc::EAI_FAMILY,
        "EAI_MEMORY" => libc::EAI_MEMORY,
        "EAI_NONAME" => libc::EAI_NONAME,
        "EAI_SERVICE" => libc::EAI_SERVICE,
        "EAI_SOCKTYPE" => libc::EAI_SOCKTYPE,
        "EAI_SYSTEM" => libc::EAI_SYSTEM,
        "EAI_OVERFLOW" => libc::EAI_OVERFLOW,
        "IP_TTL" => libc::IP_TTL,
        "IP_MULTICAST_TTL" => libc::IP_MULTICAST_TTL,
        "IP_MULTICAST_LOOP" => libc::IP_MULTICAST_LOOP,
        "IP_ADD_MEMBERSHIP" => libc::IP_ADD_MEMBERSHIP,
        "IP_DROP_MEMBERSHIP" => libc::IP_DROP_MEMBERSHIP,
        "IPV6_V6ONLY" => libc::IPV6_V6ONLY,
        // How an address named by a path is laid out, which is not a
        // constant the C library names but a shape it settles.
        "SOCKADDR_UN_SIZE" => {
            return Some(std::mem::size_of::<libc::sockaddr_un>() as i64);
        }
        "SOCKADDR_UN_PATH_MAX" => {
            let held = libc::sockaddr_un {
                ..unsafe { std::mem::zeroed() }
            };
            return Some(held.sun_path.len() as i64 - 1);
        }
        // BSD writes the length of an address in front of the family it
        // names, where Linux writes the family alone.
        "SOCKADDR_HAS_LEN" => return Some(i64::from(!cfg!(target_os = "linux"))),
        _ => return None,
    };
    Some(i64::from(held))
}

/// An address written the way the system call takes it, with the number of
/// bytes it fills.
pub(crate) fn sockaddr_of(held: &std::net::SocketAddr) -> (Vec<u8>, libc::socklen_t) {
    match held {
        std::net::SocketAddr::V4(address) => {
            let mut written = unsafe { std::mem::zeroed::<libc::sockaddr_in>() };
            written.sin_family = libc::AF_INET as libc::sa_family_t;
            written.sin_port = address.port().to_be();
            written.sin_addr.s_addr = u32::from_ne_bytes(address.ip().octets());
            let size = std::mem::size_of::<libc::sockaddr_in>();
            let bytes =
                unsafe { std::slice::from_raw_parts((&raw const written).cast::<u8>(), size) };
            (bytes.to_vec(), size as libc::socklen_t)
        }
        std::net::SocketAddr::V6(address) => {
            let mut written = unsafe { std::mem::zeroed::<libc::sockaddr_in6>() };
            written.sin6_family = libc::AF_INET6 as libc::sa_family_t;
            written.sin6_port = address.port().to_be();
            written.sin6_addr.s6_addr = address.ip().octets();
            let size = std::mem::size_of::<libc::sockaddr_in6>();
            let bytes =
                unsafe { std::slice::from_raw_parts((&raw const written).cast::<u8>(), size) };
            (bytes.to_vec(), size as libc::socklen_t)
        }
    }
}
