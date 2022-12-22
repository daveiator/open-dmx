
#[cfg(feature = "thread_priority")]
use thread_priority;

use crate::thread::*;
use crate::check_valid_channel;
use crate::error::DMXError;
use crate::DMX_CHANNELS;

use serial;
use serial::SerialPort;

use std::time;
use std::io::Write;
use std::ffi::OsStr;
use std::thread;
use std::sync::mpsc;

// Holds the serial port settings for the Break-Signals
const BREAK_SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate: serial::Baud57600,
    char_size: serial::Bits7,
    parity: serial::ParityNone,
    stop_bits: serial::Stop1,
    flow_control: serial::FlowNone,
};

// Holds the serial port settings for the Data-Signals
const DMX_SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate: serial::BaudOther(250_000),
    char_size: serial::Bits8,
    parity: serial::ParityNone,
    stop_bits: serial::Stop2,
    flow_control: serial::FlowNone,
};

// Sleep duration between sending the break and the data
const TIME_BREAK_TO_DATA: time::Duration = time::Duration::new(0, 136_000);

/// A [DMX-Interface] which writes to the [SerialPort] independently from the main thread.
/// 
/// [DMX-Interface]: DMXSerial
/// 
/// It uses the RS-485 standard *(aka. Open DMX)* to send **DMX data** over a [SerialPort]. 
/// 
/// [SerialPort]: serial::SystemPort
///
#[derive(Debug)]
pub struct DMXSerial {
    
    name: String,
    // Array of DMX-Values which are written to the Serial-Port
    channels: ArcRwLock<[u8; DMX_CHANNELS]>,
    // Connection to the Agent-Thread, if this is dropped the Agent-Thread will stop
    agent: mpsc::Sender<()>,
    agent_rec: mpsc::Receiver<()>,

    // Mode
    is_sync: ArcRwLock<bool>,

    min_time_break_to_break: ArcRwLock<time::Duration>,

}

impl DMXSerial {
    /// Opens a new [DMX-Interface] on the given [`path`]. Returns an [DMXError] if the port could not be opened.
    /// 
    /// The [`path`] should look something like this:
    /// 
    /// - **Windows**: `COM3`
    /// - **Linux**: `/dev/ttyUSB0`
    /// 
    /// [DMX-Interface]: DMXSerial
    /// [`path`]: std::ffi::OsStr
    /// 
    /// <br>
    /// 
    ///  The interface can be set to **synchronous** or **asynchronous** mode *(default)*. 
    /// 
    /// In **synchronous** mode, no `data` will be sent to the [SerialPort] unti [`DMXSerial::update()`] is called.
    /// If updates are not sent regularly in **synchronous** mode, DMX-Devices might not react to the changes.
    /// 
    /// In **asynchronous** mode, the `data` will be polled automatically to the [SerialPort].
    /// 
    /// 
    /// [`set functions`]: DMXSerial::set_channel
    /// [SerialPort]: serial::SystemPort
    /// 
    /// # Example
    /// 
    /// Basic usage:
    /// 
    /// ```
    /// use open_dmx::DMXSerial;
    /// 
    /// fn main() {
    ///    let mut dmx = DMXSerial::open("COM3").unwrap();
    ///   dmx.set_channels([255; 512]);
    ///   dmx.set_channel(1, 0).unwrap();
    /// }
    /// ```
    /// 
    pub fn open<T: AsRef<OsStr> + ?Sized>(port: &T) -> Result<DMXSerial, serial::Error> {

        let (handler, agent_rec) = mpsc::sync_channel(0);
        let (agent, handler_rec) = mpsc::channel();

        // channel default created here!
        let dmx = DMXSerial {
            name: port.as_ref().to_string_lossy().to_string(),
            channels: ArcRwLock::new([0; DMX_CHANNELS]),
            agent,
            agent_rec,
            is_sync: ArcRwLock::new(false),
            min_time_break_to_break: ArcRwLock::new(time::Duration::from_micros(22_700))};

        let mut agent = DMXSerialAgent::open(port, dmx.min_time_break_to_break.read_only())?;
        let channel_view = dmx.channels.read_only();
        let is_sync_view = dmx.is_sync.read_only();
        let _ = thread::spawn(move || {
                #[cfg(feature = "thread_priority")]
                thread_priority::set_current_thread_priority(thread_priority::ThreadPriority::Max).unwrap_or_else(|e| {
                    eprintln!("Failed to set thread priority: \"{:?}\". Continuing anyways...", e)
                });
                loop {
                    if is_sync_view.read().unwrap().clone() {
                        handler_rec.recv().unwrap();
                    }

                    let channels = channel_view.read().unwrap().clone();

                    agent.send_dmx_packet(channels).unwrap();
                    match handler.try_send(()) {
                        Err(mpsc::TrySendError::Disconnected(_)) => break,
                        _ => {}
                    }
                }
        });
        Ok(dmx)
    }

