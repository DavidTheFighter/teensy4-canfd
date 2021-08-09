use crate::util::len_to_dlc;

use crate::can_error::RxTxError;
use crate::config::{Id, MailboxConfig};
use crate::message_buffer::*;
use crate::CANFD;

#[derive(Debug, Clone)]
pub struct TxFDFrame<'a> {
    pub id: Id,
    pub buffer: &'a [u8],
    pub priority: Option<u8>,
}

impl CANFD {
    pub fn transfer_blocking(&mut self, frame: &TxFDFrame) -> Result<(), RxTxError> {
        loop {
            match self.transfer_nb(frame) {
                Ok(()) => return Ok(()),
                Err(err) => match err {
                    RxTxError::MailboxUnavailable => continue,
                    _ => return Err(err),
                },
            }
        }
    }

    pub fn transfer_nb(&mut self, frame: &TxFDFrame) -> Result<(), RxTxError> {
        // TODO Better logic for selecting mailbox (smallest size, etc)

        let buffer_len: u32 = frame.buffer.len() as u32;

        if buffer_len > self.config.region_1_config.size_bytes()
            && buffer_len > self.config.region_2_config.size_bytes()
        {
            return Err(RxTxError::FrameTooBigForRegions);
        }

        let region_1_offset = self.get_region_1_message_buffers() as usize;
        let mut region_1_iter = self
            .mailbox_configs
            .iter()
            .enumerate()
            .take(region_1_offset);
        let mut region_2_iter = self
            .mailbox_configs
            .iter()
            .enumerate()
            .skip(region_1_offset);

        let iter1: Option<&mut dyn Iterator<Item = (usize, &MailboxConfig)>>;
        let mut iter2: Option<&mut dyn Iterator<Item = (usize, &MailboxConfig)>> = None;

        let region_1_diff =
            (self.config.region_1_config.size_bytes() as i32) - (buffer_len.min(64) as i32);
        let region_2_diff =
            (self.config.region_2_config.size_bytes() as i32) - (buffer_len.min(64) as i32);

        if region_1_diff >= 0 && region_2_diff < 0 {
            // Region 1 fits & region 2 doesn't
            iter1 = Some(&mut region_1_iter);
        } else if region_2_diff >= 0 && region_1_diff < 0 {
            // Region 2 fits & region 1 doesn't
            iter1 = Some(&mut region_2_iter);
        } else if region_1_diff < region_2_diff {
            // Region 1 is a better fit
            iter1 = Some(&mut region_1_iter);

            if region_2_diff >= 0 {
                iter2 = Some(&mut region_2_iter);
            }
        } else if region_2_diff < region_1_diff {
            // Region 2 is a better fit
            iter1 = Some(&mut region_2_iter);

            if region_1_diff >= 0 {
                iter2 = Some(&mut region_1_iter);
            }
        } else {
            // Both regions are the same size
            iter1 = Some(&mut region_1_iter);
            iter2 = Some(&mut region_2_iter);
        }

        let attempt_transfer = |index: usize, mailbox: &MailboxConfig| -> Result<(), RxTxError> {
            if let MailboxConfig::Tx = mailbox {
                if let Ok(()) = self.transfer(index as u32, frame, buffer_len) {
                    // Wait for IFLAG to set to indicate a transmission
                    while self.read_iflag_bit(index as u32) {}

                    // Reset the IFLAG bit to indicate we read the message
                    self.write_iflag_bit(index as u32);

                    if cfg!(feature = "debuginfo") {
                        let region_index =
                            (index as u32) / (self.get_region_1_message_buffers() - 1) + 1;
                        let region_size = if region_index == 1 {
                            self.config.region_1_config.size_bytes()
                        } else {
                            self.config.region_2_config.size_bytes()
                        };

                        log::info!(
                            "Sent {}-byte message on MB #{} (Region #{} @ {}-bytes)",
                            buffer_len,
                            index,
                            region_index,
                            region_size
                        );
                    }

                    return Ok(());
                }
            }

            Err(RxTxError::MailboxUnavailable)
        };

        if let Some(iter1) = iter1 {
            for (index, mailbox) in iter1 {
                if attempt_transfer(index, mailbox).is_ok() {
                    return Ok(());
                }
            }
        }

        if let Some(iter2) = iter2 {
            for (index, mailbox) in iter2 {
                if attempt_transfer(index, mailbox).is_ok() {
                    return Ok(());
                }
            }
        }

        Err(RxTxError::MailboxUnavailable)
    }

    fn transfer(&self, mb_index: u32, frame: &TxFDFrame, buffer_len: u32) -> Result<(), RxTxError> {
        let mb_data_offset = self.get_mailbox_data_offset(mb_index);

        // Ensure the mailbox can transfer
        let mut cs_reg = read_cs_reg(mb_data_offset);
        if cs_reg.read_field(CSField::CODE) == CS_CODE_TX_DATA_OR_REMOTE {
            return Err(RxTxError::MailboxUnavailable);
        }

        self.write_iflag_bit(mb_index);

        // "Inactive" message buffer
        cs_reg.write_field(CSField::CODE, CS_CODE_TX_INACTIVE);
        write_cs_reg(mb_data_offset, cs_reg);

        // Write the ID register
        let mut id_reg = IDRegisterBitfield::new();

        match frame.id {
            Id::Standard(id) => id_reg.write_field(IDField::ID_STD, id),
            Id::Extended(id) => id_reg.write_field(IDField::ID_EXT, id),
        }

        if let Some(priority) = frame.priority {
            id_reg.write_field(IDField::PRIO, priority as u32);
        }

        write_id_reg(mb_data_offset, id_reg);

        write_message_buffer(mb_data_offset, frame.buffer);

        // Configure CS register for transmitting
        let mut cs_reg = CSRegisterBitfield::new();
        cs_reg.write_field(CSField::CODE, CS_CODE_TX_DATA_OR_REMOTE);
        cs_reg.write_field(CSField::EDL, 0b1); // CAN FD Frame
        cs_reg.write_field(CSField::BRS, 0b1); // Bitrate switch
        cs_reg.write_field(CSField::DLC, len_to_dlc(buffer_len));

        match frame.id {
            Id::Standard(_) => {
                cs_reg.write_field(CSField::SSR, 0b0);
                cs_reg.write_field(CSField::IDE, 0b0);
            }
            Id::Extended(_) => {
                cs_reg.write_field(CSField::SSR, 0b1);
                cs_reg.write_field(CSField::IDE, 0b1);
            }
        }

        write_cs_reg(mb_data_offset, cs_reg);

        Ok(())
    }
}
