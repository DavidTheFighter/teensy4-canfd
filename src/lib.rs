#![no_std]

pub mod can_error;
pub mod config;
mod init;
pub(crate) mod interrupt;
pub mod mailbox;
pub(crate) mod message_buffer;
pub(crate) mod util;

use can_error::RxTxError;
use core::cell::UnsafeCell;
use cortex_m::interrupt as cortex_m_interrupt;
use cortex_m::interrupt::CriticalSection;
use imxrt_ral as ral;
use mailbox::{RxFDFrame, TxFDFrame};
use teensy4_bsp::interrupt::CAN3;

struct CANFDCS(UnsafeCell<Option<CANFD>>);

impl CANFDCS {
    pub(crate) fn exec<F>(&self, _cs: &CriticalSection, f: F)
    where
        F: FnOnce(&CANFD),
    {
        unsafe {
            if let Some(canfd) = &(*self.0.get()) {
                f(canfd);
            }
        }
    }

    pub(crate) fn exec_mut<F>(&self, _cs: &CriticalSection, f: F)
    where
        F: FnOnce(&mut CANFD),
    {
        unsafe {
            if let Some(canfd) = &mut (*self.0.get()) {
                f(canfd);
            }
        }
    }

    fn set(&self, value: Option<CANFD>) {
        unsafe {
            *self.0.get() = value;
        }
    }

    // TODO fn is_none(..)
}

unsafe impl Sync for CANFDCS {}

static mut TAKEN: bool = false;
pub(crate) static CANFD_INSTANCE: CANFDCS = CANFDCS(UnsafeCell::new(None));

pub(crate) struct CANFD {
    instance: ral::can3::Instance,
    config: config::Config,
    mailbox_configs: [config::MailboxConfig; 64],
    rx_callback: Option<fn(&CriticalSection, RxFDFrame)>,
}

pub struct CAN3FD {
    _0: (),
}

impl CAN3FD {
    pub fn transfer_blocking(
        &mut self,
        cs: &CriticalSection,
        frame: TxFDFrame,
    ) -> Result<(), RxTxError> {
        let mut result: Result<(), RxTxError> = Err(RxTxError::Unknown);

        unsafe {
            if let Some(canfd) = &mut (*CANFD_INSTANCE.0.get()) {
                result = canfd.transfer_blocking(frame);
            }
        }

        result
    }

    pub fn set_rx_callback(&mut self, cs: &CriticalSection, f: fn(&CriticalSection, RxFDFrame)) {
        unsafe {
            if let Some(canfd) = &mut (*CANFD_INSTANCE.0.get()) {
                canfd.rx_callback = Some(f);
            }
        }
    }
}

pub struct CANFDBuilder {}

use log::info;

impl CANFDBuilder {
    pub fn take() -> Option<Self> {
        unsafe {
            if TAKEN {
                None
            } else {
                TAKEN = true;

                let instance = Self {};

                Some(instance)
            }
        }
    }

    pub fn build(self, can_config: config::Config) -> Result<CAN3FD, can_error::CANFDError> {
        let mut canfd = CANFD {
            instance: ral::can3::CAN3::take().unwrap(),
            config: can_config,
            mailbox_configs: [config::MailboxConfig::Unconfigured; 64],
            rx_callback: None,
        };

        info!("0");

        canfd.init_clocks();
        info!("1");
        canfd.init_pins();
        info!("2");

        if let Err(error) = canfd.init() {
            return Err(error);
        }

        info!("3");

        canfd.configure_region_mailboxes(1, canfd.config.region_1_config);
        info!("4");

        canfd.configure_region_mailboxes(2, canfd.config.region_2_config);
        info!("5");

        CANFD_INSTANCE.set(Some(canfd));

        // Enable interrupts for CAN3
        unsafe {
            cortex_m::peripheral::NVIC::unmask(CAN3);
        }
        info!("6");

        Ok(CAN3FD { _0: () })
    }
}
