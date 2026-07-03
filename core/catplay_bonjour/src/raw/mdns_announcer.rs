use std::{
    ffi::{CStr, CString},
    io, mem,
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, UdpSocket},
    os::fd::{FromRawFd, OwnedFd},
    ptr,
};

use socket2::{Domain, Protocol, Socket, Type};

use crate::{BonjourType, MulticastFamily};

pub const MDNS_PORT: u16 = 5353;
pub const MDNS_MCAST_V4: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
pub const MDNS_MCAST_V6: Ipv6Addr = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0x00fb);

pub struct MdnsAnnouncer {
    ifindex: u32,
    iface_v4_addrs: Vec<Ipv4Addr>,
    pub ll_addr: Option<Ipv6Addr>,
    sock_v4: Option<UdpSocket>,
    sock_v6: Option<UdpSocket>,
}

impl MdnsAnnouncer {
    fn if_nametoindex(name: &str) -> io::Result<u32> {
        let c = CString::new(name)?;
        let idx = unsafe { libc::if_nametoindex(c.as_ptr()) };
        if idx == 0 { Err(io::Error::last_os_error()) } else { Ok(idx) }
    }

    fn probe_mdns_v6(iface: &str) -> io::Result<()> {
        let ifindex = Self::if_nametoindex(iface)?;

        let fd = unsafe { libc::socket(libc::AF_INET6, libc::SOCK_DGRAM, 0) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let _guard = unsafe { OwnedFd::from_raw_fd(fd) };

        let ret = unsafe {
            libc::setsockopt(
                fd,
                libc::IPPROTO_IPV6,
                libc::IPV6_MULTICAST_IF,
                &ifindex as *const _ as *const libc::c_void,
                mem::size_of::<u32>() as _,
            )
        };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }

        let mut addr: libc::sockaddr_in6 = unsafe { mem::zeroed() };
        addr.sin6_family = libc::AF_INET6 as _;
        addr.sin6_port = u16::to_be(5354);
        addr.sin6_scope_id = ifindex;
        addr.sin6_addr = libc::in6_addr {
            s6_addr: MDNS_MCAST_V6.octets(),
        };
        let buf = [b'x'];
        let ret = unsafe {
            libc::sendto(
                fd,
                buf.as_ptr() as *const _,
                buf.len(),
                0,
                &addr as *const _ as *const libc::sockaddr,
                mem::size_of::<libc::sockaddr_in6>() as _,
            )
        };
        if ret < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
    }

    fn probe_mdns_v4(src: Ipv4Addr) -> io::Result<()> {
        let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let _guard = unsafe { OwnedFd::from_raw_fd(fd) };

        let src_in = libc::in_addr {
            s_addr: u32::from_ne_bytes(src.octets()),
        };
        let ret = unsafe {
            libc::setsockopt(
                fd,
                libc::IPPROTO_IP,
                libc::IP_MULTICAST_IF,
                &src_in as *const _ as *const libc::c_void,
                mem::size_of::<libc::in_addr>() as _,
            )
        };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }

