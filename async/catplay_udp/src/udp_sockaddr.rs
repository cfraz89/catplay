use libc::{sockaddr_in, sockaddr_in6, sockaddr_storage};
use std::{
    mem::MaybeUninit,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

pub fn sockaddr_in_to_std(addr: &libc::sockaddr_in) -> SocketAddr {
    let ip = Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr));
    let port = u16::from_be(addr.sin_port);
    SocketAddr::V4(SocketAddrV4::new(ip, port))
}

pub fn sockaddr_in6_to_std(addr: &libc::sockaddr_in6) -> SocketAddr {
    let ip = Ipv6Addr::from(addr.sin6_addr.s6_addr);
    let port = u16::from_be(addr.sin6_port);
    SocketAddr::V6(SocketAddrV6::new(ip, port, addr.sin6_flowinfo, addr.sin6_scope_id))
}

pub fn sockaddr_storage_to_std(storage: &libc::sockaddr_storage) -> Option<SocketAddr> {
    match storage.ss_family as i32 {
        libc::AF_INET => {
            let addr = unsafe { &*(storage as *const _ as *const libc::sockaddr_in) };
            Some(sockaddr_in_to_std(addr))
        }
        libc::AF_INET6 => {
            let addr = unsafe { &*(storage as *const _ as *const libc::sockaddr_in6) };
            Some(sockaddr_in6_to_std(addr))
        }
        _ => None,
    }
}

pub fn socketaddr_to_storage(addr: &SocketAddr) -> (sockaddr_storage, libc::socklen_t) {
    unsafe {
        let mut storage = MaybeUninit::<sockaddr_storage>::uninit();

        let len;

        match addr {
            SocketAddr::V4(a) => {
                let in_addr = &mut *(&mut storage as *mut _ as *mut sockaddr_in);
                in_addr.sin_family = libc::AF_INET as u16;
                in_addr.sin_port = a.port().to_be();
                in_addr.sin_addr.s_addr = u32::from_ne_bytes(a.ip().octets());
                len = size_of::<sockaddr_in>() as libc::socklen_t;
            }
            SocketAddr::V6(a) => {
                let in6_addr = &mut *(&mut storage as *mut _ as *mut sockaddr_in6);
                in6_addr.sin6_family = libc::AF_INET6 as u16;
                in6_addr.sin6_port = a.port().to_be();
                in6_addr.sin6_flowinfo = a.flowinfo();
                in6_addr.sin6_scope_id = a.scope_id();
                in6_addr.sin6_addr.s6_addr = a.ip().octets();
                len = size_of::<sockaddr_in6>() as libc::socklen_t;
            }
        }
        (storage.assume_init(), len)
    }
}
