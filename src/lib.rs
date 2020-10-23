#![no_std]

pub mod config;
pub mod can_error;
pub mod mailbox;
pub(crate) mod util;
pub(crate) mod message_buffer;
mod init;

use imxrt_ral as ral;

static mut TAKEN: bool = false;

pub struct CANFD {
    instance: ral::can3::Instance,
    config: config::Config,
    tx_mailboxes: [u8; 64],
    num_tx_mailboxes: u32,
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

    pub fn build(self, can_config: config::Config, can3: ral::can3::Instance, ccm: &ral::ccm::Instance, 
        iomuxc: &ral::iomuxc::Instance) -> Result<CANFD, can_error::CANFDError> {
        
        let mut canfd = CANFD {
            instance: can3,
            config: can_config,
            tx_mailboxes: [0; 64],
            num_tx_mailboxes: 0,
        };

        canfd.init_clocks(ccm);
        canfd.init_pins(iomuxc);

        if let Err(error) = canfd.init() {
            return Err(error);
        }

        canfd.configure_region_mailboxes(1, canfd.config.region_1_config);
        canfd.configure_region_mailboxes(2, canfd.config.region_2_config);

        return Ok(canfd);
    }
}