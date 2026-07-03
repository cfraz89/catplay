use crate::LSPPayload;

pub trait LSPNegotiator: Send {
    fn start(&self) -> LSPPayload;
    fn counter(&self, lsp: &LSPPayload, negotiation_counter: u32) -> Option<LSPPayload>;
}

pub struct SimpleLSPNegotiator {
    lsp: LSPPayload,
}

impl SimpleLSPNegotiator {
    pub fn new(lsp: LSPPayload) -> Self {
        Self { lsp }
    }
}

impl LSPNegotiator for SimpleLSPNegotiator {
    fn start(&self) -> LSPPayload {
        self.lsp.clone()
    }

    fn counter(&self, _lsp: &LSPPayload, _negotiation_counter: u32) -> Option<LSPPayload> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct NegotiationData {
    negotiation_counter: u32,
    local_lsp: LSPPayload,
    accepted_peer_lsp: Option<LSPPayload>,
    peer_accepted_lsp: Option<LSPPayload>,
}

#[derive(Debug, Clone)]
pub enum Negotiation {
    Start(NegotiationData),
    Offering { data: NegotiationData, pending: bool },
    Accepting { data: NegotiationData, pending: bool },
    Accepted(NegotiationData),
}

#[derive(Debug, Clone)]
pub enum NegotiationTx {
    SynAck(LSPPayload),
    FinalAck,
}

impl Negotiation {
    pub fn new(start_lsp: LSPPayload) -> Self {
        Self::Start(NegotiationData {
            negotiation_counter: 0,
            local_lsp: start_lsp,
            accepted_peer_lsp: None,
            peer_accepted_lsp: None,
        })
    }

    fn data(&self) -> &NegotiationData {
        match self {
            Self::Start(data) => data,
            Self::Offering { data, .. } => data,
            Self::Accepting { data, .. } => data,
            Self::Accepted(data) => data,
        }
    }

    fn data_mut(&mut self) -> &mut NegotiationData {
        match self {
            Self::Start(data) => data,
            Self::Offering { data, .. } => data,
            Self::Accepting { data, .. } => data,
            Self::Accepted(data) => data,
        }
    }

    pub fn local_lsp(&self) -> &LSPPayload {
        &self.data().local_lsp
    }

    pub fn set_local_lsp(&mut self, lsp: LSPPayload) {
        self.data_mut().local_lsp = lsp;
    }

    pub fn negotiation_counter(&self) -> u32 {
        self.data().negotiation_counter
    }

    pub fn increment_counter(&mut self) -> u32 {
        let data = self.data_mut();
        data.negotiation_counter += 1;
        data.negotiation_counter
    }

    pub fn note_lsp_accepted_by_us(&mut self, lsp: LSPPayload) {
        self.data_mut().accepted_peer_lsp = Some(lsp);
    }

    pub fn note_lsp_accepted_by_peer(&mut self, lsp: LSPPayload) {
        self.data_mut().peer_accepted_lsp = Some(lsp);
    }

    pub fn accepted_peer_lsp(&self) -> Option<&LSPPayload> {
        self.data().accepted_peer_lsp.as_ref()
    }

    pub fn peer_accepted_lsp(&self) -> Option<&LSPPayload> {
        self.data().peer_accepted_lsp.as_ref()
    }

    pub fn transition_offering(&mut self) {
        let data = self.data().clone();
        *self = Self::Offering { data, pending: true };
    }

    pub fn transition_accepting(&mut self) {
        let data = self.data().clone();
        *self = Self::Accepting { data, pending: true };
    }

    pub fn transition_start(&mut self) {
        let data = self.data().clone();
        *self = Self::Start(data);
    }

    pub fn transition_accepted(&mut self) {
        let data = self.data().clone();
        *self = Self::Accepted(data);
    }

    pub fn take_pending_tx(&mut self) -> Option<NegotiationTx> {
        match self {
            Self::Offering { data, pending } if *pending => {
                *pending = false;
                Some(NegotiationTx::SynAck(data.local_lsp.clone()))
            }
            Self::Accepting { pending, .. } if *pending => {
                *pending = false;
                Some(NegotiationTx::FinalAck)
            }
            _ => None,
        }
    }
}
