use alloc::{vec, vec::Vec};

use super::PayloadDecodable;
use crate::packet::Packet;

#[derive(Debug, Clone, PartialEq)]
pub struct LSPSession {
    pub id: u8,
    pub session_type: SessionType,
    pub version: u8,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Control = 0x00,
    FileTransfer = 0x01,
    ExternalAccessory = 0x02,
}

impl TryFrom<u8> for SessionType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let ret = match value {
            x if x == Self::Control as _ => Self::Control,
            x if x == Self::FileTransfer as _ => Self::FileTransfer,
            x if x == Self::ExternalAccessory as _ => Self::ExternalAccessory,
            _ => return Err(()),
        };

        Ok(ret)
    }
}

impl From<SessionType> for u8 {
    fn from(val: SessionType) -> Self {
        val as u8
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LSPPayload {
    /// The maximum number of packets that may be sent without receiving an acknowledgement.
    ///
    /// Valid values are 1 to 127.
    ///
    /// This is not a negotiable parameter.
    ///
    /// The accessory and device may propose and use different values.
    ///
    /// The accessory must not send more than the device's proposed Maximum Number of Outstanding Packets
    /// without waiting for an acknowledgement from the device, and vice versa.
    pub max_outgoing: u8,
    /// The largest possible Packet Length in bytes.
    ///
    /// Valid values are 24 to 65535.
    ///
    /// This is not a negotiable parameter.
    ///
    /// The accessory and device may propose and use different values.
    pub max_len: u16,
    /// The timeout value in milliseconds for retransmission of unacknowledged packets. This should be set to a
    ///
    /// value approximating the transmission time for a packet over the link transport.
    ///
    /// Valid values are 20 ms to 65535 ms.
    ///
    /// This is a negotiable parameter.
    ///
    /// Both the accessory and device must agree on the same value.
    pub retransmission_timeout: u16,
    ///  The timeout value in milliseconds after which an acknowledgment packet must be immediately sent if another packet is not sent.
    ///
    /// Valid values are 10 ms to half of the Retransmission Timeout.
    ///
    /// This is a negotiable parameter.
    ///
    /// Both the accessory and device must agree on the same value.
    ///
    pub ack_timeout: u16,
    ///The maximum number of packet retransmissions attempted before the link is considered to be broken.
    ///
    /// Valid values are 1 to 30.
    ///
    /// This is a negotiable parameter.
    ///
    /// Both the accessory and device must agree on the same value.
    pub max_retransmissions: u8,
    /// The maximum number of received acknowledgments that may be accumulated before an acknowledgement packet must be sent if another packet is not sent.
    ///
    /// Valid values are 0 to 127 or the Maximum Number of Outstanding Packets, whichever is smaller.
    ///
    /// This is a negotiable parameter.
    ///
    /// Both the accessory and device must agree on the same value.
    pub max_ack: u8,
    pub sessions: Vec<LSPSession>,
}

impl PayloadDecodable for LSPPayload {
    fn from_packet(packet: &Packet) -> Option<Self> {
        if (packet.header.control.is_syn() || packet.header.control.is_syn_ack())
            && let Some(payload) = &packet.payload
        {
            return LSPPayload::from_bytes(payload);
        }

        None
    }

    fn to_bytes(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.to_bytes());
    }
}

impl LSPPayload {
    pub const VERSION: u8 = 0x01;
    pub const SESSION_ID_CONTROL: u8 = 0;

