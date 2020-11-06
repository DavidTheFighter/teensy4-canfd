//! All configuration related structures and enums
//!
//! Author: David Allen (hbddallen@gmail.com)

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

    pub(crate) fn to_clk_sel(&self) -> u32 {
        match self {
            Clock::Clock8Mhz => 2,
            Clock::Clock16Mhz => 2,
            Clock::Clock20Mhz => 2,
            Clock::Clock24Mhz => 1,
            Clock::Clock30Mhz => 0,
            Clock::Clock40Mhz => 2,
            Clock::Clock60Mhz => 0,
            Clock::Clock80Mhz => 2,
        }
    }

    pub(crate) fn to_clk_podf(&self) -> u32 {
        match self {
            Clock::Clock8Mhz => 9,
            Clock::Clock16Mhz => 4,
            Clock::Clock20Mhz => 3,
            Clock::Clock24Mhz => 0,
            Clock::Clock30Mhz => 1,
            Clock::Clock40Mhz => 1,
            Clock::Clock60Mhz => 0,
            Clock::Clock80Mhz => 0,
        }
    }
}

pub struct TimingConfig {
    pub prescalar_division: u32,
    pub prop_seg: u8,
    pub phase_seg_1: u8,
    pub phase_seg_2: u8,
    pub jump_width: u8,
}

pub struct Config {
    pub clock_speed: Clock,
    pub timing_classical: TimingConfig,
    pub timing_fd: TimingConfig,
    pub region_1_config: RegionConfig,
    pub region_2_config: RegionConfig,
    pub transceiver_compensation: Option<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum RegionConfig {
    MB8 {
        mailbox_configs: [MailboxConfig; 32],
    },
    MB16 {
        mailbox_configs: [MailboxConfig; 21],
    },
    MB32 {
        mailbox_configs: [MailboxConfig; 12],
    },
    MB64 {
        mailbox_configs: [MailboxConfig; 7],
    },
}

#[derive(Debug, Clone, Copy)]
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
pub enum Id {
    Standard(u32),
    Extended(u32),
}

#[derive(Debug, Clone, Copy)]
pub struct RxMailboxConfig {
    pub id: Id,       // The ID to match incoming messages with
    pub id_mask: u32, // A bitmask used to compared the incoming ID, 0 is don't care, 1 is match
}

impl RxMailboxConfig {
    pub fn default() -> Self {
        Self {
            id: Id::Standard(0),
            id_mask: 0x3FFF_FFFF,
        }
    }
}

impl RegionConfig {
    pub(crate) fn to_mbdsr_n(&self) -> u32 {
        match self {
            RegionConfig::MB8 { mailbox_configs: _ } => 0b00,
            RegionConfig::MB16 { mailbox_configs: _ } => 0b01,
            RegionConfig::MB32 { mailbox_configs: _ } => 0b10,
            RegionConfig::MB64 { mailbox_configs: _ } => 0b11,
        }
    }

    pub(crate) fn max_buffers_per_region(&self) -> u32 {
        match self {
            RegionConfig::MB8 { mailbox_configs: _ } => 32,
            RegionConfig::MB16 { mailbox_configs: _ } => 21,
            RegionConfig::MB32 { mailbox_configs: _ } => 12,
            RegionConfig::MB64 { mailbox_configs: _ } => 7,
        }
    }

    pub(crate) fn mailbox_offset_for_idx(&self, mb_idx: u32) -> u32 {
        match self {
            RegionConfig::MB8 { mailbox_configs: _ } => mb_idx * 16,
            RegionConfig::MB16 { mailbox_configs: _ } => mb_idx * 24,
            RegionConfig::MB32 { mailbox_configs: _ } => mb_idx * 40,
            RegionConfig::MB64 { mailbox_configs: _ } => mb_idx * 72,
        }
    }

    pub(crate) fn size_bytes(&self) -> u32 {
        match self {
            RegionConfig::MB8 { mailbox_configs: _ } => 8,
            RegionConfig::MB16 { mailbox_configs: _ } => 16,
            RegionConfig::MB32 { mailbox_configs: _ } => 32,
            RegionConfig::MB64 { mailbox_configs: _ } => 64,
        }
    }
}
