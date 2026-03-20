#![allow(dead_code)]

/// Custom definitions for stack control
pub mod control;
pub mod tmd;

pub mod lcmc;
pub mod lmm;
pub mod ltpd;
pub mod sapmsg;
pub mod tla;
pub mod tle;
pub mod tlmb;
pub mod tlmc;
pub mod tma;
pub mod tmv;
pub mod tp;
pub mod tpc;

pub mod tnmm;

pub use sapmsg::*;

/// Layer 2 service selector used by MLE-UNITDATA requests.
/// ETSI EN 300 392-2 defines three service classes for basic link handling.
pub const LAYER2SERVICE_ACKNOWLEDGED_REQUEST: i32 = 0;
pub const LAYER2SERVICE_ACKNOWLEDGED_RESPONSE: i32 = 1;
pub const LAYER2SERVICE_UNACKNOWLEDGED: i32 = 2;
