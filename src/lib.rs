#![no_std]

pub mod config;
pub mod can_error;
pub(crate) mod util;
mod init;

use imxrt_ral as ral;

static mut TAKEN: bool = false;

pub struct CANFD {
    instance: ral::can3::Instance,
    config: config::Config
}

impl CANFD {

}

pub struct CANFDBuilder {
}

impl CANFDBuilder {
    pub fn take() -> Option<Self> {
        unsafe {
            if TAKEN {
                return None;
            } else {
                TAKEN = true;

                let instance = Self {
                };

                return Some(instance);
            }
        }
    }

    pub fn build(self, can_config: config::Config, can3: ral::can3::Instance, ccm: &ral::ccm::Instance, iomuxc: &ral::iomuxc::Instance) -> CANFD {
        let mut canfd = CANFD {
            instance: can3,
            config: can_config,
        };

        canfd.init_clocks(ccm);
        canfd.init_pins(iomuxc);
        canfd.init();

        canfd
    }
}