    /// Does the same as [`DMXSerial::open`] but sets the [DMXSerial] to **sync mode**.
    /// 
    /// # Example
    /// 
    /// Basic strobe effect:
    /// 
    /// ```
    /// use open_dmx::DMXSerial;
    /// fn main() {
    ///     let mut dmx = DMXSerial::open_sync("COM3").unwrap();
    ///     //strobe
    ///     loop {
    ///         dmx.set_channels([255; 512]);
    ///         dmx.update(); //returns once the data is sent
    ///         dmx.set_channels([0; 512]);
    ///         dmx.update();
    ///     }
    /// }
    pub fn open_sync(port: &str) -> Result<DMXSerial, serial::Error> {
        let mut dmx = DMXSerial::open(port)?;
        dmx.set_sync();
        Ok(dmx)
    }


    /// Gets the name of the Path on which the [DMXSerial] is opened.
    /// 
    ///  # Example
    /// 
    /// Basic usage:
    /// 
    /// ```
    /// # use open_dmx::DMXSerial;
    /// # fn main() {
    /// let mut dmx = DMXSerial::open("COM3").unwrap();
    /// assert_eq!(dmx.name(), "COM3");
    /// # }
    /// ```
    ///     
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the specified [`channel`] to the given [`value`].
    /// 
    /// [`channel`]: usize
    /// [`value`]: u8
    /// 
    /// # Example
    /// 
    /// Basic usage:
    /// 
    /// ```
    /// # use open_dmx::DMXSerial;
    /// # fn main() {
    /// # let mut dmx = DMXSerial::open("COM3").unwrap();
    /// dmx.set_channel(1, 255); //sets the first channel to 255
    /// # }
    /// ```
    /// 
    pub fn set_channel(&mut self, channel: usize, value: u8) -> Result<(), DMXError> {
        check_valid_channel(channel)?;
        let mut channels = self.channels.write().unwrap();
        channels[channel - 1] = value;
        Ok(())
    }

    /// Sets all channels to the given [`value`] via a array of size [`DMX_CHANNELS`].
    /// 
    /// [`value`]: u8
    /// 
    /// # Example
    /// 
    /// Checkerboard effect:
    /// 
    /// ```
    /// # use open_dmx::{DMXSerial, DMX_CHANNELS};
    /// # fn main() {
    ///    let mut dmx = DMXSerial::open("COM3").unwrap();
    ///    let mut channels = [0; DMX_CHANNELS];
    ///    channels.iter_mut().enumerate().for_each(|(i, value)| *value = if i % 2 == 0 { 255 } else { 0 });
    ///    dmx.set_channels(channels);
    ///  # }
    /// ```
    /// 
    pub fn set_channels(&mut self, channels: [u8; DMX_CHANNELS]) {
        *self.channels.write().unwrap() = channels;
    }

    /// Tries to get the [`value`] of the specified [`channel`].
    /// 
    /// [`channel`]: usize
    /// [`value`]: u8
    /// 
    /// Returns [`DMXError::NotValid`] if the given [`channel`] is not in the range of [`DMX_CHANNELS`].
    /// 
    /// # Example
    /// 
    /// Basic usage:
    /// 
    /// ```
    /// # use open_dmx::DMXSerial;
    /// # fn main() {
    /// # let mut dmx = DMXSerial::open("COM3").unwrap();
    /// dmx.set_channel(1, 255).unwrap();
    /// assert_eq!(dmx.get_channel(1).unwrap(), 255);
    /// # }
    /// ```
    /// 
    pub fn get_channel(&self, channel: usize) -> Result<u8, DMXError> {
        check_valid_channel(channel)?;
        let channels = self.channels.read().unwrap();
        Ok(channels[channel - 1])
    }

    /// Returns the [`value`] of all channels via a array of size [`DMX_CHANNELS`].
    /// 
    /// [`value`]: u8
    /// 
    /// # Example
    /// 
    /// Basic usage:
    /// 
    /// ```
    /// # use open_dmx::{DMXSerial, DMX_CHANNELS};
    /// # fn main() {
    /// # let mut dmx = DMXSerial::open("COM3").unwrap();
    /// dmx.set_channels([255; DMX_CHANNELS]).unwrap();
    /// assert_eq!(dmx.get_channels(), [255; DMX_CHANNELS]);
    /// # }
    /// 
    pub fn get_channels(&self) -> [u8; DMX_CHANNELS] {
        self.channels.read().unwrap().clone()
    }

