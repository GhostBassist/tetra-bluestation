// config_toml.rs
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;

use serde::Deserialize;
use toml::Value;

use crate::config::config::{CfgRfIoInfo, RfIoType};
use crate::{
    CfgCellInfo, CfgNetInfo, SharedConfig, StackConfig, StackMode, StackState,
};

/// Build `SharedConfig` from a TOML configuration file
pub fn from_toml_str(toml_str: &str) -> Result<SharedConfig, Box<dyn std::error::Error>> {
    let root: TomlRoot = toml::from_str(toml_str)?;

    // Various sanity checks
    if !root.config_version.eq("0.2") {
        tracing::warn!("Unrecognized config_version: {}", root.config_version);
    }
    if !root.extra.is_empty() {
        tracing::warn!("Unrecognized top-level fields: {:?}", sorted_keys(&root.extra));
    }
    if let Some(ref ni) = root.rfio_info {
        if !ni.extra.is_empty() {
            tracing::warn!("Unrecognized fields in rfio_info: {:?}", sorted_keys(&ni.extra));
        }
    }
    if let Some(ref ni) = root.net_info {
        if !ni.extra.is_empty() {
            tracing::warn!("Unrecognized fields in net_info: {:?}", sorted_keys(&ni.extra));
        }
    }
    if let Some(ref ci) = root.cell_info {
        if !ci.extra.is_empty() {
            tracing::warn!("Unrecognized fields in cell_info: {:?}", sorted_keys(&ci.extra));
        }
    }
    if let Some(ref ss) = root.stack_state {
        if !ss.extra.is_empty() {
            tracing::warn!("Unrecognized fields in stack_state: {:?}", sorted_keys(&ss.extra));
        }
    }

    // Require stack_mode to be explicitly set
    let Some(stack_mode) = root.stack_mode else {
        return Err("stack_mode is required in config file".into());
    };

    // Require net_info to be explicitly set
    let Some(net_info_dto) = root.net_info else {
        return Err("net_info section is required in config file".into());
    };
    let Some(mcc) = net_info_dto.mcc else {
        return Err("net_info.mcc is required in config file".into());
    };
    let Some(mnc) = net_info_dto.mnc else {
        return Err("net_info.mnc is required in config file".into());
    };

    // Build config from required and optional values
    let mut cfg = StackConfig {
        stack_mode,
        rfio: CfgRfIoInfo::default(),
        net: CfgNetInfo { mcc, mnc },
        cell: CfgCellInfo::default(),
    };

    if let Some(ni) = root.rfio_info {
        apply_rfio_info_patch(&mut cfg.rfio, ni);
    }

    if let Some(ci) = root.cell_info {
        apply_cell_info_patch(&mut cfg.cell, ci);
    }

    // Validate required fields
    cfg.validate()?;

    // Mutable runtime state
    let mut state = StackState::default();
    if let Some(ss) = root.stack_state {
        if let Some(v) = ss.cell_load_ca {
            state.cell_load_ca = v;
        }
    }

    Ok(SharedConfig::from_parts(cfg, state))
}

/// Build `SharedConfig` from any reader.
pub fn from_reader<R: Read>(reader: R) -> Result<SharedConfig, Box<dyn std::error::Error>> {
    let mut contents = String::new();
    let mut reader = BufReader::new(reader);
    reader.read_to_string(&mut contents)?;
    
    from_toml_str(&contents)
}

/// Build `SharedConfig` from a file path.
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<SharedConfig, Box<dyn std::error::Error>> {
    let f = File::open(path)?;
    let r = BufReader::new(f);
    let cfg = from_reader(r)?;
    Ok(cfg)
}

fn apply_rfio_info_patch(dst: &mut CfgRfIoInfo, ni: RfioInfoDto) {
    dst.input_type = ni.input_type;
    dst.input_file = ni.input_file;
    dst.driver = ni.driver;
    dst.rx_freq = ni.rx_freq;
    dst.tx_freq = ni.tx_freq;
    dst.ppm_err = ni.ppm_err;
    // dst.rx_gain = ni.rx_gain;
    // dst.tx_gain = ni.tx_gain;
    // dst.sample_rate = ni.sample_rate;
    // dst.antenna = ni.antenna;
    // dst.channel = ni.channel;
}

