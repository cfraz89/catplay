use libc::{getpeername, sockaddr, socklen_t};
use macaddr::MacAddr6;
use std::io::{self, Result};
use std::mem::{size_of, zeroed};
use std::os::fd::RawFd;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct SockAddrRc {
    rc_family: libc::sa_family_t,
    rc_bdaddr: BdAddr,
    rc_channel: u8,
}

// Bluetooth MAC address
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct BdAddr {
    b: [u8; 6], // little-endian MAC
}

pub fn get_rfcomm_local_mac(fd: RawFd) -> Result<MacAddr6> {
    let mut addr: SockAddrRc = unsafe { std::mem::zeroed() };
    let mut len = std::mem::size_of::<SockAddrRc>() as socklen_t;

    let ret = unsafe { libc::getsockname(fd, &mut addr as *mut _ as *mut sockaddr, &mut len) };

    if ret < 0 {
        return Err(std::io::Error::last_os_error());
    }

    Ok(bdaddr_to_mac(addr.rc_bdaddr.b))
}

pub fn get_rfcomm_peer_mac(fd: RawFd) -> Result<MacAddr6> {
    let mut addr: SockAddrRc = unsafe { zeroed() };
    let mut len = size_of::<SockAddrRc>() as socklen_t;

    let ret = unsafe { getpeername(fd, &mut addr as *mut _ as *mut sockaddr, &mut len) };

    if ret < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(bdaddr_to_mac(addr.rc_bdaddr.b))
}

// fn bdaddr_to_string(b: [u8; 6]) -> String {
//     // little endian
//     format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", b[5], b[4], b[3], b[2], b[1], b[0])
// }

fn bdaddr_to_mac(b: [u8; 6]) -> MacAddr6 {
    // little endian
    MacAddr6::new(b[5], b[4], b[3], b[2], b[1], b[0])
}
