use open_dmx::DMXSerial;

fn main() {
    let mut dmx = DMXSerial::open("COM3").unwrap();
    dmx.set_sync();
    loop {
        let _ = strobe(&mut dmx);
        println!("Device has been disconnected! Reopening...");
        match dmx.reopen() {
            Ok(_) => println!("Device has been reopened!"),
            Err(e) => {
                println!("Error reopening device: {}", e);
                println!("Waiting 1 second before trying again...");
                std::thread::sleep(std::time::Duration::from_secs(1));
            },
        }
    }
}

fn strobe(dmx: &mut DMXSerial) -> Result<(), Box<dyn std::error::Error>>{
    println!("Sending strobe packets...");
    loop {
        dmx.set_channels([255; 512]);
        dmx.update()?;
        dmx.set_channels([0; 512]);
        dmx.update()?;
    }
}