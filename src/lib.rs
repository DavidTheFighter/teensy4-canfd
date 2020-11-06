//! A driver for the Teensy 4 CAN FD bus, written entirely in Rust
//! Author: David Allen (hbddallen@gmail.com)
//!

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
use core::sync::atomic::{AtomicBool, Ordering};
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
}

unsafe impl Sync for CANFDCS {}

static TAKEN: AtomicBool = AtomicBool::new(false);
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
        _cs: &CriticalSection,
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

    pub fn transfer_nb(
        &mut self,
        _cs: &CriticalSection,
        frame: TxFDFrame,
    ) -> Result<(), RxTxError> {
        let mut result: Result<(), RxTxError> = Err(RxTxError::Unknown);

        unsafe {
            if let Some(canfd) = &mut (*CANFD_INSTANCE.0.get()) {
                result = canfd.transfer_nb(frame);
            }
        }

        result
    }

    pub fn set_rx_callback(&mut self, _cs: &CriticalSection, f: fn(&CriticalSection, RxFDFrame)) {
        unsafe {
            if let Some(canfd) = &mut (*CANFD_INSTANCE.0.get()) {
                canfd.rx_callback = Some(f);
            }
        }
    }
}

pub struct CANFDBuilder {}

impl CANFDBuilder {
    pub fn take() -> Option<Self> {
        let mut result: Option<Self> = None;

        cortex_m_interrupt::free(|_cs| {
            if !TAKEN.load(Ordering::Relaxed) {
                TAKEN.store(true, Ordering::Relaxed);

                result = Some(Self {});
            }
        });

        result
    }

    pub fn build(self, can_config: config::Config) -> Result<CAN3FD, can_error::CANFDError> {
        let mut canfd = CANFD {
            instance: ral::can3::CAN3::take().unwrap(),
            config: can_config,
            mailbox_configs: [config::MailboxConfig::Unconfigured; 64],
            rx_callback: None,
        };

        canfd.init_clocks();
        canfd.init_pins();

        if let Err(error) = canfd.init() {
            return Err(error);
        }

        canfd.configure_regions();

        unsafe {
            cortex_m_interrupt::free(|_cs| {
                *CANFD_INSTANCE.0.get() = Some(canfd);
            });

            // Enable interrupts for CAN3
            cortex_m::peripheral::NVIC::unmask(CAN3);
        }

        Ok(CAN3FD { _0: () })
    }
}
