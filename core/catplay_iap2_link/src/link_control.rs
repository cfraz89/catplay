use alloc::{string::String, vec};
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct LinkControl: u8 {
        /// Link Synchronization Payload is present
        const SYN = 0b1000_0000; // Bit 7
        /// Packet Acknowledgement Number is valid, and iAP2 Session Payload may be present
        const ACK = 0b0100_0000; // Bit 6
        /// Extended Acknowledgement Payload is present
        const EAK = 0b0010_0000; // Bit 5
        /// Link reset
        const RST = 0b0001_0000; // Bit 4
        /// Device sleep
        const SLP = 0b0000_1000; // Bit 3
    }
}

pub enum LinkControlEnum {
    Syn,
    SynAck,
    Rst,
    Eak,
    Slp,

    Invalid,
}

impl LinkControl {
    pub fn as_enum(&self) -> LinkControlEnum {
        if self.is_syn_ack() {
            return LinkControlEnum::SynAck;
        }

        if self.is_syn() {
            return LinkControlEnum::Syn;
        }

        if self.is_rst() {
            return LinkControlEnum::Rst;
        }

        if self.is_eak() {
            return LinkControlEnum::Eak;
        }

        if self.is_slp() {
            return LinkControlEnum::Slp;
        }

        LinkControlEnum::Invalid
    }

    pub fn is_ack(&self) -> bool {
        self.is_exactly(LinkControl::ACK)
    }

    pub fn is_syn(&self) -> bool {
        self.is_exactly(LinkControl::SYN)
    }

    pub fn is_syn_ack(&self) -> bool {
        self.is_exactly(LinkControl::SYN | LinkControl::ACK)
    }

    pub fn is_rst(&self) -> bool {
        self.is_exactly(LinkControl::RST)
    }

    pub fn is_eak(&self) -> bool {
        self.is_exactly(LinkControl::ACK | LinkControl::EAK)
    }

    pub fn is_any_ack(&self) -> bool {
        self.is_ack() || self.is_syn_ack() || self.is_eak()
    }

    pub fn is_slp(&self) -> bool {
        self.is_exactly(LinkControl::SLP)
    }

    pub fn is_exactly(&self, expected: LinkControl) -> bool {
        self.bits() == expected.bits()
    }

    pub fn to_str(&self) -> String {
        let mut parts = vec![];

        if self.contains(Self::SYN) {
            parts.push("SYN");
        }
        if self.contains(Self::ACK) {
            parts.push("ACK");
        }
        if self.contains(Self::EAK) {
            parts.push("EAK");
        }
        if self.contains(Self::RST) {
            parts.push("RST");
        }
        if self.contains(Self::SLP) {
            parts.push("SLP");
        }

        if parts.is_empty() { "NONE".into() } else { parts.join("|") }
    }
}
