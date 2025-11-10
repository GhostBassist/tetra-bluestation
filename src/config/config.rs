use std::sync::{Arc, RwLock};
use serde::Deserialize;

use crate::{common::freqs::FreqInfo, entities::lmac::components::scramble::scrambler};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum StackMode {
    Bs,
    Ms,
    Mon,
}

/// The PHY layer input type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RfIoType {
    Undefined,
    None,
    Soapysdr,
    File,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgRfIoInfo {
    /// Set to: soapysdr or file
    pub input_type: RfIoType,

    /// For type file: set to path to input file
    pub input_file: Option<String>,
    
    /// For type soapysdr: set to "uhd", "limesdr", etc.
    pub driver: Option<String>,
    /// For type soapysdr: set to rx frequency in Hz
    pub rx_freq: Option<f64>,
    /// For type soapysdr: set to tx frequency in Hz
    pub tx_freq: Option<f64>,
    /// For type soapysdr: SDR PPM tuning error (SDR specific) 
    pub ppm_err: Option<f64>,
    // /// For type soapysdr: set to RX gain in dB
    // pub rx_gain: Option<f32>,
    // /// For type soapysdr: set to TX gain in dB
    // pub tx_gain: Option<f32>,
    // /// For type soapysdr: set to SDR sample rate in Hz
    // pub sample_rate: Option<u32>,
    // /// For type soapysdr: set to antenna name, e.g. "TX/RX", "RX2", etc.
    // pub antenna: Option<String>,
    // /// For type soapysdr: set to channel number, e.g. 0, 1, etc.
    // pub channel: Option<u32>
}

