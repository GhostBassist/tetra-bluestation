
#[allow(dead_code)]
#[derive(Copy, Debug, Clone, PartialEq)]
pub enum SsiType {
    Unknown,
    Ssi, // Generic for other types
    Issi,
    Gssi,
    Ussi,
    Smi,
    EventLabel, // Only usable in Umac, needs to be replaced with true SSI
}

impl core::fmt::Display for SsiType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SsiType::Unknown => write!(f, "Unknown"),
            SsiType::Ssi => write!(f, "SSI"),
            SsiType::Issi => write!(f, "ISSI"),
            SsiType::Gssi => write!(f, "GSSI"),
            SsiType::Ussi => write!(f, "USSI"),
            SsiType::Smi => write!(f, "SMI"),
            SsiType::EventLabel => write!(f, "EventLabel"),
        }
    }
}

#[derive(Copy, Debug, Clone)]
pub struct TetraAddress {
    /// Set to true if the address is an ESI (Encrypted Subscriber Identity)
    /// We maintain this field to allow us to pass still-encrypted SSIs up the stack if we want to
    pub encrypted: bool, 
    pub ssi_type: SsiType,
    pub ssi: u32,
}

impl Default for TetraAddress {
    fn default() -> Self {
        TetraAddress {
            encrypted: false,
            ssi_type: SsiType::Unknown,
            ssi: 0,
        }
    }
}

impl core::fmt::Display for TetraAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.encrypted {
            write!(f, "E_{}:{}", self.ssi_type, self.ssi)
        } else {
            write!(f, "{}:{}", self.ssi_type, self.ssi)
        }
    }
}

