//! An example that periodically sends a message to the address 123

#![no_std]
#![no_main]

extern crate panic_halt;

use teensy4_bsp as bsp;
use cortex_m_rt::entry;
use log::info;

use teensy4_canfd::{CAN3FD, CANFDBuilder, TxFDFrame, RxFDFrame};
use teensy4_canfd::config::{
    Clock, Config, Id, MailboxConfig, RegionConfig, RxMailboxConfig, TimingConfig,
};

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::interrupt;

static CAN3: Mutex<RefCell<Option<CAN3FD>>> = Mutex::new(RefCell::new(None));

fn on_rx_recv(_cs: &interrupt::CriticalSection, _rx_frame: RxFDFrame) {
    info!("main::on_rx_recv(..)");
}

#[entry]
fn main() -> ! {
    let mut _p = bsp::Peripherals::take().unwrap();
    let core_peripherals = cortex_m::Peripherals::take().unwrap();
    let mut systick = bsp::SysTick::new(core_peripherals.SYST);

    //let (mut reader, mut writer) = bsp::usb::split(&systick).unwrap();
    bsp::usb::init(
        &systick,
        bsp::usb::LoggingConfig::default(),
    )
    .unwrap();

    systick.delay(2000);

    info!("Teensy 4 CANFD Tester - Periodic transfer");

    let mbrx = RxMailboxConfig {
        id: Id::Standard(123),
        id_mask: 0x3FFF_FFFF,
    };

    let region_1_config = RegionConfig::MB32 {
        mailbox_configs: [
            MailboxConfig::Tx,
            MailboxConfig::Rx {
                rx_config: mbrx,
            },
            MailboxConfig::Unconfigured,
            MailboxConfig::Unconfigured,
            MailboxConfig::Unconfigured,
            MailboxConfig::Unconfigured,
            MailboxConfig::Unconfigured,
            MailboxConfig::Unconfigured,
            MailboxConfig::Unconfigured,
            MailboxConfig::Unconfigured,
            MailboxConfig::Unconfigured,
            MailboxConfig::Unconfigured,
            ],
    };

    let region_2_config = RegionConfig::MB64 {
        mailbox_configs: [MailboxConfig::Unconfigured; 7],
    };

    // TOOD Timings
    let can_config = Config {
        clock_speed: Clock::Clock30Mhz,
        timing_classical: TimingConfig {
            prescalar_division: 1,
            prop_seg: 13,
            phase_seg_1: 3,
            phase_seg_2: 3,
            jump_width: 3,
        },
        timing_fd: TimingConfig {
            prescalar_division: 1,
            prop_seg: 0,
            phase_seg_1: 3,
            phase_seg_2: 2,
            jump_width: 2,
        },
        region_1_config: region_1_config.clone(),
        region_2_config: region_2_config.clone(),
        transceiver_compensation: Some(3),
    };

    let canfd = CANFDBuilder::take().unwrap().build(can_config);

    if let Err(error) = canfd {
        info!("COULD NOT BUILD GOT {:?}", error);
    }

    let mut canfd = canfd.unwrap();

    interrupt::free(|cs| {
        canfd.set_rx_callback(cs, Some(on_rx_recv));
    });

    interrupt::free(|cs| CAN3.borrow(cs).replace(Some(canfd)));

    let mut counter = 42u32;
    let mut buffer = [0_u8; 16];

    loop {
        buffer[0] = (counter % 255) as u8;
        counter = counter + 1;

        let tx_frame = TxFDFrame {
            id: Id::Standard(123),
            buffer: &buffer,
            priority: None,
        };

        interrupt::free(|cs| {
            if let Some(canfd) = CAN3.borrow(cs).borrow_mut().as_mut() {
                if let Err(_error) = canfd.transfer_blocking(cs, &tx_frame) {
                    info!("WELL crap, got error");
                }
            } else {
                info!("COULD NOT BORROW");
            }
        });

        info!("SENT FRAME");

        systick.delay(50);
    }
}
