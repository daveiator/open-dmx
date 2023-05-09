//! [![Latest Release](https://img.shields.io/crates/v/open_dmx?style=for-the-badge)](https://crates.io/crates/open_dmx) [![Documentation](https://img.shields.io/docsrs/open_dmx?style=for-the-badge)](https://docs.rs/open_dmx) [![License](https://img.shields.io/crates/l/open_dmx?style=for-the-badge)]()
//!
//! A wrapper around the [**serial**] library to send **DMX data** over a [SerialPort].
//!
//! <br>
//! 
//! ## Usage
//! 
//! ```rust	
//! use open_dmx::DMXSerial;
//! 
//! fn main() {
//!    let mut dmx = DMXSerial::open("COM3").unwrap();
//!   dmx.set_channels([255; 512]);
//!   dmx.set_channel(1, 0).unwrap();
//! }
//! ```
//! 
//! <br>
//!
//! ## Feature flags
//! 
//! - `thread_priority` *(enabled by default)*- Tries to set the [thread] priority of the [SerialPort] to *`MAX`*
//! 
//! [**serial**]: https://dcuddeback.github.io/serial-rs/serial/
//! [SerialPort]: https://dcuddeback.github.io/serial-rs/serial_core/trait.SerialPort
//! [thread]: std::thread
//! 
pub mod error;

mod dmx_serial;
pub use dmx_serial::*;

mod thread;






/// The fixed amount **DMX channels** for a singe [Interface]
/// 
/// [Interface]: DMXSerial
/// 
pub const DMX_CHANNELS: usize = 512;

/// Checks if a given [usize] is a valid **DMX channel**.
/// 
/// The size of a **DMX universe** is `512` channels. *(1-512)* Everything else will be considerd invalid.
/// 
/// # Example
/// 
/// ```
/// use open_dmx::check_valid_channel;
/// 
/// // Valid channels
/// assert!(check_valid_channel(1).is_ok());
/// assert!(check_valid_channel(512).is_ok());
/// 
/// // Invalid channels
/// assert!(check_valid_channel(0).is_err());
/// assert!(check_valid_channel(513).is_err());
/// 
/// ```
///
/// # Errors
///
/// Returns an [`DMXChannelValidityError`] if the channel is not valid.
/// 
/// - [`DMXChannelValidityError::TooLow`] if the channel is lower than `1`.
/// 
/// - [`DMXChannelValidityError::TooHigh`] if the channel is higher than `512`.
///
/// [DMXChannelValidityError]: error::DMXChannelValidityError
/// [`DMXChannelValidityError`]: error::DMXChannelValidityError
/// [`DMXChannelValidityError::TooLow`]: error::DMXChannelValidityError::TooLow
/// [`DMXChannelValidityError::TooHigh`]: error::DMXChannelValidityError::TooHigh
pub fn check_valid_channel(channel: usize) -> Result<(), error::DMXChannelValidityError> {
    if channel > crate::DMX_CHANNELS {
        return Err(error::DMXChannelValidityError::TooHigh);
    }
    if channel < 1 {
        return Err(error::DMXChannelValidityError::TooLow);
    }
    Ok(())
}