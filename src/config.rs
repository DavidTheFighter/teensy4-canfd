//! All config related things

pub const MAX_BAUDRATE_CLASSICAL: u32 = 1_000_000;
pub const MAX_BAUDRATE_FD: u32 = 8_000_000;

pub enum Clock {
    Clock8Mhz,
    Clock16Mhz,
    Clock20Mhz,
    Clock24Mhz,
    Clock30Mhz,
    Clock40Mhz,
    Clock60Mhz,
    Clock80Mhz,
}

impl Clock {
    pub fn to_hz(&self) -> u32 {
        match self {
            Clock::Clock8Mhz => 8_000_000,
            Clock::Clock16Mhz => 16_000_000,
            Clock::Clock20Mhz => 20_000_000,
            Clock::Clock24Mhz => 24_000_000,
            Clock::Clock30Mhz => 30_000_000,
            Clock::Clock40Mhz => 40_000_000,
            Clock::Clock60Mhz => 60_000_000,
            Clock::Clock80Mhz => 80_000_000,
        }
    }
}

pub struct TimingConfig {
    pub baudrate: u32,
    pub jump_width: u8,
    pub phase_seg_1: u8,
    pub phase_seg_2: u8,
    pub prop_seg: u8,
}

pub struct Config {
    pub clock_speed: Clock,
    pub timing_classical: TimingConfig,
    pub timing_fd: TimingConfig,
    pub region_1_config: RegionConfig,
    pub region_2_config: RegionConfig,
    pub transceiver_compensation: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionConfig {
    MB8 { mailbox_configs: [MailboxConfig; 32] },
    MB16 { mailbox_configs: [MailboxConfig; 21] },
    MB32 { mailbox_configs: [MailboxConfig; 12] },
    MB64 { mailbox_configs: [MailboxConfig; 7] },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailboxConfig {
    Unconfigured,
    Rx { rx_config: RxMailboxConfig },
    Tx,
}

impl Default for MailboxConfig {
    fn default() -> Self {
        MailboxConfig::Unconfigured
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RxMailboxConfig {
    pub id: u32,            // The ID to match incoming messages with
    pub id_mask: u32,       // A bitmask used to compared the incoming ID, 0 is don't care, 1 is match        
    pub extended_id: bool,  // If the ID is 11 bits or 29 bits
}

impl RxMailboxConfig {
    pub fn default() -> Self {
        Self {
            id: 0,
            id_mask: 0x3FFF_FFFF,
            extended_id: false,
        }
    }
}

impl RegionConfig {
    pub(crate) fn to_mbdsr_n(&self) -> u32 {
        match self {
            RegionConfig::MB8{mailbox_configs} => 0b00,
            RegionConfig::MB16{mailbox_configs} => 0b01,
            RegionConfig::MB32{mailbox_configs} => 0b10,
            RegionConfig::MB64{mailbox_configs} => 0b11,
        }
    }

    pub(crate) fn max_buffers_per_region(&self) -> u32 {
        match self {
            RegionConfig::MB8{mailbox_configs} => 32,
            RegionConfig::MB16{mailbox_configs} => 21,
            RegionConfig::MB32{mailbox_configs} => 12,
            RegionConfig::MB64{mailbox_configs} => 7,
        }
    }

    pub(crate) fn mailbox_offset_for_idx(&self, mb_idx: u32) -> u32 {
        match self {
            RegionConfig::MB8{mailbox_configs} => mb_idx * 16,
            RegionConfig::MB16{mailbox_configs} => mb_idx * 24,
            RegionConfig::MB32{mailbox_configs} => mb_idx * 40,
            RegionConfig::MB64{mailbox_configs} => mb_idx * 72,
        }
    }

    pub(crate) fn size_bytes(&self) -> u32 {
        match self {
            RegionConfig::MB8{mailbox_configs} => 8,
            RegionConfig::MB16{mailbox_configs} => 16,
            RegionConfig::MB32{mailbox_configs} => 32,
            RegionConfig::MB64{mailbox_configs} => 64,
        }
    }
}
