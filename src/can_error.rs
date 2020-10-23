//! Errors relating to CAN FD

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CANFDError {
    BaudrateTooHigh,
    PrescalarTooHigh,
    TransceiverDelayCompensationTooHigh,
    TransceiverDelayCompensationFail,
}

impl CANFDError {
    pub fn get_error_message(&self) -> &'static str {
        match self {
            CANFDError::BaudrateTooHigh => "Baudrate is too high, check the baudrate limits",
            CANFDError::PrescalarTooHigh => "Prescalar divison is too high, checking your timing config",
            CANFDError::TransceiverDelayCompensationTooHigh => "TDCOFF is too high, check clock speed & baudrate",
            CANFDError::TransceiverDelayCompensationFail => "TDCOFF failed, check clock speed & baudrate",
        }
    }
}

pub enum RxTxError {
    MailboxUnavailable,
}

impl RxTxError {
    
}
