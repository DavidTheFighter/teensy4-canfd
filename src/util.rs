//! Kind of a misc for various CAN related things

use imxrt_ral as ral;
use super::CANFD;

impl CANFD {
    pub fn enable(&mut self, state: bool) {
        ral::modify_reg!(ral::can3, self.instance, MCR, MDIS: if state { 0b0 } else { 0b1 });

        while (ral::read_reg!(ral::can3, self.instance, MCR, LPMACK) == if state { 0b1 } else { 0b0 }) {}
    }

    pub fn enter_freeze(&mut self) {
        ral::modify_reg!(ral::can3, self.instance, MCR, FRZ: 0b1, HALT: 0b1);
        while (ral::read_reg!(ral::can3, self.instance, MCR, FRZACK) != 0b1) {}
    }
    
    pub fn exit_freeze(&mut self) {
        ral::modify_reg!(ral::can3, self.instance, MCR, HALT: 0b0);
        while (ral::read_reg!(ral::can3, self.instance, MCR, FRZACK) != 0b0) {}
    }

    pub fn reset(&mut self) {
        ral::modify_reg!(ral::can3, self.instance, MCR, DOZE: 0b0);
    
        // Wait for exit from low power mode
        while (ral::read_reg!(ral::can3, self.instance, MCR, LPMACK) == 0b1) {}
    
        ral::modify_reg!(ral::can3, self.instance, MCR, SOFTRST: 0b1);
        while (ral::read_reg!(ral::can3, self.instance, MCR, SOFTRST) == 0b1) {}
    
        // Make sure FREEZE mode is enabled
        while (ral::read_reg!(ral::can3, self.instance, MCR, FRZACK) == 0b0) {}
    
        ral::modify_reg!(ral::can3, self.instance, MCR, WRNEN: 0b1, WAKSRC: 0b1, MAXMB: 63, SUPV: 0b0, LPRIOEN: 0b1);
        ral::write_reg!(ral::can3, self.instance, CTRL1, 0);
        ral::write_reg!(ral::can3, self.instance, CTRL2, RRS: 0b1, EACEN: 0b0, TASD: 0x16, ISOCANFDEN: 0b1);
    
        // Reset RXIMRn registers
        for n in 0..64 {
            self.get_rximr_n(n).write(0x3FFF_FFFF);
        }
    
        ral::write_reg!(ral::can3, self.instance, RXMGMASK, 0);
        ral::write_reg!(ral::can3, self.instance, RX14MASK, 0);
        ral::write_reg!(ral::can3, self.instance, RX15MASK, 0);
        ral::write_reg!(ral::can3, self.instance, RXFGMASK, 0);
    
        ral::write_reg!(ral::can3, self.instance, IMASK1, 0);
        ral::write_reg!(ral::can3, self.instance, IMASK2, 0);
    
        ral::write_reg!(ral::can3, self.instance, IFLAG1, 0);
        ral::write_reg!(ral::can3, self.instance, IFLAG2, 0);
    
        // Clear all MB CS fields
        for n in 0..63 {
            self.get_cs_n(n).write(0);
        }
    }

    pub fn get_region_1_message_buffers(&self) -> u32 {
        self.config.region_1_config.max_buffers_per_region()
    }

    pub fn get_region_2_message_buffers(&self) -> u32 {
        self.config.region_2_config.max_buffers_per_region()
    }

    pub fn get_max_message_buffers(&self) -> u32 {
        self.get_region_1_message_buffers() + self.get_region_2_message_buffers()
    }