    pub fn from_bytes(payload: &[u8]) -> Option<Self> {
        if payload.len() < 10 {
            return None;
        }

        let version = payload[0];
        if version != Self::VERSION {
            return None;
        }

        let max_outgoing = payload[1];
        let max_len = u16::from_be_bytes(payload[2..4].try_into().unwrap());
        let retransmission_timeout = u16::from_be_bytes(payload[4..6].try_into().unwrap());
        let ack_timeout = u16::from_be_bytes(payload[6..8].try_into().unwrap());
        let max_retransmissions = payload[8];
        let max_ack = payload[9];

        let mut sessions = Vec::new();
        let mut idx = 10;
        while idx + 2 < payload.len() {
            let id = payload[idx];
            let session_type = payload[idx + 1].try_into().ok()?;
            let version = payload[idx + 2];
            sessions.push(LSPSession { id, session_type, version });
            idx += 3;
        }

        Some(Self {
            max_outgoing,
            max_len,
            retransmission_timeout,
            ack_timeout,
            max_retransmissions,
            max_ack,
            sessions,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(10 + self.sessions.len() * 3);
        buffer.push(Self::VERSION);
        buffer.push(self.max_outgoing);
        buffer.extend_from_slice(&self.max_len.to_be_bytes());
        buffer.extend_from_slice(&self.retransmission_timeout.to_be_bytes());
        buffer.extend_from_slice(&self.ack_timeout.to_be_bytes());
        buffer.push(self.max_retransmissions);
        buffer.push(self.max_ack);

        for session in &self.sessions {
            buffer.push(session.id);
            buffer.push(session.session_type.into());
            buffer.push(session.version);
        }

        buffer
    }

    /// Fix invalid values to stay within valid range.
    pub fn clamp(&mut self) {
        self.max_outgoing = self.max_outgoing.clamp(1, 127);
        self.max_len = self.max_len.clamp(24, u16::MAX);
        self.retransmission_timeout = self.retransmission_timeout.clamp(20, u16::MAX);
        self.ack_timeout = self.ack_timeout.clamp(10, self.retransmission_timeout / 2);
        self.max_retransmissions = self.max_retransmissions.clamp(1, 30);
        self.max_ack = self.max_ack.clamp(0, self.max_outgoing.min(127));
    }

    /// Check if the negotiable part is equal between two [LSPPayload]s.
    pub fn is_similar(&self, other: &LSPPayload) -> bool {
        self.sessions == other.sessions
            && self.retransmission_timeout == other.retransmission_timeout
            && self.ack_timeout == other.ack_timeout
            && self.max_retransmissions == other.max_retransmissions
            && self.max_ack == other.max_ack
    }
}

impl LSPPayload {
    pub fn modern_sessions() -> Vec<LSPSession> {
        vec![
            LSPSession {
                id: 1,
                session_type: SessionType::Control,
                version: 2, // 1
            },
            LSPSession {
                id: 2,
                session_type: SessionType::FileTransfer,
                version: 2, // 1
            },
            LSPSession {
                id: 3,
                session_type: SessionType::ExternalAccessory,
                version: 1,
            },
        ]
    }

    pub fn negotiating() -> LSPPayload {
        LSPPayload {
            max_outgoing: 1,
            max_len: 128,
            retransmission_timeout: 1000,
            ack_timeout: 10,
            max_retransmissions: 30,
            max_ack: 0,
            sessions: Self::modern_sessions(),
        }
    }

    pub fn bluetooth() -> LSPPayload {
        LSPPayload {
            max_outgoing: 5,
            max_len: 2048,
            retransmission_timeout: 1500,
            ack_timeout: 73,
            max_retransmissions: 30,
            max_ack: 3,
            sessions: Self::modern_sessions(),
        }
    }

    pub fn usb() -> LSPPayload {
        LSPPayload {
            max_outgoing: 5,
            max_len: 4096,
            retransmission_timeout: 2000,
            ack_timeout: 22,
            max_retransmissions: 30,
            max_ack: 3,
            sessions: Self::modern_sessions(),
        }
    }

    pub fn airplay() -> LSPPayload {
        LSPPayload {
            max_outgoing: 5,
            max_len: 65535,
            retransmission_timeout: 2000,
            ack_timeout: 22,
            max_retransmissions: 30,
            max_ack: 3,
            sessions: Self::modern_sessions(),
        }
    }
}

#[test]
fn test_round_trip() {
    let payload: Vec<u8> = vec![
        0x01, 0x05, // version, max_outgoing
        0x10, 0x00, // max_len (4096)
        0x04, 0x0B, // retransmission_timeout (1035)
        0x00, 0x17, // ack_timeout (23)
        0x03, // max_retransmissions
        0x03, // max_ack
        0x0A, 0x00, 0x01, // session 1: id=10, type=0, version=1
        0x0B, 0x02, 0x01, // session 2: id=11, type=2, version=1
    ];

    let lsp = LSPPayload::from_bytes(&payload).expect("Failed to parse payload");

    assert_eq!(lsp.max_outgoing, 5);
    assert_eq!(lsp.max_len, 4096);
    assert_eq!(lsp.retransmission_timeout, 1035);
    assert_eq!(lsp.ack_timeout, 23);
    assert_eq!(lsp.max_retransmissions, 3);
    assert_eq!(lsp.max_ack, 3);
    assert_eq!(lsp.sessions.len(), 2);
    assert_eq!(
        lsp.sessions[0],
        LSPSession {
            id: 10,
            session_type: SessionType::Control,
            version: 1
        }
    );
    assert_eq!(
        lsp.sessions[1],
        LSPSession {
            id: 11,
            session_type: SessionType::ExternalAccessory,
            version: 1
        }
    );

    let repacked = lsp.to_bytes();
    assert_eq!(repacked, payload);
}
