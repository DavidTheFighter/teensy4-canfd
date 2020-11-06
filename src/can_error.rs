//! Errors relating to CAN FD
//!
//! Author: David Allen (hbddallen@gmail.com)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CANFDError {
    BaudrateTooHigh,                     // Make sure baudrate is within limits
    PrescalarTooHigh,                    // Check timing config
    TransceiverDelayCompensationTooHigh, // Check clock speed & baudrate ratio
    TransceiverDelayCompensationFail,    // Check clock speed & baudrate ratio
    Unknown,                             // Placeholder, *shouldn't* ever get it
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RxTxError {
    MailboxUnavailable, // Could not use this mailbox, it was unavailable for the operation
    Unknown,            // Placeholder, *shouldn't* ever get this
}
