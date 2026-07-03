use std::{
    io,
    net::TcpListener,
    os::fd::{FromRawFd, IntoRawFd, OwnedFd},
};

use libc::{
    AF_INET, AF_INET6, IPV6_V6ONLY, SO_BINDTODEVICE, SO_REUSEADDR, SO_REUSEPORT, SOCK_STREAM, SOL_SOCKET, bind, c_int, c_void, in_addr,
    listen, setsockopt, sockaddr, sockaddr_in, sockaddr_in6, socket, socklen_t,
};

pub fn bind_iface_ipv6(port: u16, iface: &str) -> io::Result<TcpListener> {
    const BACKLOG: usize = 128;

    unsafe {
        let fd = socket(AF_INET6, SOCK_STREAM, 0);
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let guard = OwnedFd::from_raw_fd(fd);

        let mut ifname = iface.to_string();
        ifname.push('\0');
        if setsockopt(
            fd,
            SOL_SOCKET,
            SO_BINDTODEVICE,
            ifname.as_ptr() as *const c_void,
            ifname.len() as socklen_t,
        ) != 0
        {
            return Err(io::Error::last_os_error());
        }

        if setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_REUSEADDR,
            &1 as *const _ as *const c_void,
            std::mem::size_of::<c_int>() as libc::socklen_t,
        ) != 0
        {
            return Err(io::Error::last_os_error());
        }

        if setsockopt(
            fd,
            SOL_SOCKET,
            SO_REUSEPORT,
            &1 as *const _ as *const c_void,
            std::mem::size_of::<c_int>() as libc::socklen_t,
        ) != 0
        {
            return Err(io::Error::last_os_error());
        }

        let v6only: c_int = 0;
        if setsockopt(
            fd,
            libc::IPPROTO_IPV6,
            IPV6_V6ONLY,
            &v6only as *const _ as *const c_void,
            std::mem::size_of::<c_int>() as libc::socklen_t,
        ) != 0
        {
            return Err(io::Error::last_os_error());
        }

        let addr = sockaddr_in6 {
            sin6_family: AF_INET6 as u16,
            sin6_port: port.to_be(),
            sin6_flowinfo: 0,
            sin6_addr: libc::in6_addr { s6_addr: [0; 16] }, // ::0
            sin6_scope_id: 0,
        };

        if bind(fd, &addr as *const _ as *const sockaddr, std::mem::size_of::<sockaddr_in6>() as u32) != 0 {
            return Err(io::Error::last_os_error());
        }

        if listen(fd, BACKLOG as _) != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(TcpListener::from_raw_fd(guard.into_raw_fd()))
    }
}

pub fn bind_iface_ipv4(port: u16, iface: &str) -> io::Result<TcpListener> {
    const BACKLOG: usize = 128;

    unsafe {
        let fd = socket(AF_INET, SOCK_STREAM, 0);

        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let guard = OwnedFd::from_raw_fd(fd);

        if setsockopt(
            fd,
            SOL_SOCKET,
            SO_REUSEADDR,
            &1 as *const _ as *const c_void,
            std::mem::size_of::<c_int>() as libc::socklen_t,
        ) != 0
        {
            return Err(io::Error::last_os_error());
        }

        if setsockopt(
            fd,
            SOL_SOCKET,
            SO_REUSEPORT,
            &1 as *const _ as *const c_void,
            std::mem::size_of::<c_int>() as libc::socklen_t,
        ) != 0
        {
            return Err(io::Error::last_os_error());
        }

        let mut ifname = iface.to_string();
        ifname.push('\0');
        if setsockopt(
            fd,
            SOL_SOCKET,
            SO_BINDTODEVICE,
            ifname.as_ptr() as *const c_void,
            ifname.len() as libc::socklen_t,
        ) != 0
        {
            return Err(io::Error::last_os_error());
        }

        let addr = sockaddr_in {
            sin_family: AF_INET as _,
            sin_port: port.to_be(),
            sin_addr: in_addr { s_addr: 0 },
            sin_zero: [0u8; _],
        };

        if bind(fd, &addr as *const _ as *const sockaddr, std::mem::size_of::<sockaddr_in6>() as u32) != 0 {
            return Err(io::Error::last_os_error());
        }

        if listen(fd, BACKLOG as _) != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(TcpListener::from_raw_fd(guard.into_raw_fd()))
    }
}

#[test]
fn test_bind_ipv4() {
    let _listener = bind_iface_ipv4(5151, "lo").unwrap();
    assert!(bind_iface_ipv4(5152, "notfound").is_err());
}

#[test]
fn test_bind_ipv6() {
    let _listener = bind_iface_ipv6(5153, "lo").unwrap();
    assert!(bind_iface_ipv6(5154, "notfound").is_err());
}
