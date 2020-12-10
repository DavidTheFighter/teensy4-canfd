//! Code related to mailboxes and sending and receiving data
//!
//! Author: David Allen (hbddallen@gmail.com)

use imxrt_ral as ral;

use crate::config::{Id, MailboxConfig, RegionConfig, RxMailboxConfig};
use crate::message_buffer::*;
use crate::CANFD;

impl CANFD {
    pub(crate) fn configure_regions(&mut self) {
        self.exec_freeze_mut(|canfd| {
            let region_2_mb_offset = canfd.get_region_1_message_buffers();

            canfd.configure_region(canfd.config.region_1_config, 0);
            canfd.configure_region(canfd.config.region_2_config, region_2_mb_offset);
        });
    }

    fn configure_region(&mut self, region_config: RegionConfig, mb_offset: u32) {
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
    }

    fn configure_mailbox(&mut self, mb_index: u32, config: &MailboxConfig) {
        match config {
            MailboxConfig::Tx => self.configure_tx_mailbox(mb_index),
            MailboxConfig::Rx { rx_config } => self.configure_rx_mailbox(mb_index, rx_config),
            MailboxConfig::Unconfigured => (),
        }

        self.mailbox_configs[mb_index as usize] = *config;
    }

    fn configure_tx_mailbox(&mut self, mb_index: u32) {
        let mb_data_offset = self.get_mailbox_data_offset(mb_index);

        if cfg!(feature = "debuginfo") {
            log::info!(
                "TXConf | Index: {}, Offset: {}, Size: {}, Max bound: {}",
                mb_index,
                mb_data_offset,
                self.get_mailbox_size(mb_index),
                mb_data_offset + self.get_mailbox_size(mb_index)
            );
        }

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

        if cfg!(feature = "debuginfo") {
            log::info!(
                "RXConf | Index: {}, Offset: {}, Size: {}, Max bound: {}",
                mb_index,
                mb_data_offset,
                self.get_mailbox_size(mb_index),
                mb_data_offset + self.get_mailbox_size(mb_index)
            );
        }

        self.write_iflag_bit(mb_index);
        self.set_imask_bit(mb_index, true);

        // "Inactive" and clean the message buffer
        let mut cs_reg = CSRegisterBitfield::new();
        cs_reg.write_field(CSField::CODE, CS_CODE_RX_INACTIVE);

        write_cs_reg(mb_data_offset, cs_reg);

        clear_message_buffer_data(mb_data_offset, self.get_mailbox_size(mb_index));

        // Configure the message buffer
        let mut id_reg = IDRegisterBitfield::new();

        match config.id {
            Id::Standard(id) => id_reg.write_field(IDField::ID_STD, id),
            Id::Extended(id) => id_reg.write_field(IDField::ID_EXT, id),
        }

        write_id_reg(mb_data_offset, id_reg);

        let mut cs_reg = CSRegisterBitfield::new();
        cs_reg.write_field(CSField::CODE, CS_CODE_RX_EMPTY);

        match config.id {
            Id::Standard(_) => cs_reg.write_field(CSField::IDE, 0b0),
            Id::Extended(_) => cs_reg.write_field(CSField::IDE, 0b1),
        }

        write_cs_reg(mb_data_offset, cs_reg);

        self.exec_freeze_mut(|canfd| match config.id {
            Id::Standard(_) => canfd
                .get_rximr_n(mb_index)
                .write((config.id_mask & 0x7FF) << 18),
            Id::Extended(_) => canfd
                .get_rximr_n(mb_index)
                .write(config.id_mask & 0x1FFF_FFFF),
        });
    }

    pub fn get_mailbox_data_offset(&self, mb_index: u32) -> u32 {
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

    pub fn get_mailbox_size(&self, mb_index: u32) -> u32 {
        if mb_index < self.get_region_1_message_buffers() {
            self.config.region_1_config.size_bytes()
        } else {
            self.config.region_2_config.size_bytes()
        }
    }

    pub fn set_imask_bit(&self, index: u32, state: bool) {
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

    pub fn write_iflag_bit(&self, index: u32) {
        if index < 32 {
            ral::write_reg!(ral::can3, &self.instance, IFLAG1, 1 << index);
        } else if index < 64 {
            ral::write_reg!(ral::can3, &self.instance, IFLAG2, 1 << (index - 32));
        }
    }

    pub fn read_iflag_bit(&self, index: u32) -> bool {
        if index < 32 {
            ral::read_reg!(ral::can3, &self.instance, IFLAG1) & (1 << index) == 1
        } else {
            ral::read_reg!(ral::can3, &self.instance, IFLAG2) & (1 << (index - 32)) == 1
        }
    }
}