    pub fn get_rximr_n(&mut self, n: u32) -> &ral::RWRegister<u32> {
        match n {
            0 => &self.instance.RXIMR0,
            1 => &self.instance.RXIMR1,
            2 => &self.instance.RXIMR2,
            3 => &self.instance.RXIMR3,
            4 => &self.instance.RXIMR4,
            5 => &self.instance.RXIMR5,
            6 => &self.instance.RXIMR6,
            7 => &self.instance.RXIMR7,
            8 => &self.instance.RXIMR8,
            9 => &self.instance.RXIMR9,
            10 => &self.instance.RXIMR10,
            11 => &self.instance.RXIMR11,
            12 => &self.instance.RXIMR12,
            13 => &self.instance.RXIMR13,
            14 => &self.instance.RXIMR14,
            15 => &self.instance.RXIMR15,
            16 => &self.instance.RXIMR16,
            17 => &self.instance.RXIMR17,
            18 => &self.instance.RXIMR18,
            19 => &self.instance.RXIMR19,
            20 => &self.instance.RXIMR20,
            21 => &self.instance.RXIMR21,
            22 => &self.instance.RXIMR22,
            23 => &self.instance.RXIMR23,
            24 => &self.instance.RXIMR24,
            25 => &self.instance.RXIMR25,
            26 => &self.instance.RXIMR26,
            27 => &self.instance.RXIMR27,
            28 => &self.instance.RXIMR28,
            29 => &self.instance.RXIMR29,
            30 => &self.instance.RXIMR30,
            31 => &self.instance.RXIMR31,
            32 => &self.instance.RXIMR32,
            33 => &self.instance.RXIMR33,
            34 => &self.instance.RXIMR34,
            35 => &self.instance.RXIMR35,
            36 => &self.instance.RXIMR36,
            37 => &self.instance.RXIMR37,
            38 => &self.instance.RXIMR38,
            39 => &self.instance.RXIMR39,
            40 => &self.instance.RXIMR40,
            41 => &self.instance.RXIMR41,
            42 => &self.instance.RXIMR42,
            43 => &self.instance.RXIMR43,
            44 => &self.instance.RXIMR44,
            45 => &self.instance.RXIMR45,
            46 => &self.instance.RXIMR46,
            47 => &self.instance.RXIMR47,
            48 => &self.instance.RXIMR48,
            49 => &self.instance.RXIMR49,
            50 => &self.instance.RXIMR50,
            51 => &self.instance.RXIMR51,
            52 => &self.instance.RXIMR52,
            53 => &self.instance.RXIMR53,
            54 => &self.instance.RXIMR54,
            55 => &self.instance.RXIMR55,
            56 => &self.instance.RXIMR56,
            57 => &self.instance.RXIMR57,
            58 => &self.instance.RXIMR58,
            59 => &self.instance.RXIMR59,
            60 => &self.instance.RXIMR60,
            61 => &self.instance.RXIMR61,
            62 => &self.instance.RXIMR62,
            63 => &self.instance.RXIMR63,
            _ => &self.instance.RXIMR0,
        }
    }
    
    fn get_cs_n(&mut self, n: u32) -> &ral::RWRegister<u32> {
        match n {
            0 => &self.instance.CS0,
            1 => &self.instance.CS1,
            2 => &self.instance.CS2,
            3 => &self.instance.CS3,
            4 => &self.instance.CS4,
            5 => &self.instance.CS5,
            6 => &self.instance.CS6,
            7 => &self.instance.CS7,
            8 => &self.instance.CS8,
            9 => &self.instance.CS9,
            10 => &self.instance.CS10,
            11 => &self.instance.CS11,
            12 => &self.instance.CS12,
            13 => &self.instance.CS13,
            14 => &self.instance.CS14,
            15 => &self.instance.CS15,
            16 => &self.instance.CS16,
            17 => &self.instance.CS17,
            18 => &self.instance.CS18,
            19 => &self.instance.CS19,
            20 => &self.instance.CS20,
            21 => &self.instance.CS21,
            22 => &self.instance.CS22,
            23 => &self.instance.CS23,
            24 => &self.instance.CS24,
            25 => &self.instance.CS25,
            26 => &self.instance.CS26,
            27 => &self.instance.CS27,
            28 => &self.instance.CS28,
            29 => &self.instance.CS29,
            30 => &self.instance.CS30,
            31 => &self.instance.CS31,
            32 => &self.instance.CS32,
            33 => &self.instance.CS33,
            34 => &self.instance.CS34,
            35 => &self.instance.CS35,
            36 => &self.instance.CS36,
            37 => &self.instance.CS37,
            38 => &self.instance.CS38,
            39 => &self.instance.CS39,
            40 => &self.instance.CS40,
            41 => &self.instance.CS41,
            42 => &self.instance.CS42,
            43 => &self.instance.CS43,
            44 => &self.instance.CS44,
            45 => &self.instance.CS45,
            46 => &self.instance.CS46,
            47 => &self.instance.CS47,
            48 => &self.instance.CS48,
            49 => &self.instance.CS49,
            50 => &self.instance.CS50,
            51 => &self.instance.CS51,
            52 => &self.instance.CS52,
            53 => &self.instance.CS53,
            54 => &self.instance.CS54,
            55 => &self.instance.CS55,
            56 => &self.instance.CS56,
            57 => &self.instance.CS57,
            58 => &self.instance.CS58,
            59 => &self.instance.CS59,
            60 => &self.instance.CS60,
            61 => &self.instance.CS61,
            62 => &self.instance.CS62,
            63 => &self.instance.CS63,
            _ => &self.instance.CS0,
        }
    }
}

pub(crate) fn dlc_to_len(dlc: u32) -> u32 {
    match dlc {
        9 => 12,
        10 => 16,
        11 => 20,
        12 => 24,
        13 => 32,
        14 => 48,
        15 => 64,
        _ => dlc % 9
    }
}

pub(crate) fn len_to_dlc(len: u32) -> u32 {
    if len <= 8 {
        len
    } else if len <= 12 {
        9
    } else if len <= 16 {
        10
    } else if len <= 20 {
        11
    } else if len <= 24 {
        12
    } else if len <= 32 {
        13
    } else if len <= 48 {
        14
    } else if len <= 64 {
        15
    } else {
        8
    }
}
