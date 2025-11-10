
/// Relies on USRP or LimeSDR and will transmit, which is possibly not expected by the user.
/// As such, these tests are disabled by default
pub mod phy_bs_tests;

/// Stand-alone tests of UMAC layer and above
pub mod umac_ms_tests;

