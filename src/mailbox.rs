//! Mailboxes

use crate::util::{dlc_to_len, len_to_dlc};
use imxrt_ral as ral;

use crate::can_error::RxTxError;
use crate::config::{MailboxConfig, RegionConfig, RxMailboxConfig};
use crate::message_buffer::*;
use crate::CANFD;

pub struct TxFDFrame {
    pub id: u32,
    pub extended_id: bool,
    pub buffer_len: u32,
    pub buffer: [u32; 16],
    pub priority: Option<u8>,
}

pub struct RxFDFrame {
    pub id: u32,
    pub extended_id: bool,
    pub buffer_len: u32,
    pub buffer: [u32; 16],
    pub timestamp: u16,
    pub error_state: bool,
}
use log::info;

impl CANFD {
    pub fn transfer_blocking(&mut self, frame: TxFDFrame) -> Result<(), RxTxError> {
        // TODO Better logic for selecting mailbox (smallest size, etc)
        info!("TRANSFER");

        loop {
            for (index, mailbox) in self.mailbox_configs.iter().enumerate() {
                if *mailbox != MailboxConfig::Tx {
                    continue;
                }

                if let Ok(()) = self.transfer(index as u32, &frame) {
                    // Wait for IFLAG to set to indicate a transmission
                    while self.read_iflag_bit(index as u32) {}

                    // Reset the IFLAG bit to indicate we read the message
                    self.write_iflag_bit(index as u32);

                    return Ok(());
                }
            }
        }
    }

