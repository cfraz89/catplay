use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    time::Instant,
};

pub trait BonjourEntryType: Send + Sync + 'static {
    const SERVICE_TYPE: &'static str;
    type Entry: Send + Sync + Clone + Eq + std::fmt::Debug;

    fn from_props(data: &HashMap<String, String>) -> Self::Entry;

    fn to_props(data: Self::Entry, output: &mut HashMap<String, String>);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BonjourMeta {
    pub iface: String,
    pub hostname: String,
    pub port: u16,
    pub ips: Vec<IpAddr>,
    pub addrs: Vec<SocketAddr>,
    pub txt: HashMap<String, String>,

    pub service_type: String,
    pub instance_name: String,
    pub fullname: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BonjourEntry<T> {
    pub meta: BonjourMeta,
    pub data: T,
}

impl<T> BonjourEntry<T> {
    pub fn new(meta: BonjourMeta, data: T) -> Self {
        Self { meta, data }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BonjourCached<T> {
    pub entry: BonjourEntry<T>,
    pub masked: bool,
    pub last_ping: Option<Instant>,
}

impl<T> BonjourCached<T> {
    pub fn new(entry: BonjourEntry<T>) -> Self {
        Self {
            entry,
            masked: false,
            last_ping: None,
        }
    }
}

pub fn get_bonjour_txt_optional(txt: &HashMap<String, String>, key: &str) -> String {
    txt.get(key).cloned().unwrap_or_default()
}
