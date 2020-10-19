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
    pub region_1_mb_size: MessageBufferSize,
    pub region_2_mb_size: MessageBufferSize,
    pub transceiver_compensation: bool,
}

pub enum MessageBufferSize {
    MB8,
    MB16,
    MB32,
    MB64
}

impl MessageBufferSize {
    pub(crate) fn to_mbdsr_n(&self) -> u32 {
        match self {
            MessageBufferSize::MB8 => 0b00,
            MessageBufferSize::MB16 => 0b01,
            MessageBufferSize::MB32 => 0b10,
            MessageBufferSize::MB64 => 0b11,
        }
    }

    pub(crate) fn max_buffers_per_region(&self) -> u32 {
        match self {
            MessageBufferSize::MB8 => 32,
            MessageBufferSize::MB16 => 21,
            MessageBufferSize::MB32 => 12,
            MessageBufferSize::MB64 => 7,
        }
    }
}
