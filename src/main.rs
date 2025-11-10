// #![allow(unused_imports)]
// #![allow(unused_variables)]
#![allow(dead_code)]
// #![allow(unused_mut)]

#[cfg(test)]
mod testing;
mod config;
mod common;
mod entities;
mod saps;
 
use std::env;

use common::debug::setup_logging_default;
use common::tdma_time::TdmaTime;
use common::messagerouter::MessageRouter;
use config::config::*;
use config::config_toml;
use crate::entities::phy::components::rxtxdev_soapysdr;
use crate::entities::cmce::cmce_bs::CmceBs;
use crate::entities::mle::mle_bs_ms::Mle;
use crate::entities::sndcp::sndcp_bs::Sndcp;
use crate::entities::lmac::lmac_bs::LmacBs;
use crate::entities::mm::mm_bs::MmBs;
use crate::entities::phy::phy_bs::PhyBs;
use crate::entities::llc::llc_bs_ms::Llc;
use crate::entities::umac::umac_bs::UmacBs;


/// Runs the full stack either forever or for a specified number of ticks.
fn run_stack(router: &mut MessageRouter, num_ticks: Option<usize>) {
    
    let mut ticks: usize = 0;

    loop {
        // Send tick_start event
        router.tick_all();
        
        // Deliver messages until queue empty
        while router.get_msgqueue_len() > 0{
            router.deliver_all_messages();
        }

        // Send tick_end event and process final messages
        router.tick_end();
        
        // Check if we should stop
        ticks += 1;
        if let Some(num_ticks) = num_ticks {
            if ticks >= num_ticks {
                break;
            }
        }
    }
}

/// Load configuration file
fn load_config_from_toml(cfg_path: &str) -> SharedConfig {
    match config_toml::from_file(cfg_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to load configuration from {}: {}", cfg_path, e);
            std::process::exit(1);
        }
    }
}

fn build_soapysdr_phy(cfg: &SharedConfig) -> PhyBs<rxtxdev_soapysdr::RxTxDevSoapySdr> {
    let c = cfg.config();

    let dl_carrier_freq_base = c.rfio.tx_freq.expect("tx_freq must be set for Soapysdr RFIO");
    let ul_carrier_freq_base = c.rfio.rx_freq.expect("rx_freq must be set for Soapysdr RFIO");
    let ppm_err = c.rfio.ppm_err.unwrap_or(0.0);

    let dl_err = (dl_carrier_freq_base / 1000000f64) * ppm_err;
    let dl_carrier_freq = dl_carrier_freq_base + dl_err;
    let ul_err = (ul_carrier_freq_base / 1000000f64) * ppm_err;
    let ul_carrier_freq = ul_carrier_freq_base + ul_err;

    tracing::info!(
        "Freqs: DL / UL: {:.6} MHz / {:.6} MHz   PPM: {:.2} -> err {:.0} / {:.0} hz, adj {:.6} MHz / {:.6} MHz",
        dl_carrier_freq_base / 1e6,
        ul_carrier_freq_base / 1e6,
        ppm_err,
        dl_err,
        ul_err,
        dl_carrier_freq / 1e6,
        ul_carrier_freq / 1e6
    );

    let driver = c.rfio.driver.as_deref().expect("driver must be set for Soapysdr RFIO");
    let sdrconfig = rxtxdev_soapysdr::SdrConfig {
        dev_args: &[("driver", driver)],
        // Offset RX center frequency from carrier frequency
        // to keep DC offset and 1/f noise outside of received bandwidth.
        rx_freq: Some(ul_carrier_freq - 20000.0),
        tx_freq: Some(dl_carrier_freq),
    };
    let phyconfig = rxtxdev_soapysdr::PhyConfig {
        bs_dl_frequencies: &[dl_carrier_freq],
        bs_ul_frequencies: &[ul_carrier_freq],
        ..Default::default()
    };
    let rxdev = rxtxdev_soapysdr::RxTxDevSoapySdr::new(
        sdrconfig,
        phyconfig,
    );

    PhyBs::new(cfg.clone(), rxdev)
}

// fn build_iofile_phy(cfg: &SharedConfig) -> PhyBs<rxdev_inputfile::RxDevInputFile> {
//     let rxdev = rxdev_inputfile::RxDevInputFile::new(cfg.config().rfio.input_file.as_ref().expect("input_file must be set for File RFIO"));
//     PhyBs::new(cfg.clone(), rxdev)
// }

/// Start base station stack
fn build_bs_stack(cfg: &mut SharedConfig) -> MessageRouter {

    let mut router = MessageRouter::new(cfg.clone());

    // Add suitable Phy component based on RFIO type
    if cfg.config().rfio.input_type == RfIoType::Soapysdr {
        let phy = build_soapysdr_phy(cfg);
        router.register_entity(Box::new(phy));
    } else if cfg.config().rfio.input_type == RfIoType::File {
        // let phy = build_iofile_phy(&cfg);
        // router.register_entity(Box::new(phy));
        unimplemented!("File RFIO type not implemented currently");
    } else {
        panic!("Unsupported RFIO type: {:?}", cfg.config().rfio.input_type);
    }
    
    // Add remaining components
    let lmac = LmacBs::new(cfg.clone());
    let umac = UmacBs::new(cfg.clone());
    let llc = Llc::new(cfg.clone());
    let mle = Mle::new(cfg.clone());
    let mm = MmBs::new(cfg.clone());
    let sndcp = Sndcp::new(cfg.clone());
    let cmce = CmceBs::new(cfg.clone());
    router.register_entity(Box::new(lmac));
    router.register_entity(Box::new(umac));
    router.register_entity(Box::new(llc));
    router.register_entity(Box::new(mle));
    router.register_entity(Box::new(mm));
    router.register_entity(Box::new(sndcp));
    router.register_entity(Box::new(cmce));
    
    // Init network time
    router.set_dl_time(TdmaTime::default());

    router
}


fn print_usage(args: Vec<String>) {
    eprintln!("Usage: {} <config.toml>", args[0]);
    eprintln!("Example: {} ./bs_example.toml", args[0]);
}

fn main() {

    setup_logging_default();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Error: Invalid number of arguments.");
        print_usage(args);
        std::process::exit(1);
    }

    let filepath = &args[1];
    let mut cfg = load_config_from_toml(filepath);
    let mut router = match cfg.config().stack_mode {

        StackMode::Mon => {
            unimplemented!("Monitor mode is not implemented");
        },
        StackMode::Ms => {
            unimplemented!("MS mode is not implemented");
        },
        StackMode::Bs => {
            build_bs_stack(&mut cfg)
        }
    };

    run_stack(&mut router, None);
}
