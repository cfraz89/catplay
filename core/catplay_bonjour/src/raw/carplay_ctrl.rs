use std::net::Ipv6Addr;

use crate::raw::{TYPE_AAAA, TYPE_PTR, TYPE_SRV, TYPE_TXT};

use super::MdnsBuilder;

#[derive(Debug, Clone)]
pub struct CarplayCtrlAnnounceParams<'a> {
    pub instance: &'a str,    // "iPhone SIM"
    pub host: &'a str,        // "iPhone-SIM"
    pub port: u16,            // 50013
    pub addr_v6_ll: Ipv6Addr, // fe80::....
    pub srcvers: &'a str,     // "925.5.1"
    pub device_id: &'a str,   // "80:b4:89:3c:01:65"
    pub model: &'a str,       // "D84AP"
    pub ttl: u32,             // Apple: 4500
}

impl<'a> CarplayCtrlAnnounceParams<'a> {
    pub fn build_announce(&self) -> Vec<u8> {
        let instance_fqdn = format!("{}._carplay-ctrl._tcp.local.", self.instance);
        let service_fqdn = "_carplay-ctrl._tcp.local.";
        let services_fqdn = "_services._dns-sd._udp.local.";
        let host_fqdn = format!("{}.local.", self.host);
        let device_info_fqdn = format!("{}._device-info._tcp.local.", self.instance);

        let ip6_arpa_fqdn = ipv6_to_ip6_arpa(self.addr_v6_ll);

        let mut b = MdnsBuilder::new(1500);

        // 1) SRV instance → host (flush)
        b.rr_srv(&instance_fqdn, self.ttl, self.port, &host_fqdn, true);
        // 2) AAAA host (flush)
        b.rr_aaaa(&host_fqdn, self.ttl, self.addr_v6_ll.octets(), true);
        // 3) PTR ip6.arpa → host (flush)
        b.rr_ptr(&ip6_arpa_fqdn, self.ttl, &host_fqdn, true);
        // 4) TXT instance (flush)
        let txt_id = format!("id={}", self.device_id);
        let txt_srcvers = format!("srcvers={}", self.srcvers);
        b.rr_txt(&instance_fqdn, self.ttl, &[txt_id.as_bytes(), txt_srcvers.as_bytes()], true);
        // 5) PTR service → instance (no flush)
        b.rr_ptr(service_fqdn, self.ttl, &instance_fqdn, false);
        // 6) PTR _services → service (no flush)
        b.rr_ptr(services_fqdn, self.ttl, service_fqdn, false);
        // 7) TXT device-info (no flush)
        let txt_model = format!("model={}", self.model);
        b.rr_txt(&device_info_fqdn, self.ttl, &[txt_model.as_bytes()], false);

        // =========================================================
        // ADDITIONAL (NSEC) — Apple style
        // =========================================================

        // A1) NSEC instance: TXT + SRV
        b.rr_nsec(&instance_fqdn, self.ttl, &instance_fqdn, &[TYPE_TXT, TYPE_SRV], true);
        // A2) NSEC host: AAAA
        b.rr_nsec(&host_fqdn, self.ttl, &host_fqdn, &[TYPE_AAAA], true);
        // A3) NSEC ip6.arpa: PTR
        b.rr_nsec(&ip6_arpa_fqdn, self.ttl, &ip6_arpa_fqdn, &[TYPE_PTR], true);

        let mut vec = Vec::new();
        vec.extend_from_slice(&b.build_header());
        vec.extend_from_slice(&b.build_data());

        vec
    }
}

fn ipv6_to_ip6_arpa(addr: Ipv6Addr) -> String {
    let mut s = String::with_capacity(74);
    fn hex(n: u8) -> char {
        match n {
            0..=9 => (b'0' + n) as char,
            _ => (b'A' + (n - 10)) as char,
        }
    }

    for b in addr.octets().iter().rev() {
        s.push(hex(*b & 0x0F));
        s.push('.');
        s.push(hex(b >> 4));
        s.push('.');
    }
    s.push_str("ip6.arpa.");
    s
}

#[test]
fn test() {
    let payload = CarplayCtrlAnnounceParams {
        instance: "iPhone SIM",
        host: "iPhone-SIM",
        port: 50013,
        addr_v6_ll: "fe80::7c20:b2f1:fecc:d351".parse::<Ipv6Addr>().unwrap(),
        srcvers: "925.5.1",
        model: "D84AP",
        device_id: "80:b4:89:3c:01:65",
        ttl: 4500,
    }
    .build_announce();
    assert_eq!(payload.len(), 399);
    assert_eq!(
        payload,
        [
            0, 0, 132, 0, 0, 0, 0, 7, 0, 0, 0, 3, 10, 105, 80, 104, 111, 110, 101, 32, 83, 73, 77, 13, 95, 99, 97, 114, 112, 108, 97, 121,
            45, 99, 116, 114, 108, 4, 95, 116, 99, 112, 5, 108, 111, 99, 97, 108, 0, 0, 33, 128, 1, 0, 0, 17, 148, 0, 19, 0, 0, 0, 0, 195,
            93, 10, 105, 80, 104, 111, 110, 101, 45, 83, 73, 77, 192, 42, 192, 65, 0, 28, 128, 1, 0, 0, 17, 148, 0, 16, 254, 128, 0, 0, 0,
            0, 0, 0, 124, 32, 178, 241, 254, 204, 211, 81, 1, 49, 1, 53, 1, 51, 1, 68, 1, 67, 1, 67, 1, 69, 1, 70, 1, 49, 1, 70, 1, 50, 1,
            66, 1, 48, 1, 50, 1, 67, 1, 55, 1, 48, 1, 48, 1, 48, 1, 48, 1, 48, 1, 48, 1, 48, 1, 48, 1, 48, 1, 48, 1, 48, 1, 48, 1, 48, 1,
            56, 1, 69, 1, 70, 3, 105, 112, 54, 4, 97, 114, 112, 97, 0, 0, 12, 128, 1, 0, 0, 17, 148, 0, 2, 192, 65, 192, 12, 0, 16, 128, 1,
            0, 0, 17, 148, 0, 37, 20, 105, 100, 61, 56, 48, 58, 98, 52, 58, 56, 57, 58, 51, 99, 58, 48, 49, 58, 54, 53, 15, 115, 114, 99,
            118, 101, 114, 115, 61, 57, 50, 53, 46, 53, 46, 49, 192, 23, 0, 12, 0, 1, 0, 0, 17, 148, 0, 2, 192, 12, 9, 95, 115, 101, 114,
            118, 105, 99, 101, 115, 7, 95, 100, 110, 115, 45, 115, 100, 4, 95, 117, 100, 112, 192, 42, 0, 12, 0, 1, 0, 0, 17, 148, 0, 2,
            192, 23, 10, 105, 80, 104, 111, 110, 101, 32, 83, 73, 77, 12, 95, 100, 101, 118, 105, 99, 101, 45, 105, 110, 102, 111, 192, 37,
            0, 16, 0, 1, 0, 0, 17, 148, 0, 12, 11, 109, 111, 100, 101, 108, 61, 68, 56, 52, 65, 80, 192, 12, 0, 47, 128, 1, 0, 0, 17, 148,
            0, 9, 192, 12, 0, 5, 0, 0, 128, 0, 64, 192, 65, 0, 47, 128, 1, 0, 0, 17, 148, 0, 8, 192, 65, 0, 4, 0, 0, 0, 8, 192, 106, 0, 47,
            128, 1, 0, 0, 17, 148, 0, 6, 192, 106, 0, 2, 0, 8
        ]
    );
}
