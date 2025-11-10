
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MmClientState {
    Unknown,
    Attached,
    Detached,
}

pub struct MmClientProperties {
    pub ssi: u32,
    pub state: MmClientState,
    // pub last_seen: TdmaTime,
}

impl MmClientProperties {
    pub fn new(ssi: u32) -> Self {
        MmClientProperties {
            ssi,
            state: MmClientState::Unknown,
            // last_seen: TdmaTime::default(),
        }
    }
}

pub struct MmClientMgr {
    clients: std::collections::HashMap<u32, MmClientProperties>,
}

impl MmClientMgr {
    pub fn new() -> Self {
        MmClientMgr {
            clients: std::collections::HashMap::new(),
        }
    }

    pub fn fetch_or_create(&mut self, ssi: u32) -> &mut MmClientProperties {
        self.clients.entry(ssi).or_insert_with(|| MmClientProperties::new(ssi))
    }

    pub fn is_known(&self, ssi: u32) -> bool {
        self.clients.contains_key(&ssi)
    }

    /// Adds a client to the client state manager
    /// Optionally also flags state as 'attached'
    pub fn register(&mut self, ssi: u32, attached: bool) {
        let elem = MmClientProperties {
            ssi,
            state: if attached { MmClientState::Attached } else { MmClientState::Unknown },
            // last_seen: TdmaTime::default(),
        };
        self.clients.insert(ssi, elem);
    }

    pub fn remove(&mut self, ssi: u32) -> Option<MmClientProperties> {
        self.clients.remove(&ssi)
    }
}