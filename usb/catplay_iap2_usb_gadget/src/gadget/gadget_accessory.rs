use catplay_iap2_usb::GadgetResult;
use macaddr::MacAddr6;
use tokio::io::DuplexStream;

use crate::gadget::GadgetStatus;

pub trait GadgetAccessory {
    /// Accepts iAP2 connection if remote side has "Enabled" the iAP2 interface and if one is pending.
    ///
    /// This process may repeat several times in response to Enable/Disable FFS events,
    /// in each case the session should be started from scratch, and the previous stream will observe EOF.
    fn accept_iap2(&mut self) -> Option<DuplexStream>;

    /// Whether this gadget exports an NCM interface for CarPlay.
    fn has_ncm(&self) -> bool;

    /// Gets current NCM interface namy if any, returns Error if not configured yet or not binding.
    fn ncm_name(&self) -> GadgetResult<Option<String>>;

    /// Gets MAC adddres of NCM interface if any, returns Error if not configured yet or not binding.
    fn mac_address(&self) -> GadgetResult<Option<MacAddr6>>;

    /// Binds and configures CarPlay NCM interface. Only possible after successful gadget bind.
    async fn bind_ncm(&mut self, ip_with_mask: &str) -> GadgetResult<String>;

    /// Takes NCM interface of CarPlay down, preventing any further traffic.
    async fn unbind_ncm(&mut self) -> GadgetResult<()>;

    fn status(&self) -> GadgetStatus;

    fn is_binding(&self) -> bool;

    /// Unbinds gadget from UDC such that it can be reused later if desired.
    ///
    /// If this is a CarPlay gadget, the NCM interface will disappear as a result of this call.
    async fn unbind(&mut self) -> GadgetResult<()>;

    async fn bind(&mut self) -> GadgetResult<()>;

    async fn set_soft_connect(&mut self, connect: bool);
}
