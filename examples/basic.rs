use open_dmx::DMXSerial;

fn main() {
    let mut dmx = DMXSerial::open("COM3").unwrap();
    dmx.set_channels([255; 512]);
    dmx.set_channel(1, 0).unwrap();
}