fn apply_cell_info_patch(dst: &mut CfgCellInfo, ci: CellInfoDto) {
    if let Some(v) = ci.neighbor_cell_broadcast {
        dst.neighbor_cell_broadcast = v;
    }
    if let Some(v) = ci.cell_load_ca {
        dst.cell_load_ca = v;
    }
    if let Some(v) = ci.late_entry_supported {
        dst.late_entry_supported = v;
    }
    if let Some(v) = ci.main_carrier {
        dst.main_carrier = v;
    }
    if let Some(v) = ci.freq_band {
        dst.freq_band = v;
    }
    if let Some(v) = ci.freq_offset {
        dst.freq_offset = v;
    }
    if let Some(v) = ci.duplex_spacing {
        dst.duplex_spacing_setting = v;
    }
    if let Some(v) = ci.reverse_operation {
        dst.reverse_operation = v;
    }
    if let Some(v) = ci.location_area {
        dst.location_area = v;
    }
    if let Some(v) = ci.subscriber_class {
        dst.subscriber_class = v;
    }
    if let Some(v) = ci.registration {
        dst.registration = v;
    }
    if let Some(v) = ci.deregistration {
        dst.deregistration = v;
    }
    if let Some(v) = ci.priority_cell {
        dst.priority_cell = v;
    }
    if let Some(v) = ci.no_minimum_mode {
        dst.no_minimum_mode = v;
    }
    if let Some(v) = ci.migration {
        dst.migration = v;
    }
    if let Some(v) = ci.system_wide_services {
        dst.system_wide_services = v;
    }
    if let Some(v) = ci.voice_service {
        dst.voice_service = v;
    }
    if let Some(v) = ci.circuit_mode_data_service {
        dst.circuit_mode_data_service = v;
    }
    if let Some(v) = ci.sndcp_service {
        dst.sndcp_service = v;
    }
    if let Some(v) = ci.aie_service {
        dst.aie_service = v;
    }
    if let Some(v) = ci.advanced_link {
        dst.advanced_link = v;
    }
    if let Some(v) = ci.system_code {
        dst.system_code = v;
    }
    if let Some(v) = ci.colour_code {
        dst.colour_code = v;
    }
    if let Some(v) = ci.sharing_mode {
        dst.sharing_mode = v;
    }
    if let Some(v) = ci.ts_reserved_frames {
        dst.ts_reserved_frames = v;
    }
    if let Some(v) = ci.u_plane_dtx {
        dst.u_plane_dtx = v;
    }
    if let Some(v) = ci.frame_18_ext {
        dst.frame_18_ext = v;
    }
}

fn sorted_keys(map: &HashMap<String, Value>) -> Vec<&str> {
    let mut v: Vec<&str> = map.keys().map(|s| s.as_str()).collect();
    v.sort_unstable();
    v
}

/// ----------------------- DTOs for input shape -----------------------

#[derive(Deserialize)]
struct TomlRoot {
    config_version: String,
    stack_mode: Option<StackMode>,
    rfio_info: Option<RfioInfoDto>,

    #[serde(default)]
    net_info: Option<NetInfoDto>,

    #[serde(default)]
    cell_info: Option<CellInfoDto>,

    #[serde(default)]
    stack_state: Option<StackStatePatch>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Deserialize)]
struct RfioInfoDto {
    pub input_type: RfIoType,
    pub input_file: Option<String>,
    pub driver: Option<String>,
    pub rx_freq: Option<f64>,
    pub tx_freq: Option<f64>,
    pub ppm_err: Option<f64>,
    pub rx_gain: Option<f32>,
    pub tx_gain: Option<f32>,
    pub sample_rate: Option<u32>,
    pub antenna: Option<String>,
    pub channel: Option<u32>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Default, Deserialize)]
struct NetInfoDto {
    pub mcc: Option<u16>,
    pub mnc: Option<u16>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Default, Deserialize)]
struct CellInfoDto {
    pub neighbor_cell_broadcast: Option<u8>,
    pub cell_load_ca: Option<u8>,
    pub late_entry_supported: Option<bool>,

    pub main_carrier: Option<u16>,
    pub freq_band: Option<u8>,
    pub freq_offset: Option<u8>,
    pub duplex_spacing: Option<u8>,
    pub reverse_operation: Option<bool>,

    pub location_area: Option<u16>,
    pub subscriber_class: Option<u16>,

    pub registration: Option<bool>,
    pub deregistration: Option<bool>,
    pub priority_cell: Option<bool>,
    pub no_minimum_mode: Option<bool>,
    pub migration: Option<bool>,
    pub system_wide_services: Option<bool>,
    pub voice_service: Option<bool>,
    pub circuit_mode_data_service: Option<bool>,
    pub sndcp_service: Option<bool>,
    pub aie_service: Option<bool>,
    pub advanced_link: Option<bool>,

    pub system_code: Option<u8>,
    pub colour_code: Option<u8>,
    pub sharing_mode: Option<u8>,
    pub ts_reserved_frames: Option<u8>,
    pub u_plane_dtx: Option<bool>,
    pub frame_18_ext: Option<bool>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Default, Deserialize)]
struct StackStatePatch {
    pub cell_load_ca: Option<u8>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}
