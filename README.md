# open-dmx &emsp; [![Latest Release][crates-io-badge]][crates-io-url] [![Documentation][docs-rs-img]][docs-rs-url]

[crates-io-badge]: https://img.shields.io/crates/v/open-dmx.svg
[crates-io-url]: https://crates.io/crates/open-dmx
[docs-rs-img]: https://docs.rs/open-dmx/badge.svg
[docs-rs-url]: https://docs.rs/open-dmx

**A wrapper around the [**serial**](https://crates.io/crates/serial) library to send [DMX](https://en.wikipedia.org/wiki/DMX512) data over a serial port via the *Open-DMX(RS-485)* protocol**

---
## Basic Setup
```rust
use open_dmx::DMXSerial;

fn main() {
    let mut dmx = DMXSerial::open("COM3").unwrap();
    dmx.set_channels([255; 512]);
    dmx.set_channel(1, 0).unwrap();
}
```

`DMXSerial` updates it's channels automatically to the Serial Port for a stable connection. For strobe effects `DMXSerial.update()` can be used, which blocks the main thread until a packet is sent over serial.

The automatic sending can also be disabled with `DMXSerial::open_sync(path)` or `DMXSerial.set_sync()` 