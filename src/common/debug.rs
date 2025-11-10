use std::sync::Once;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::prelude::*;


#[macro_export]
macro_rules! unimplemented_log {
    ( $($arg:tt)* ) => {{
        // will print: "unimplemented: <your message> at src/foo.rs:42"
        tracing::warn!(
            "unimplemented: {} at {}:{}",
            format_args!($($arg)*),
            file!(),
            line!(),
        );
    }};
}

/// if `cond` is false, logs a warning with your message.
#[macro_export]
macro_rules! assert_warn {
    ($cond:expr, $($arg:tt)+) => {{
        if !$cond {
            tracing::warn!(
                target: module_path!(),
                "assertion warning: `{}` failed: {} at {}:{}",
                stringify!($cond),
                format_args!($($arg)+),
                file!(),
                line!(),
            );
        }
    }};
}

static INIT_LOG: Once = Once::new();


pub fn setup_logging_verbose() {
    let filter = EnvFilter::new("trace");
    setup_logging(filter);
}

/// May be updated as desired. However, the below filters don't remove (part of) runtime overhead
/// from evaluating tracing parameter preparation. As such, we also use compiler flags to disable
/// all trace / debug level logging events in release builds. See Cargo.toml for these settings.

pub fn setup_logging_default() {

    let filter = EnvFilter::new("trace")
        // Generic
        .add_directive("tetra_bs::common::messagerouter=warn".parse().unwrap())
        .add_directive("tetra_bs::common::bitbuffer=warn".parse().unwrap())

        // Basic level for tetra entities
        // .add_directive("tetra_bs::entities=info".parse().unwrap())

        // Phy
        .add_directive("tetra_bs::entities::phy=info".parse().unwrap())
        .add_directive("tetra_bs::entities::phy::components::rxdev_soapysdr=debug".parse().unwrap())
        
        // Lmac
        .add_directive("tetra_bs::entities::lmac=info".parse().unwrap())
        .add_directive("tetra_bs::entities::lmac::components=info".parse().unwrap())

        // Umac
        .add_directive("tetra_bs::entities::umac::subcomp::slotter=debug".parse().unwrap())
        .add_directive("tetra_bs::entities::umac=debug".parse().unwrap())

        // Llc
        .add_directive("tetra_bs::entities::llc=debug".parse().unwrap())

        // Higher layers
        .add_directive("tetra_bs::entities::mle=trace".parse().unwrap())
        .add_directive("tetra_bs::entities::cmce=trace".parse().unwrap())
        .add_directive("tetra_bs::entities::sndcp=trace".parse().unwrap())
        .add_directive("tetra_bs::entities::mm=trace".parse().unwrap())
    ;

    setup_logging(filter);
}    
    
    
fn setup_logging(filter: EnvFilter) {


    // Define line formatting
    let fmt_layer = fmt::layer()
        // .with_timer(fmt::time::UtcTime::rfc_3339()) // 12:30:42.786690Z
        .without_time()
        .with_target(false)                         // remove module path
        .with_file(true)                            // include file name
        .with_line_number(true)                     // include line number
        .compact();

    // Setup once
    INIT_LOG.call_once(||{
        tracing_subscriber::registry()
            .with(fmt_layer)
        .with(filter)
        .init();
    });
}
