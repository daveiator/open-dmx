use open_dmx::DMXSerial;

fn main() {
    let mut dmx = DMXSerial::open("COM3").unwrap();
    dmx.set_sync();
    //strobe
    loop {
        dmx.set_channels([255; 512]);
        dmx.update();
        dmx.set_channels([0; 512]);
        dmx.update();
    }
}