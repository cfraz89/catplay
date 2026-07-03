use dbus::arg::{OwnedFd as DbusOwnedFd, RefArg};
use dbus_crossroads::Crossroads;
use log::debug;
use macaddr::MacAddr6;
use std::{
    collections::HashMap,
    os::fd::{AsFd, AsRawFd, OwnedFd},
    sync::{Arc, Mutex},
};

use crate::{get_rfcomm_local_mac, get_rfcomm_peer_mac};

// pub const IAP_UUID: &str = "02030302-1d19-415f-86f2-22a2106a0a77"; // 0000fef0-0000-1000-8000-00805f9b34fb";
pub const IAP_SERVER_UUID: &str = "00000000-deca-fade-deca-deafdecacaff";
pub const IAP_CLIENT_UUID: &str = "00000000-deca-fade-deca-deafdecacafe";

// iPhone exposes this UUID as "Wireless iAP v2". Doesn't seem to be used for anything.
pub const IAP_V2_UUID: &str = "02030302-1d19-415f-86f2-22a2106a0a77";
pub const IAP_CHANNEL: u16 = 19;

pub const CARPLAY_EIR_ACCESSORY: &str = "ec884348-cd41-40a2-9727-575d50bf1fd3";
pub const CARPLAY_EIR_PHONE: &str = "2d8d2466-e14d-451c-88bc-7301abea291a";

pub const CARPLAY_HCI_CLASS_ACCESSORY: &str = "0x020040";
pub const CARPLAY_HCI_CLASS_ACCESSORY_MGMT_BYTES: [u8; 2] = [0x20, 0x04];

pub fn generate_iap2_sdp(server_uuid: String, channel: u16) -> String {
    format!(
        r#"
        <?xml version="1.0" encoding="UTF-8" ?>
        <record>
        <!-- Service Class ID List: iAP2 + SPP -->
        <attribute id="0x0001">
            <sequence>
            <uuid value="{server_uuid}"/>
            </sequence>
        </attribute>

        <attribute id="0x0004">
            <sequence>
                <sequence>
                    <uuid value="0x0100"/> <!-- L2CAP -->
                </sequence>
                <sequence>
                    <uuid value="0x0003"/> <!-- RFCOMM -->
                    <uint8 value="{channel}"/> <!-- channel -->
                </sequence>
            </sequence>
        </attribute>

        <!-- Bluetooth Profile Descriptor List: Serial Port -->
        <attribute id="0x0009">
            <sequence>
            <sequence>
                <uuid value="0x1101"/>
                <uint16 value="0x0102"/>
            </sequence>
            </sequence>
        </attribute>

        <attribute id="0x0005">
            <sequence>
                <uuid value="0x1002"/>
            </sequence>
        </attribute>

        <attribute id="0x0008">
            <uint8 value="0xFF"/>
        </attribute>

        <!-- Service Name -->
        <attribute id="0x0100">
            <text value="Wireless iAP"/>
        </attribute>
        </record>
    "#
    )
}

pub(crate) type BluezIapConnectionHandler = Arc<dyn Fn(IapConnection) + Send + Sync>;
pub(crate) type DbusVariant = dbus::arg::Variant<Box<dyn RefArg + 'static>>;

pub struct IapProfile {
    is_client_profile: bool,

    path: &'static str,
    on_connection: BluezIapConnectionHandler,
    release_callback: Arc<Mutex<dyn FnMut() + Send + 'static>>,
}

impl IapProfile {
    pub fn new(
        is_client_profile: bool,
        path: &'static str,
        on_connection: BluezIapConnectionHandler,
        release_callback: impl FnMut() + Send + 'static,
    ) -> Self {
        Self {
            is_client_profile,
            path,
            on_connection,
            release_callback: Arc::new(Mutex::new(release_callback)),
        }
    }

    fn release(&self) {
        debug!("Release called on profile {}", self.path);
        (self.release_callback.lock().unwrap())();
    }

    fn new_connection(&self, device: &str, fd: DbusOwnedFd, _opts: HashMap<String, DbusVariant>) {
        let raw_fd = fd.as_fd().as_raw_fd();
        let peer = get_rfcomm_peer_mac(raw_fd).ok();
        let local = get_rfcomm_local_mac(raw_fd).ok();
        let (Some(peer), Some(local)) = (peer, local) else {
            debug!("invalid peer/local addresses, dropping connection");
            return;
        };

        debug!(
            "New connection from device {} mac {:?} for local {:?} at path {}",
            device, peer, local, self.path
        );
        let conn = IapConnection::new(self.is_client_profile, fd, peer, local);
        (self.on_connection)(conn);
    }

    fn request_disconnection(&self, device: &str) {
        debug!("Request disconnection from {}", device);
    }
}

/// Represents accepted iAP Bluetooth connection.
#[derive(Debug)]
pub struct IapConnection {
    /// If true this was an outgoing connection, else it was an incoming one.
    ///
    /// This by itself does not imply which side should handshake as an iAP2 client and which as an iAP2 server.
    pub is_client: bool,
    /// The pseudo-TCP file descriptor that can be used for duplex communication.
    pub socket: OwnedFd,
    /// The peer MAC address in form of AA:BB:CC:DD:EE:FF.
    ///
    /// Should always be available if system provides it.
    pub peer: MacAddr6,
    /// The local MAC address in form of AA:BB:CC:DD:EE:FF.
    ///
    /// Should always be available if system provides it.
    pub local: MacAddr6,
}

impl IapConnection {
    pub fn new(is_client: bool, socket: OwnedFd, peer: MacAddr6, local: MacAddr6) -> Self {
        Self {
            is_client,
            socket,
            peer,
            local,
        }
    }
}

pub(crate) fn register_iap_profile_interface(cr: &mut Crossroads, path: &'static str, profile: IapProfile) {
    let iface = cr.register("org.bluez.Profile1", |b| {
        b.method("Release", (), (), |_, profile: &mut IapProfile, _: ()| {
            profile.release();
            Ok(())
        });

        b.method(
            "NewConnection",
            ("device", "fd", "fd_properties"),
            (),
            |_, profile: &mut IapProfile, (device, fd, opts): (dbus::Path<'static>, DbusOwnedFd, HashMap<String, DbusVariant>)| {
                profile.new_connection(&device.to_string(), fd, opts);
                Ok(())
            },
        );

        b.method(
            "RequestDisconnection",
            ("device",),
            (),
            |_, profile: &mut IapProfile, (device,): (dbus::Path<'static>,)| {
                profile.request_disconnection(&device.to_string());
                Ok(())
            },
        );
    });

    cr.insert(path, &[iface], profile);
}