        let mut addr: libc::sockaddr_in = unsafe { mem::zeroed() };
        addr.sin_family = libc::AF_INET as _;
        addr.sin_port = u16::to_be(5354);
        addr.sin_addr = libc::in_addr {
            s_addr: u32::from_ne_bytes(MDNS_MCAST_V4.octets()),
        };
        let buf = [b'x'];
        let ret = unsafe {
            libc::sendto(
                fd,
                buf.as_ptr() as *const _,
                buf.len(),
                0,
                &addr as *const _ as *const libc::sockaddr,
                mem::size_of::<libc::sockaddr_in>() as _,
            )
        };
        if ret < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
    }

    pub fn probe_multicast_stability(iface: &str, btype: BonjourType) -> Result<(), (MulticastFamily, io::Error)> {
        let (v4, v6) = Self::snapshot_iface_addrs(iface).map_err(|e| (MulticastFamily::Ipv6, e))?;
        let mut attempted = false;

        if v6.is_some() && (btype == BonjourType::Ipv6 || btype == BonjourType::Ipv6AndIpv4) {
            attempted = true;
            let result = Self::probe_mdns_v6(iface);
            if let Err(err) = result {
                return Err((MulticastFamily::Ipv6, err));
            }
        }

        if let Some(src) = v4.first()
            && (btype == BonjourType::Ipv4 || btype == BonjourType::Ipv6AndIpv4)
        {
            attempted = true;
            let result = Self::probe_mdns_v4(*src);
            if let Err(err) = result {
                return Err((MulticastFamily::Ipv4, err));
            }
        }

        if !attempted {
            return Err((
                MulticastFamily::Ipv6,
                io::Error::new(
                    io::ErrorKind::AddrNotAvailable,
                    format!("interface {iface} has no IPv4 or IPv6-link-local address yet"),
                ),
            ));
        }

        Ok(())
    }

    fn set_reuse_port(fd: std::os::fd::RawFd) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            let ret = unsafe {
                libc::setsockopt(
                    fd,
                    libc::SOL_SOCKET,
                    libc::SO_REUSEPORT,
                    &1 as *const _ as *const _,
                    std::mem::size_of::<libc::c_int>() as _,
                )
            };
            if ret < 0 {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(())
    }

    fn bind_to_device(fd: std::os::fd::RawFd, iface: &str) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            let iface_c = CString::new(iface)?;
            let ret = unsafe {
                libc::setsockopt(
                    fd,
                    libc::SOL_SOCKET,
                    libc::SO_BINDTODEVICE,
                    iface_c.as_ptr() as *const libc::c_void,
                    (iface_c.as_bytes_with_nul().len()) as libc::socklen_t,
                )
            };
            if ret < 0 {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(())
    }

    pub fn get_ipv6_link_local_addr(iface: &str) -> io::Result<Ipv6Addr> {
        let iface_c = CString::new(iface)?;
        unsafe {
            let mut ifap: *mut libc::ifaddrs = ptr::null_mut();
            if libc::getifaddrs(&mut ifap) != 0 {
                return Err(io::Error::last_os_error());
            }
            struct FreeOnDrop(*mut libc::ifaddrs);
            impl Drop for FreeOnDrop {
                fn drop(&mut self) {
                    unsafe { libc::freeifaddrs(self.0) }
                }
            }
            let _guard = FreeOnDrop(ifap);

            let mut p = ifap;
            while !p.is_null() {
                let ifa = &*p;

                if !ifa.ifa_name.is_null() {
                    let name = CStr::from_ptr(ifa.ifa_name);
                    if name.to_bytes() == iface_c.as_bytes()
                        && !ifa.ifa_addr.is_null()
                        && (*ifa.ifa_addr).sa_family as i32 == libc::AF_INET6
                    {
                        let sa6 = &*(ifa.ifa_addr as *const libc::sockaddr_in6);
                        let oct = sa6.sin6_addr.s6_addr;
                        let is_link_local = oct[0] == 0xfe && (oct[1] & 0xc0) == 0x80;
                        if is_link_local {
                            return Ok(Ipv6Addr::from(oct));
                        }
                    }
                }

                p = ifa.ifa_next;
            }

            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("no IPv6 link-local address found on interface {iface}"),
            ))
        }
    }

    fn get_ipv4_addrs(iface: &str) -> io::Result<Vec<Ipv4Addr>> {
        let addrs = if_addrs::get_if_addrs()?
            .into_iter()
            .filter(|a| a.name == iface)
            .filter_map(|a| match a.addr.ip() {
                std::net::IpAddr::V4(v4) => Some(v4),
                _ => None,
            })
            .collect::<Vec<_>>();

        Ok(addrs)
    }

    pub fn snapshot_iface_addrs(iface: &str) -> io::Result<(Vec<Ipv4Addr>, Option<Ipv6Addr>)> {
        let v4 = Self::get_ipv4_addrs(iface)?;
        let v6 = Self::get_ipv6_link_local_addr(iface).ok();
        Ok((v4, v6))
    }

    fn new_v4_socket(iface: &str, v4_addrs: &[Ipv4Addr]) -> io::Result<Option<UdpSocket>> {
        if v4_addrs.is_empty() {
            return Ok(None);
        }

        let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        sock.set_reuse_address(true)?;
        Self::set_reuse_port(sock.as_raw_fd())?;
        Self::bind_to_device(sock.as_raw_fd(), iface)?;
        sock.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, MDNS_PORT).into())?;

        for addr in v4_addrs {
            sock.join_multicast_v4(&MDNS_MCAST_V4, addr)?;
        }
        if let Some(src) = v4_addrs.first() {
            sock.set_multicast_if_v4(src)?;
        }

        sock.set_multicast_loop_v4(true)?;
        sock.set_multicast_ttl_v4(255)?;
        sock.set_nonblocking(true)?;
        Ok(Some(sock.into()))
    }

    fn new_v6_socket(iface: &str, ifindex: u32) -> io::Result<UdpSocket> {
        let sock = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
        sock.set_reuse_address(true)?;
        Self::set_reuse_port(sock.as_raw_fd())?;
        Self::bind_to_device(sock.as_raw_fd(), iface)?;
        sock.set_only_v6(true)?;
        sock.bind(&SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, MDNS_PORT, 0, 0).into())?;
        sock.join_multicast_v6(&MDNS_MCAST_V6, ifindex)?;
        sock.set_multicast_if_v6(ifindex)?;
        sock.set_multicast_loop_v6(true)?;
        sock.set_nonblocking(true)?;
        Ok(sock.into())
    }

    pub fn new(iface: &str) -> io::Result<Self> {
        let ifindex = Self::if_nametoindex(iface)?;
        let iface_v4_addrs = Self::get_ipv4_addrs(iface)?;
        let ll_addr = Self::get_ipv6_link_local_addr(iface).ok();
        let sock_v4 = Self::new_v4_socket(iface, &iface_v4_addrs)?;
        let sock_v6 = Self::new_v6_socket(iface, ifindex).ok();

        Ok(Self {
            ifindex,
            iface_v4_addrs,
            ll_addr,
            sock_v4,
            sock_v6,
        })
    }

    pub fn ifindex(&self) -> u32 {
        self.ifindex
    }

    pub fn iface_v4_addrs(&self) -> &[Ipv4Addr] {
        &self.iface_v4_addrs
    }

    pub fn clone_v4_socket(&self) -> io::Result<Option<UdpSocket>> {
        self.sock_v4.as_ref().map(UdpSocket::try_clone).transpose()
    }

    pub fn clone_v6_socket(&self) -> io::Result<Option<UdpSocket>> {
        self.sock_v6.as_ref().map(UdpSocket::try_clone).transpose()
    }

    pub fn send_mdns(&self, payload: &[u8]) -> io::Result<()> {
        let mut sent = false;

        if let Some(sock_v4) = &self.sock_v4 {
            let _ = sock_v4.send_to(payload, SocketAddrV4::new(MDNS_MCAST_V4, MDNS_PORT));
            sent = true;
        }

        if let Some(sock_v6) = &self.sock_v6 {
            let _ = sock_v6.send_to(payload, SocketAddrV6::new(MDNS_MCAST_V6, MDNS_PORT, 0, self.ifindex));
            sent = true;
        }

        if sent {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::AddrNotAvailable, "no multicast sockets available"))
        }
    }
}

use std::os::fd::AsRawFd;
