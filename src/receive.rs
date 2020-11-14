use crate::util::dlc_to_len;
use imxrt_ral as ral;

use crate::config::Id;
use crate::message_buffer::*;
use crate::CANFD;

pub struct RxFDFrame {
    pub id: Id,
    pub buffer_len: u32,
    pub buffer: [u8; 64],
    pub timestamp: u16,
    pub error_state: bool,
}

impl CANFD {
    pub(crate) fn receive(&self, mb_index: u32) -> Option<RxFDFrame> {
        let mb_data_offset = self.get_mailbox_data_offset(mb_index);

        let mut cs_reg = read_cs_reg(mb_data_offset);
        let cs_reg_code = cs_reg.read_field(CSField::CODE);
        if cs_reg_code != CS_CODE_RX_FULL && cs_reg_code != CS_CODE_RX_OVERRUN {
            return None;
        }

        // Read the message buffer and store the data in an RxFDFrame

        let id_reg = read_id_reg(mb_data_offset);

        let extended = cs_reg.read_field(CSField::IDE) == 0b1;
        let buffer_len = dlc_to_len(cs_reg.read_field(CSField::DLC));

        let frame = RxFDFrame {
            id: if extended {
                Id::Extended(id_reg.read_field(IDField::ID_EXT))
            } else {
                Id::Standard(id_reg.read_field(IDField::ID_STD))
            },
            buffer_len,
            buffer: read_message_buffer(mb_data_offset, buffer_len),
            timestamp: cs_reg.read_field(CSField::TIMESTAMP) as u16,
            error_state: cs_reg.read_field(CSField::ESI) == 0b1,
        };

        // Reconfigure the message buffer to receive more messages
        cs_reg.write_field(CSField::CODE, CS_CODE_RX_EMPTY);

        // Quirk: Read the free-running timer to unlock the message buffer, cuz why not...
        ral::read_reg!(ral::can3, &self.instance, TIMER);

        self.write_iflag_bit(mb_index);

        if cfg!(feature = "debuginfo") {
            let region_index = mb_index / (self.get_region_1_message_buffers() - 1) + 1;
            let region_size = if region_index == 1 {
                self.config.region_1_config.size_bytes()
            } else {
                self.config.region_2_config.size_bytes()
            };

            log::info!(
                "Received {}-byte message w/ ID {} ({}) on MB #{} (Region #{} @ {}-bytes); CS: {}, ID: {}",
                frame.buffer_len,
                id_reg.read_field(IDField::ID_STD),
                extended,
                mb_index,
                region_index,
                region_size,
                cs_reg.serialize(),
                id_reg.serialize(),
            );
        }
        
        Some(frame)
    }
}
