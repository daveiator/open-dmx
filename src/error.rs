//! Error types for the library


/// Error for when the [DMXSerial] port is disconnected.
/// 
/// [DMXSerial]: crate::DMXSerial
/// [methods]: crate::DMXSerial#implementations
///
#[derive(Debug)]
pub struct DMXDisconnectionError;

impl std::fmt::Display for DMXDisconnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DMX Port disconnected")
    }
} 

impl std::error::Error for DMXDisconnectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

    /// Error for when the channel is not inside the valid channel range of [`DMX_CHANNELS`].
    /// 
    /// - [`DMXChannelValidityError::TooLow`] if the channel is lower than `1`.
    /// 
    /// - [`DMXChannelValidityError::TooHigh`] if the channel is higher than [`DMX_CHANNELS`].
    /// 
    /// [`DMX_CHANNELS`]: crate::DMX_CHANNELS
#[derive(Debug)]
pub enum DMXChannelValidityError {
    TooHigh,
    TooLow,
}

impl std::fmt::Display for DMXChannelValidityError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DMXChannelValidityError::TooHigh => write!(f, "DMX channel too high"),
            DMXChannelValidityError::TooLow => write!(f, "DMX channel too low"),
        }
    }
}

impl std::error::Error for DMXChannelValidityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}