impl Default for CfgRfIoInfo {
    fn default() -> Self {
        Self {
            input_type: RfIoType::Undefined,
            input_file: None,
            driver: None,
            rx_freq: None,
            tx_freq: None,
            ppm_err: None,
            // rx_gain: None,
            // tx_gain: None,
            // sample_rate: None,
            // antenna: None,
            // channel: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgNetInfo {
    /// 10 bits, from 18.4.2.1 D-MLE-SYNC
    pub mcc: u16,
    /// 14 bits, from 18.4.2.1 D-MLE-SYNC
    pub mnc: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgCellInfo {
    // 2 bits, from 18.4.2.1 D-MLE-SYNC
    #[serde(default)]
    pub neighbor_cell_broadcast: u8,
    // 2 bits, from 18.4.2.1 D-MLE-SYNC
    #[serde(default)]
    pub cell_load_ca: u8,
    // 1 bit, from 18.4.2.1 D-MLE-SYNC
    #[serde(default)]
    pub late_entry_supported: bool,

    /// 12 bits, from MAC SYSINFO
    #[serde(default = "default_main_carrier")]
    pub main_carrier: u16,
    /// 4 bits, from MAC SYSINFO
    #[serde(default = "default_freq_band")]
    pub freq_band: u8,
    /// 2 bits, from MAC SYSINFO
    #[serde(default)]
    pub freq_offset: u8,
    /// 3 bits, from MAC SYSINFO
    #[serde(default)]
    pub duplex_spacing_setting: u8,
    /// 1 bits, from MAC SYSINFO
    #[serde(default)]
    pub reverse_operation: bool,

    // 14 bits, from 18.4.2.2 D-MLE-SYSINFO
    #[serde(default)]
    pub location_area: u16,
    // 16 bits, from 18.4.2.2 D-MLE-SYSINFO
    #[serde(default)]
    pub subscriber_class: u16,

    // 1-bit service flags
    #[serde(default)]
    pub registration: bool,
    #[serde(default)]
    pub deregistration: bool,
    #[serde(default)]
    pub priority_cell: bool,
    #[serde(default)]
    pub no_minimum_mode: bool,
    #[serde(default)]
    pub migration: bool,
    #[serde(default)]
    pub system_wide_services: bool,
    #[serde(default)]
    pub voice_service: bool,
    #[serde(default)]
    pub circuit_mode_data_service: bool,
    #[serde(default)]
    pub sndcp_service: bool,
    #[serde(default)]
    pub aie_service: bool,
    #[serde(default)]
    pub advanced_link: bool,

    // From SYNC
    #[serde(default)]
    pub system_code: u8,
    #[serde(default)]
    pub colour_code: u8,
    #[serde(default)]
    pub sharing_mode: u8,
    #[serde(default)]
    pub ts_reserved_frames: u8,
    #[serde(default)]
    pub u_plane_dtx: bool,
    #[serde(default)]
    pub frame_18_ext: bool,
}

impl Default for CfgCellInfo {
    fn default() -> Self {
        Self {
            freq_band: default_freq_band(),
            main_carrier: default_main_carrier(),
            freq_offset: 0,
            duplex_spacing_setting: 0,
            reverse_operation: false,

            neighbor_cell_broadcast: 0,
            cell_load_ca: 0,
            late_entry_supported: false,
            location_area: 0,
            subscriber_class: 0,
            registration: true,
            deregistration: true,
            priority_cell: false,
            no_minimum_mode: false,
            migration: false,
            system_wide_services: false,
            voice_service: false,
            circuit_mode_data_service: false,
            sndcp_service: false,
            aie_service: false,
            advanced_link: false,

            system_code: 0,
            colour_code: 0,
            sharing_mode: 0,
            ts_reserved_frames: 0,
            u_plane_dtx: false,
            frame_18_ext: false,
        }
    }
}

#[inline]
fn default_freq_band() -> u8 {
    4
}

#[inline]
fn default_main_carrier() -> u16 {
    1521
}

#[derive(Debug, Clone, Deserialize)]
pub struct StackConfig {
    #[serde(default = "default_stack_mode")]
    pub stack_mode: StackMode,

    #[serde(default)]
    pub rfio: CfgRfIoInfo,

    /// Network info is REQUIRED - no default provided
    pub net: CfgNetInfo,

    #[serde(default)]
    pub cell: CfgCellInfo,
}

fn default_stack_mode() -> StackMode {
    StackMode::Bs
}

impl StackConfig {
    
    pub fn new(mode: StackMode, mcc: u16, mnc: u16) -> Self {
        StackConfig {
            stack_mode: mode,
            rfio: CfgRfIoInfo::default(),
            net: CfgNetInfo { mcc, mnc },
            cell: CfgCellInfo::default(),
        }
    }

    /// Validate that all required configuration fields are properly set.
    pub fn validate(&self) -> Result<(), &str> {

        // Check input device settings
        match self.rfio.input_type {

            RfIoType::Soapysdr => {
                match &self.rfio.driver {
                    Some(val) => {
                        let supported_drivers = ["uhd", "limesdr", "bladeRF"];
                        if !supported_drivers.contains(&val.as_str()) {
                            return Err("unsupported rfio driver for Soapysdr input_type");
                        }
                    },
                    None => return Err("rfio driver must be set for Soapysdr input_type"),
                }
            },
            RfIoType::Undefined => {
                return Err("rfio input_type must be defined");
            },
            RfIoType::None => {}, // For testing
            _ => {
                return Err("Currently unsupported rfio.input_type");
            } 
        };

        // Sanity check on main carrier property fields in SYSINFO
        // if self.stack_mode == StackMode::Bs && self.rfio.input_type == RfIoType::Soapysdr {
        if self.rfio.input_type == RfIoType::Soapysdr {
            // Check consistency of RF frequency settings with TETRA stack settings
            let Some(ul_freq) = self.rfio.rx_freq else {
                return Err("RFIO Rx frequency must be set for BS stack mode");
            };
            let Some(dl_freq) = self.rfio.tx_freq else {
                return Err("RFIO Tx frequency must be set for BS stack mode");
            };

            let Ok(f1) = FreqInfo::from_dlul_freqs(dl_freq as u32, ul_freq as u32) else {
                return Err("Invalid RFIO DL/UL frequencies");
            };
            let     Ok(f2) = FreqInfo::from_sysinfo_settings(
                    self.cell.freq_band, 
                    self.cell.main_carrier, 
                    self.cell.freq_offset, 
                    self.cell.duplex_spacing_setting,
                    self.cell.reverse_operation) else {
                return Err("Invalid cell info frequency settings");
            };

            if f1.band != f2.band {
                return Err("RFIO Tx frequency band does not match cell info band");
            };
            if f1.carrier != f2.carrier {
                return Err("RFIO Tx frequency carrier does not match cell info carrier");
            };
            if f1.freq_offset != f2.freq_offset {
                return Err("RFIO Tx frequency offset does not match cell info offset");
            };
            if f1.reverse_operation != f2.reverse_operation {
                return Err("RFIO Tx frequency reverse operation does not match cell info reverse operation");
            };
            if f1.duplex_spacing != f2.duplex_spacing {
                return Err("RFIO Tx frequency duplex spacing does not match cell info duplex spacing");
            }
        }

        Ok(())
    }

    /// Useful shorthand to get scrambling code for the current configuration.
    pub fn scrambling_code(&self) -> u32 {
        scrambler::tetra_scramb_get_init(self.net.mcc, self.net.mnc, self.cell.colour_code)
    }
}

/// Mutable, stack-editable state (mutex-protected).
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct StackState {
    pub cell_load_ca: u8,
}


/// Global shared configuration: immutable config + mutable state.
#[derive(Clone)]
pub struct SharedConfig {
    /// Read-only configuration (immutable after construction).
    cfg: Arc<StackConfig>,
    /// Mutable state guarded with RwLock (write by the stack, read by others).
    state: Arc<RwLock<StackState>>,
}

impl SharedConfig {
    pub fn new(mode: StackMode, mcc: u16, mnc: u16) -> Self {
        Self::from_config(StackConfig::new(mode, mcc, mnc))
    }

    pub fn from_config(cfg: StackConfig) -> Self {
        Self::from_parts(cfg, StackState::default())
    }

    pub fn from_parts(cfg: StackConfig, state: StackState) -> Self {
        
        // Check config for validity before returning the SharedConfig object
        match cfg.validate() {
            Ok(_) => {}
            Err(e) => panic!("Invalid stack configuration: {}", e),
        }

        Self {
            cfg: Arc::new(cfg),
            state: Arc::new(RwLock::new(state)),
        }
    }

    /// Access immutable config.
    pub fn config(&self) -> Arc<StackConfig> {
        Arc::clone(&self.cfg)
    }

    /// Read guard for mutable state.
    pub fn state_read(&self) -> std::sync::RwLockReadGuard<'_, StackState> {
        self.state.read().expect("StackState RwLock blocked")
    }

    /// Write guard for mutable state.
    pub fn state_write(&self) -> std::sync::RwLockWriteGuard<'_, StackState> {
        self.state.write().expect("StackState RwLock blocked")
    }
}
