//! Message buffers

use core::ptr;

pub const MESSAGE_BUFFER_BASE_ADDR: u32 = 0x401D_8000 + 0x80;

pub const CS_CODE_RX_INACTIVE: u32 = 0x0;
pub const CS_CODE_RX_FULL: u32 = 0x2;
pub const CS_CODE_RX_EMPTY: u32 = 0x4;
pub const CS_CODE_RX_OVERRUN: u32 = 0x6;
pub const _CS_CODE_RX_BUSY: u32 = 0x8;
pub const _CS_CODE_RX_RANSWER: u32 = 0xA;
pub const _CS_CODE_RX_NOTUSED: u32 = 0xF;

pub const CS_CODE_TX_INACTIVE: u32 = 0x8;
pub const _CS_CODE_TX_ABORT: u32 = 0x9;
pub const CS_CODE_TX_DATA_OR_REMOTE: u32 = 0xC;
pub const _CS_CODE_TX_ANSWER: u32 = 0xE;
pub const _CS_CODE_TX_NOT_USED: u32 = 0xF;

pub fn read_cs_reg(mb_data_offset: u32) -> CSRegisterBitfield {
    unsafe {
        CSRegisterBitfield {
            val: ptr::read_volatile((MESSAGE_BUFFER_BASE_ADDR + mb_data_offset) as *mut u32),
        }
    }
}

pub fn write_cs_reg(mb_data_offset: u32, cs_reg: CSRegisterBitfield) {
    unsafe {
        ptr::write_volatile(
            (MESSAGE_BUFFER_BASE_ADDR + mb_data_offset) as *mut u32,
            cs_reg.val,
        );
    }
}

pub fn read_id_reg(mb_data_offset: u32) -> IDRegisterBitfield {
    unsafe {
        IDRegisterBitfield {
            val: ptr::read_volatile((MESSAGE_BUFFER_BASE_ADDR + mb_data_offset + 4) as *mut u32),
        }
    }
}

pub fn write_id_reg(mb_data_offset: u32, id_reg: IDRegisterBitfield) {
    unsafe {
        ptr::write_volatile(
            (MESSAGE_BUFFER_BASE_ADDR + mb_data_offset + 4) as *mut u32,
            id_reg.val,
        );
    }
}

pub fn clear_message_buffer_data(mb_data_offset: u32, mb_data_size: u32) {
    unsafe {
        let addr = (MESSAGE_BUFFER_BASE_ADDR + mb_data_offset + 8) as *mut u32;
        ptr::write_bytes(addr, 0, mb_data_size as usize);
    }
}

pub fn write_message_buffer(mb_data_offset: u32, buffer: [u32; 16], buffer_len: u32) {
    unsafe {
        let addr = MESSAGE_BUFFER_BASE_ADDR + mb_data_offset + 8;

        // TODO ptr::copy_nonoverlapping?

        for (offset, buffer_elem) in buffer
            .iter()
            .take(((buffer_len >> 2) as usize).min(16))
            .enumerate()
        {
            // (x >> 2) is fancy for (x / 4)
            ptr::write_volatile((addr + (offset as u32) * 4) as *mut u32, *buffer_elem);
        }
    }
}

pub fn read_message_buffer(mb_data_offset: u32, read_len: u32) -> [u32; 16] {
    unsafe {
        let mut buf = [0u32; 16];
        let base_addr = MESSAGE_BUFFER_BASE_ADDR + mb_data_offset + 8;

        // TODO ptr::copy_nonoverlapping?

        for i in 0..(read_len >> 2).min(16) {
            // (x >> 2) is fancy for (x / 4)
            buf[i as usize] = ptr::read_volatile((base_addr + i * 4) as *mut u32);
        }

        buf
    }
}

pub enum CSField {
    EDL,
    BRS,
    ESI,
    CODE,
    SSR,
    IDE,
    _RTR,
    DLC,
    TIMESTAMP,
}

impl CSField {
    fn mask(&self) -> u32 {
        match self {
            CSField::EDL => 0x8000_0000,
            CSField::BRS => 0x4000_0000,
            CSField::ESI => 0x2000_0000,
            CSField::CODE => 0xF00_0000,
            CSField::SSR => 0x40_0000,
            CSField::IDE => 0x20_0000,
            CSField::_RTR => 0x10_0000,
            CSField::DLC => 0xF_0000,
            CSField::TIMESTAMP => 0xFFFF,
        }
    }

    fn shift(&self) -> u32 {
        match self {
            CSField::EDL => 31,
            CSField::BRS => 30,
            CSField::ESI => 29,
            CSField::CODE => 24,
            CSField::SSR => 22,
            CSField::IDE => 21,
            CSField::_RTR => 20,
            CSField::DLC => 16,
            CSField::TIMESTAMP => 0,
        }
    }
}

pub struct CSRegisterBitfield {
    val: u32,
}

impl CSRegisterBitfield {
    pub fn new() -> Self {
        Self { val: 0 }
    }

    pub fn write_field(&mut self, field: CSField, value: u32) {
        self.val = (self.val & (!field.mask())) | ((value << field.shift()) & field.mask());
    }

    pub fn read_field(&self, field: CSField) -> u32 {
        (self.val & field.mask()) >> field.shift()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum IDField {
    PRIO,
    ID_STD,
    ID_EXT,
}

impl IDField {
    fn mask(&self) -> u32 {
        match self {
            IDField::PRIO => 0xE000_0000,
            IDField::ID_STD => 0x1FFC_0000,
            IDField::ID_EXT => 0x1FFF_FFFF,
        }
    }

    fn shift(&self) -> u32 {
        match self {
            IDField::PRIO => 29,
            IDField::ID_STD => 18,
            IDField::ID_EXT => 0,
        }
    }
}

pub struct IDRegisterBitfield {
    val: u32,
}

impl IDRegisterBitfield {
    pub fn new() -> Self {
        Self { val: 0 }
    }

    pub fn write_field(&mut self, field: IDField, value: u32) {
        self.val = (self.val & (!field.mask())) | ((value << field.shift()) & field.mask());
    }

    pub fn read_field(&self, field: IDField) -> u32 {
        (self.val & field.mask()) >> field.shift()
    }
}
