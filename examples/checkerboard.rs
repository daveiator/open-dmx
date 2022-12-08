use open_dmx::{DMXSerial, DMX_CHANNELS};
fn main() {
    let mut dmx = DMXSerial::open("COM3").unwrap();
    let mut channels = [0; DMX_CHANNELS];
    channels.iter_mut().enumerate().for_each(|(i, value)| *value = if i % 2 == 0 { 255 } else { 0 });
    dmx.set_channels(channels);
}