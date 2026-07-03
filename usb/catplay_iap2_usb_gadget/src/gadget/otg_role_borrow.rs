use std::borrow::Cow;

use catplay_iap2_usb::{GadgetError, GadgetResult};
use catplay_util::AsyncShutdown;
use log::debug;
use usb_gadget::Udc;

use crate::gadget::GadgetHelper;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtgRole {
    Host,
    Gadget,
    None,
}

impl From<OtgRole> for Option<bool> {
    fn from(value: OtgRole) -> Self {
        match value {
            OtgRole::Host => Some(false),
            OtgRole::Gadget => Some(true),
            OtgRole::None => None,
        }
    }
}

impl From<Option<bool>> for OtgRole {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(false) => OtgRole::Host,
            Some(true) => OtgRole::Gadget,
            None => OtgRole::None,
        }
    }
}

pub struct OtgRoleBorrow {
    unsupported: bool,
    next: OtgRole,
    current: OtgRole,
    udc: Udc,
    restored: bool,
}

fn udc_name(udc: &Udc) -> Cow<'_, str> {
    udc.name().to_string_lossy()
}

impl OtgRoleBorrow {
    /// Snapshots current OTG role(system-defined), changes to user-provided role
    /// and resets the role to snapshot when dropped.
    ///
    /// Subject to race condition(may snapshot unexpected role) if application is restarted
    /// without forcing expected role on start.
    pub fn new(udc: &Udc, role: OtgRole) -> GadgetResult<Self> {
        let udc_name = udc_name(udc);
        let current = GadgetHelper::get_usb_otg_role(&udc_name)?;

        let Some(current) = current else {
            debug!("OTG role-switch is unsupported for UDC {udc_name}");
            return Ok(Self::unsupported(udc));
        };

        Self::with_next(udc, role, current.into())
    }

    /// Changes current otg role to "role" now and to "next" when dropped.
    pub fn with_next(udc: &Udc, role: OtgRole, next: OtgRole) -> GadgetResult<Self> {
        let udc_name = udc_name(udc);

        let current = GadgetHelper::get_usb_otg_role(&udc_name)?;

        let Some(_) = current else {
            debug!("OTG role-switch is unsupported for UDC {udc_name}");
            return Ok(Self::unsupported(udc));
        };

        match GadgetHelper::change_usb_otg_role(&udc_name, role) {
            Err(err) => Err(GadgetError::FailedOtgBorrow {
                udc: udc_name.into(),
                err: err.into(),
            }),
            Ok(false) => Err(GadgetError::FailedOtg { udc: udc_name.into() }),
            Ok(true) => Ok(OtgRoleBorrow {
                unsupported: false,
                next,
                current: role,
                udc: udc.clone(),
                restored: false,
            }),
        }
    }

    pub fn unsupported(udc: &Udc) -> Self {
        OtgRoleBorrow {
            unsupported: true,
            next: OtgRole::None,
            current: OtgRole::None,
            udc: udc.clone(),
            restored: false,
        }
    }

    pub fn is_unsupported(&self) -> bool {
        self.unsupported
    }

    fn restore(&mut self) {
        let udc = udc_name(&self.udc);

        if self.next == self.current || self.restored {
            return;
        }

        debug!("Restoring OTG role {:?} (from {:?}) for UDC {}", self.next, self.current, &udc);

        match GadgetHelper::change_usb_otg_role(&udc_name(&self.udc), self.next) {
            Err(err) => debug!("Failed to restore OTG role {:?} for UDC {:?}: {err:?}", self.next, udc),
            Ok(false) => debug!("Failed to restore OTG role {:?} for UDC {:?}", self.next, udc),
            Ok(true) => {
                debug!("Restored OTG role {:?} for UDC {:?}", self.next, udc)
            }
        }

        self.restored = true;
    }
}

impl AsyncShutdown for OtgRoleBorrow {
    async fn shutdown(&mut self) {
        self.restore();
    }
}

impl Drop for OtgRoleBorrow {
    fn drop(&mut self) {
        self.restore();
    }
}