    /// Resets all channels to `0`.
    ///     
    /// # Example
    /// 
    /// Basic usage:
    /// 
    /// ```
    /// # use open_dmx::{DMXSerial, DMX_CHANNELS};
    /// # fn main() {
    /// # let mut dmx = DMXSerial::open("COM3").unwrap();
    /// dmx.set_channels([255; DMX_CHANNELS]).unwrap();
    /// assert_eq!(dmx.get_channels(), [255; DMX_CHANNELS]);
    /// dmx.reset_channels();
    /// assert_eq!(dmx.get_channels(), [0; DMX_CHANNELS]);
    /// # }
    /// ```
    /// 
    pub fn reset_channels(&mut self) {
        self.channels.write().unwrap().fill(0);
    }

    fn wait_for_update(&self) {
        self.agent_rec.recv().unwrap();
    }
    
    /// Updates the DMX data.
    /// 
    /// Returns after the data has been sent.
    /// 
    /// Works both in **sync** and **async** mode.
    /// 
    /// # Example
    /// 
    /// [Basic Usage]
    /// 
    /// [Basic Usage]: #example-1
    /// 
    pub fn update(&mut self) {
        self.update_async();
        self.wait_for_update();
    }

    /// Updates the DMX data but returns immediately.
    /// 
    /// Useless in **async** mode.
    /// 
    pub fn update_async(&self) {
        self.agent.send(()).unwrap();
    }

    /// Sets the DMX mode to **sync**.
    /// 
    pub fn set_sync(&mut self) {
        *self.is_sync.write().unwrap() = true;
    }

    /// Sets the DMX mode to **async**.
    ///     
    pub fn set_async(&mut self) {
        *self.is_sync.write().unwrap() = false;
    }

    /// Returns `true` if the DMX mode is **sync**.
    ///     
    pub fn is_sync(&self) -> bool {
        self.is_sync.read().unwrap().clone()
    }

    /// Returns `true` if the DMX mode is **async**.
    /// 
    pub fn is_async(&self) -> bool {
        !self.is_sync()
    }

    /// Sets the minimum [`Duration`] between two **DMX packets**.
    /// 
    /// [`Duration`]: time::Duration
    /// 
    /// # Default
    /// 
    /// - 22.7 ms
    /// 
    /// <br>
    /// 
    /// Some devices may require a longer time between two **packets**.
    /// 
    /// See the [DMX512-Standard] for timing.
    /// 
    /// [DMX512-Standard]: https://www.erwinrol.com/page/articles/dmx512/
    pub fn set_packet_time(&mut self, time: time::Duration) {
        self.min_time_break_to_break.write().unwrap().clone_from(&time);
    }

    /// Returns the minimum [`Duration`] between two **DMX packets**.
    /// 
    /// [`Duration`]: time::Duration
    /// 
    pub fn get_packet_time(&self) -> time::Duration {
        self.min_time_break_to_break.read().unwrap().clone()
    }

}


struct DMXSerialAgent {
    port: serial::SystemPort,
    min_b2b: ReadOnly<time::Duration>,
}

impl DMXSerialAgent {

    pub fn open<T: AsRef<OsStr> + ?Sized>(port: &T, min_b2b: ReadOnly<time::Duration>) -> Result<DMXSerialAgent, serial::Error> {
        let port = serial::SystemPort::open(port)?;
        let dmx = DMXSerialAgent {
            port,
            min_b2b,
        };
        Ok(dmx)
    }
    fn send_break(&mut self) -> serial::Result<()> {
        self.port.configure(&BREAK_SETTINGS)?;
        self.port.write(&[0x00])?;
        Ok(())
    }

    fn send_data(&mut self, data: &[u8]) -> serial::Result<()> {
        self.port.configure(&DMX_SETTINGS)?;
        self.port.write(data)?;
        Ok(())
    }
    
    pub fn send_dmx_packet(&mut self, channels: [u8; DMX_CHANNELS]) -> serial::Result<()> {
        let start = time::Instant::now();
        self.send_break()?;
        thread::sleep(TIME_BREAK_TO_DATA);
        let mut prefixed_data = [0; 513];// 1 start byte + 512 channels
        prefixed_data[1..].copy_from_slice(&channels);
        self.send_data(&prefixed_data)?;

        #[cfg(not(profile = "release"))]
        print!("\rTime: {:?} ", start.elapsed());

        thread::sleep(self.min_b2b.read().unwrap().saturating_sub(start.elapsed()));

        #[cfg(not(profile = "release"))]
        print!("Time to send: {:?}", start.elapsed());

        Ok(())
    }
}