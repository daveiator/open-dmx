//! Error types for the library


/// Error returned by various [methods] of [DMXSerial].
/// 
/// [DMXSerial]: crate::DMXSerial
/// [methods]: crate::DMXSerial#implementations
///
#[derive(Debug)]
pub enum DMXError {

    /// The serial-port is already in use.
    AlreadyInUse,

    /// If the channel is not inside the valid channel range of [`DMX_CHANNELS`].    
    /// 
    /// The exact error is stored in the enum.
    /// 
    /// - [`DMXErrorValidity::TooLow`] if the channel is lower than `1`.
    /// 
    /// - [`DMXErrorValidity::TooHigh`] if the channel is higher than [`DMX_CHANNELS`].
    /// 
    /// [`DMX_CHANNELS`]: crate::DMX_CHANNELS
    /// 
    NotValid(DMXErrorValidity),

    /// If there are no channels available.
    NoChannels,

    /// For custom errors.
    Other(String), 
}

impl std::fmt::Display for DMXError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DMXError::AlreadyInUse => write!(f, "DMX channel already in use"),
            DMXError::NotValid(exact) => match exact {
                DMXErrorValidity::TooHigh => write!(f, "DMX channel too high"),
                DMXErrorValidity::TooLow => write!(f, "DMX channel too low"),
                // _ => write!(f, "Channel is not valid ( < 1 or > 512"),
            },
            DMXError::NoChannels => write!(f, "No channels available"),
            DMXError::Other(ref s) => write!(f, "{}", s),
        }
    }
} 


impl From<String> for DMXError {
    fn from(err: String) -> DMXError {
        DMXError::Other(err)
    }
}

impl std::error::Error for DMXError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None //I'm laz
    }
}

/// The exact error for [`DMXError::NotValid`].
///
/// # Variants
/// 
/// - [`DMXErrorValidity::TooLow`] if the channel is lower than `1`.
/// 
/// - [`DMXErrorValidity::TooHigh`] if the channel is higher than [`DMX_CHANNELS`].
/// 
/// [`DMX_CHANNELS`]: crate::DMX_CHANNELS
/// 
#[derive(Debug)]
pub enum DMXErrorValidity {
    TooHigh,
    TooLow,
}