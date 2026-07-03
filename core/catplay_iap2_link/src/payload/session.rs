use alloc::{collections::VecDeque, vec::Vec};

use crate::{LinkSession, Packet};

pub struct PayloadSession {
    queue_tx: VecDeque<Vec<u8>>,
    queue_rx: VecDeque<Vec<u8>>,
}

impl PayloadSession {
    pub fn new() -> Self {
        Self {
            queue_tx: VecDeque::new(),
            queue_rx: VecDeque::new(),
        }
    }

    pub fn enqueue_tx(&mut self, packet: Vec<u8>) {
        self.queue_tx.push_back(packet);
    }

    pub fn dequeue_rx(&mut self) -> Option<Vec<u8>> {
        self.queue_rx.pop_front()
    }
}

impl LinkSession for PayloadSession {
    fn dequeue_tx(&mut self, _max_payload_len: usize) -> Option<Vec<u8>> {
        self.queue_tx.pop_front()
    }

    fn enqueue_rx(&mut self, packet: &Packet) {
        if let Some(payload) = packet.payload.clone() {
            self.queue_rx.push_back(payload);
        }
    }

    fn has_tx_pending(&self) -> bool {
        !self.queue_tx.is_empty()
    }
}