    fn transfer(&self, mb_index: u32, frame: &TxFDFrame) -> Result<(), RxTxError> {
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
        id_reg.write_field(
            if frame.extended_id {
                IDField::ID_EXT
            } else {
                IDField::ID_STD
            },
            frame.id,
        );

        if let Some(priority) = frame.priority {
            id_reg.write_field(IDField::PRIO, priority as u32);
        }

        write_id_reg(mb_data_offset, id_reg);

        write_message_buffer(mb_data_offset, frame.buffer, frame.buffer_len);

        // Configure CS register for transmitting
        let mut cs_reg = CSRegisterBitfield::new();
        cs_reg.write_field(CSField::CODE, CS_CODE_TX_DATA_OR_REMOTE);
        cs_reg.write_field(CSField::SSR, if frame.extended_id { 0b1 } else { 0b0 });
        cs_reg.write_field(CSField::IDE, if frame.extended_id { 0b1 } else { 0b0 });
        cs_reg.write_field(CSField::EDL, 0b1); // CAN FD Frame
        cs_reg.write_field(CSField::BRS, 0b1); // Bitrate switch
        cs_reg.write_field(CSField::DLC, len_to_dlc(frame.buffer_len));

        write_cs_reg(mb_data_offset, cs_reg);

        return Ok(());
    }

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
                id_reg.read_field(IDField::ID_EXT)
            } else {
                id_reg.read_field(IDField::ID_STD)
            },
            extended_id: extended,
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

        return Some(frame);
    }

    pub(crate) fn configure_region_mailboxes(
        &mut self,
        region_index: u32,
        region_config: RegionConfig,
    ) {
        let mb_offset = if region_index == 2 {
            self.get_region_1_message_buffers()
        } else {
            0
        };

        info!("2-0");

        self.enter_freeze();
        info!("2-1");

        match region_config {
            RegionConfig::MB8 { mailbox_configs } => {
                for (mb_index, config) in mailbox_configs.iter().enumerate() {
                    self.configure_mailbox(mb_offset + mb_index as u32, config);
                }
            }
            RegionConfig::MB16 { mailbox_configs } => {
                for (mb_index, config) in mailbox_configs.iter().enumerate() {
                    self.configure_mailbox(mb_offset + mb_index as u32, config);
                }
            }
            RegionConfig::MB32 { mailbox_configs } => {
                for (mb_index, config) in mailbox_configs.iter().enumerate() {
                    self.configure_mailbox(mb_offset + mb_index as u32, config);
                }
            }
            RegionConfig::MB64 { mailbox_configs } => {
                for (mb_index, config) in mailbox_configs.iter().enumerate() {
                    self.configure_mailbox(mb_offset + mb_index as u32, config);
                }
            }
        }
        info!("2-2");

        self.exit_freeze();
        info!("2-3");
    }

    fn configure_mailbox(&mut self, mb_index: u32, config: &MailboxConfig) {
        match config {
            MailboxConfig::Tx => self.configure_tx_mailbox(mb_index),
            MailboxConfig::Rx { rx_config } => self.configure_rx_mailbox(mb_index, rx_config),
            MailboxConfig::Unconfigured => (),
        }

        self.mailbox_configs[mb_index as usize] = config.clone();
    }

    fn configure_tx_mailbox(&mut self, mb_index: u32) {
        let mb_data_offset = self.get_mailbox_data_offset(mb_index);

        self.write_iflag_bit(mb_index);
        self.set_imask_bit(mb_index, false);

        // TODO Use transmission abort feature to "inactivate" a tx configured mailbox
        let mut cs_reg = CSRegisterBitfield::new();
        cs_reg.write_field(CSField::CODE, CS_CODE_TX_INACTIVE);
        write_cs_reg(mb_data_offset, cs_reg);

        let id_reg = IDRegisterBitfield::new();
        write_id_reg(mb_data_offset, id_reg);

        clear_message_buffer_data(mb_data_offset, self.get_mailbox_size(mb_index));
    }

    fn configure_rx_mailbox(&mut self, mb_index: u32, config: &RxMailboxConfig) {
        let mb_data_offset = self.get_mailbox_data_offset(mb_index);

        self.write_iflag_bit(mb_index);
        self.set_imask_bit(mb_index, true);

        // "Inactive" and clean the message buffer
        let mut cs_reg = CSRegisterBitfield::new();
        cs_reg.write_field(CSField::CODE, CS_CODE_RX_INACTIVE);
        write_cs_reg(mb_data_offset, cs_reg);
        clear_message_buffer_data(mb_data_offset, self.get_mailbox_size(mb_index));

        // Configure the message buffer
        let mut id_reg = IDRegisterBitfield::new();
        id_reg.write_field(
            if config.extended_id {
                IDField::ID_EXT
            } else {
                IDField::ID_STD
            },
            config.id,
        );
        write_id_reg(mb_data_offset, id_reg);

        let mut cs_reg = CSRegisterBitfield::new();
        cs_reg.write_field(CSField::CODE, CS_CODE_RX_EMPTY);
        cs_reg.write_field(CSField::IDE, if config.extended_id { 0b1 } else { 0b0 });
        write_cs_reg(mb_data_offset, cs_reg);

        self.get_rximr_n(mb_index).write(config.id_mask);
    }

    fn get_mailbox_data_offset(&self, mb_index: u32) -> u32 {
        let region_1_mbs = self.get_region_1_message_buffers();

        if mb_index < region_1_mbs {
            self.config.region_1_config.mailbox_offset_for_idx(mb_index)
        } else {
            512 + self
                .config
                .region_2_config
                .mailbox_offset_for_idx(mb_index - region_1_mbs)
        }
    }

    fn get_mailbox_size(&self, mb_index: u32) -> u32 {
        if mb_index < self.get_region_1_message_buffers() {
            self.config.region_1_config.size_bytes()
        } else {
            self.config.region_2_config.size_bytes()
        }
    }

    fn set_imask_bit(&self, index: u32, state: bool) {
        if index < 32 {
            let mask: u32 = 1 << index;
            let value: u32 = if state { 1 << index } else { 0 };
            ral::modify_reg!(ral::can3, &self.instance, IMASK1, |reg| (reg & (!mask))
                | value);
        } else if index < 64 {
            let mask: u32 = 1 << (index - 32);
            let value: u32 = if state { 1 << (index - 32) } else { 0 };
            ral::modify_reg!(ral::can3, &self.instance, IMASK2, |reg| (reg & (!mask))
                | value);
        }
    }

    fn write_iflag_bit(&self, index: u32) {
        if index < 32 {
            ral::write_reg!(ral::can3, &self.instance, IFLAG1, 1 << index);
        } else if index < 64 {
            ral::write_reg!(ral::can3, &self.instance, IFLAG2, 1 << (index - 32));
        }
    }

    fn read_iflag_bit(&self, index: u32) -> bool {
        if index < 32 {
            ral::read_reg!(ral::can3, &self.instance, IFLAG1) & (1 << index) == 1
        } else {
            ral::read_reg!(ral::can3, &self.instance, IFLAG2) & (1 << (index - 32)) == 1
        }
    }
}
