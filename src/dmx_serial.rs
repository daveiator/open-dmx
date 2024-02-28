
#[cfg(feature = "thread_priority")]
use thread_priority;

use crate::thread::*;
use crate::check_valid_channel;
use crate::error::{DMXDisconnectionError, DMXChannelValidityError};
use crate::DMX_CHANNELS;

use serialport::SerialPort;

use std::time;
use std::io::Write;
use std::thread;
use std::sync::mpsc;

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
    agent: AgentCommunication::<()>,

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
    pub fn open(port: &str) -> Result<DMXSerial, serialport::Error> {

        let (handler, agent_rx) = mpsc::sync_channel(0);
        let (agent_tx, handler_rec) = mpsc::channel();

        // channel default created here!
        let dmx = DMXSerial {
            name: port.to_string(),
            channels: ArcRwLock::new([0; DMX_CHANNELS]),
            agent: AgentCommunication::new(agent_tx, agent_rx),
            is_sync: ArcRwLock::new(false),
            min_time_break_to_break: ArcRwLock::new(time::Duration::from_micros(22_700))};

        let mut agent = DMXSerialAgent::open(&port, dmx.min_time_break_to_break.read_only())?;
        let channel_view = dmx.channels.read_only();
        let is_sync_view = dmx.is_sync.read_only();
        let _ = thread::spawn(move || {
                #[cfg(feature = "thread_priority")]
                thread_priority::set_current_thread_priority(thread_priority::ThreadPriority::Max).unwrap_or_else(|e| {
                    eprintln!("Failed to set thread priority: \"{:?}\". Continuing anyways...", e)
                });
                loop {
                    // This can be unwrapped since the values can't be dropped while the thread is running
                    if is_sync_view.read().unwrap().clone() {
                        if handler_rec.recv().is_err() {
                            // If the channel is dropped by the other side, the thread will stop
                            break;
                        }
                    }

                    let channels = channel_view.read().unwrap().clone();

                    // If an error occurs, the thread will stop
                    if let Err(_) = agent.send_dmx_packet(channels) {
                        break;
                    }

                    //If the channel is dropped by the other side, the thread will stop
                    if let Err(mpsc::TrySendError::Disconnected(_)) = handler.try_send(()) {
                        break;
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
    pub fn open_sync(port: &str) -> Result<DMXSerial, serialport::Error> {
        let mut dmx = DMXSerial::open(port)?;
        dmx.set_sync();
        Ok(dmx)
    }

    /// Reopens the [DMXSerial] on the same [`path`].
    /// 
    /// It keeps the current [`channel`] values.
    pub fn reopen(&mut self) -> Result<(), serialport::Error> {
        let channels = self.get_channels();
        let new_dmx = DMXSerial::open(&self.name)?;
        *self = new_dmx;
        self.set_channels(channels);
        Ok(())
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
    pub fn set_channel(&mut self, channel: usize, value: u8) -> Result<(), DMXChannelValidityError> {
        check_valid_channel(channel)?;
        // RwLock can be unwrapped here
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
        // RwLock can be unwrapped here
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
    pub fn get_channel(&self, channel: usize) -> Result<u8, DMXChannelValidityError> {
        check_valid_channel(channel)?;
        // RwLock can be unwrapped here
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
        // RwLock can be unwrapped here
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
        // RwLock can be unwrapped here
        self.channels.write().unwrap().fill(0);
    }

    fn wait_for_update(&self) -> Result<(), DMXDisconnectionError> {
        self.agent.rx.recv().map_err(|_| DMXDisconnectionError)?;
        Ok(())
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
    pub fn update(&mut self) -> Result<(), DMXDisconnectionError> {
        self.update_async()?;
        self.wait_for_update().map_err(|_| DMXDisconnectionError)?;
        Ok(())
    }

    /// Updates the DMX data but returns immediately.
    /// 
    /// Useless in **async** mode.
    /// 
    pub fn update_async(&self) -> Result<(), DMXDisconnectionError> {
        self.agent.tx.send(()).map_err(|_| DMXDisconnectionError)?;
        Ok(())
    }

    /// Sets the DMX mode to **sync**.
    /// 
    pub fn set_sync(&mut self) {
        // RwLock can be unwrapped here
        *self.is_sync.write().unwrap() = true;
    }

    /// Sets the DMX mode to **async**.
    ///     
    pub fn set_async(&mut self) {
        // RwLock can be unwrapped here
        *self.is_sync.write().unwrap() = false;
    }

    /// Returns `true` if the DMX mode is **sync**.
    ///     
    pub fn is_sync(&self) -> bool {
        // RwLock can be unwrapped here
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
        // RwLock can be unwrapped here
        self.min_time_break_to_break.write().unwrap().clone_from(&time);
    }

    /// Returns the minimum [`Duration`] between two **DMX packets**.
    /// 
    /// [`Duration`]: time::Duration
    /// 
    pub fn get_packet_time(&self) -> time::Duration {
        // RwLock can be unwrapped here
        self.min_time_break_to_break.read().unwrap().clone()
    }

    /// Checks if the [`DMXSerial`] device is still connected.
    ///
    /// # Example
    /// 
    /// Basic usage:
    /// 
    /// ```
    /// # use open_dmx::DMXSerial;
    /// # fn main() {
    /// # let mut dmx = DMXSerial::open("COM3").unwrap();
    /// assert!(dmx.check_agent().is_ok()); // If not, the device got disconnected
    /// # }
    pub fn check_agent(&self) -> Result<(), DMXDisconnectionError> {
        if let Err(mpsc::TryRecvError::Disconnected) = self.agent.rx.try_recv() {
            return Err(DMXDisconnectionError);
        }
        Ok(())
    }
}

#[derive(Debug)]
struct AgentCommunication<T> {
    pub tx: mpsc::Sender<T>,
    pub rx: mpsc::Receiver<T>,
}

impl<T> AgentCommunication<T> {
    pub fn new(tx: mpsc::Sender<T>, rx: mpsc::Receiver<T>) -> AgentCommunication<T> {
        AgentCommunication {
            tx,
            rx,
        }
    }
}

struct DMXSerialAgent {
    port: Box<dyn SerialPort>,
    min_b2b: ReadOnly<time::Duration>,
}

impl DMXSerialAgent {

    pub fn open (port: &str, min_b2b: ReadOnly<time::Duration>) -> Result<DMXSerialAgent, serialport::Error> {
        let port = serialport::new(port, 200000).open()?;
        let dmx = DMXSerialAgent {
            port,
            min_b2b,
        };
        Ok(dmx)
    }
    fn send_break(&mut self) -> serialport::Result<()> {
        self.port.set_baud_rate(57600)?;
        self.port.set_data_bits(serialport::DataBits::Seven)?;
        self.port.set_stop_bits(serialport::StopBits::One)?;
        self.port.set_parity(serialport::Parity::None)?;
        self.port.set_flow_control(serialport::FlowControl::None)?;

        self.port.write(&[0x00])?;
        Ok(())
    }

    fn send_data(&mut self, data: &[u8]) -> serialport::Result<()> {
        self.port.set_baud_rate(250000)?;
        self.port.set_data_bits(serialport::DataBits::Eight)?;
        self.port.set_stop_bits(serialport::StopBits::Two)?;
        self.port.set_parity(serialport::Parity::None)?;
        self.port.set_flow_control(serialport::FlowControl::None)?;

        self.port.write(data)?;
        Ok(())
    }
    
    pub fn send_dmx_packet(&mut self, channels: [u8; DMX_CHANNELS]) -> serialport::Result<()> {
        let start = time::Instant::now();
        self.send_break()?;
        thread::sleep(TIME_BREAK_TO_DATA);
        let mut prefixed_data = [0; 513];// 1 start byte + 512 channels
        prefixed_data[1..].copy_from_slice(&channels);
        self.send_data(&prefixed_data)?;

        thread::sleep(self.min_b2b.read().unwrap().saturating_sub(start.elapsed()));

        Ok(())
    }
}