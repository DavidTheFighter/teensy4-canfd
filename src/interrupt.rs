//! Interrupt related things

use crate::CANFD;
use crate::CANFD_INSTANCE;
use cortex_m::interrupt as cortex_m_interrupt;
use cortex_m_rt::interrupt;
use imxrt_ral as ral;
use teensy4_bsp::interrupt;

#[interrupt]
unsafe fn CAN3() {
    cortex_m_interrupt::free(|cs| {
        CANFD_INSTANCE.exec(cs, |canfd| canfd.handle_interrupt(cs));
    });
}

impl CANFD {
    fn handle_interrupt(&self, cs: &cortex_m_interrupt::CriticalSection) {
        // TODO Make sure this is OPTIMIZED

        let iflag = self.read_iflag();
        let imask = self.read_imask();
        let num_mbs = self.get_max_message_buffers();

        let mut reset_mask = 0u64;

        for mb_index in 0..num_mbs {
            let mask = 1u64 << mb_index;

            // Check to make sure interrupts are enabled for this MB & it was flagged for interrupt
            if imask & mask == 0 || iflag & mask == 0 {
                continue;
            }

            if let Some(rx_frame) = self.receive(mb_index) {
                if let Some(rx_callback) = self.rx_callback {
                    rx_callback(cs, rx_frame);
                }
            }

            reset_mask |= mask;
        }

        ral::write_reg!(
            ral::can3,
            &self.instance,
            IFLAG1,
            (reset_mask & 0xFFFF_FFFF) as u32
        );
        ral::write_reg!(
            ral::can3,
            &self.instance,
            IFLAG2,
            ((reset_mask >> 32) & 0xFFFF_FFFF) as u32
        );
    }

    fn read_iflag(&self) -> u64 {
        ral::read_reg!(ral::can3, &self.instance, IFLAG1) as u64
            + ((ral::read_reg!(ral::can3, &self.instance, IFLAG2) as u64) << 32)
    }

    fn read_imask(&self) -> u64 {
        ral::read_reg!(ral::can3, &self.instance, IMASK1) as u64
            + ((ral::read_reg!(ral::can3, &self.instance, IMASK2) as u64) << 32)
    }
}
