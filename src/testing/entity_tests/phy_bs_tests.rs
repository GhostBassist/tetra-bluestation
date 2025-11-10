
#[cfg(test)]
mod tests {
    use crate::build_soapysdr_phy;
    use crate::config::config::{RfIoType, SharedConfig, StackMode};
    use crate::common::messagerouter::MessageRouter;
    use crate::common::debug;
    use crate::entities::TetraEntityTrait;
    use crate::entities::mle::mle_bs_ms::Mle;
    use crate::entities::lmac::lmac_bs::LmacBs;
    use crate::entities::mm::mm_bs::MmBs;
    use crate::entities::llc::llc_bs_ms::Llc;
    use crate::entities::umac::umac_bs::UmacBs;
    use crate::testing::component_test::{ComponentTest, default_test_config};

    // HAM range in many countries
    const DL_FREQ: f64 = 438.025e6;
    const UL_FREQ: f64 = DL_FREQ - 5.0e6;

    /// Builds a message router with the necessary components for a base station stack.
    fn build_bs_stack_components(
        shared_config: &SharedConfig,
        phy_component: Box<dyn TetraEntityTrait>,
    ) -> MessageRouter {

        let mut router = MessageRouter::new(shared_config.clone());
        
        let lmac = LmacBs::new(shared_config.clone());
        let umac = UmacBs::new(shared_config.clone());
        let llc = Llc::new(shared_config.clone());
        let mle = Mle::new(shared_config.clone());
        let mm: MmBs = MmBs::new(shared_config.clone());

        router.register_entity(phy_component);
        router.register_entity(Box::new(lmac));
        router.register_entity(Box::new(umac));
        router.register_entity(Box::new(llc));
        router.register_entity(Box::new(mle));
        router.register_entity(Box::new(mm));        

        router
    }

    /// Calls tick() on all components and subsequently delivers all messages
    /// Either infinitely (num_ticks is None) or for a specified number of ticks.
    fn run_stack(_config: &mut SharedConfig, router: &mut MessageRouter, num_ticks: Option<u64>) {
        
        let mut ticks: u64 = 0;
        loop {
            router.tick_all();
            router.deliver_all_messages();
            ticks += 1;
            if let Some(num_ticks) = num_ticks {
                if ticks >= num_ticks {
                    break;
                }
            }
        }
    }

    #[test]
    #[ignore] // Requires LimeSDR hardware
    fn test_limesdr_bs() {
        // Setup logging and make default stack configuration
        debug::setup_logging_default();
        let mut raw_config  = default_test_config(StackMode::Bs);

        // Update default config to suit our needs
        raw_config.rfio.driver = Some("lime".to_string());

        let mut test = ComponentTest::new(raw_config);
        let config = test.config.clone();

        // Create PHY and insert it into the message router
        let phy = build_soapysdr_phy(&config);
        test.register_entity(phy);
        test.run_ticks(None);
    }

    #[test]
    #[ignore] // Requires USRP hardware
    fn test_usrp_bs() {

        // Setup logging and make default stack configuration
        debug::setup_logging_default();
        let mut raw_config  = default_test_config(StackMode::Bs);

        // Update default config to suit our needs
        raw_config.rfio.input_type = RfIoType::Soapysdr;
        raw_config.rfio.driver = Some("uhd".to_string());
        raw_config.rfio.rx_freq = Some(UL_FREQ);
        raw_config.rfio.tx_freq = Some(DL_FREQ);

        let mut test = ComponentTest::new(raw_config);
        let config = test.config.clone();

        // Create PHY and insert it into the message router
        let phy = build_soapysdr_phy(&config);
        test.register_entity(phy);
        test.run_ticks(None);
    }
}
