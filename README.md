# open_dmx &emsp; [![Latest Release][crates-io-badge]][crates-io-url] [![Documentation][docs-rs-img]][docs-rs-url] [![License][license-badge]]()

[crates-io-badge]: https://img.shields.io/crates/v/open_dmx.svg?style=for-the-badge
[crates-io-url]: https://crates.io/crates/open_dmx
[docs-rs-img]: https://img.shields.io/docsrs/open_dmx?style=for-the-badge
[docs-rs-url]: https://docs.rs/open_dmx
[license-badge]: https://img.shields.io/crates/l/open_dmx.svg?style=for-the-badge

A wrapper around the [**serialport**](https://crates.io/crates/serialport) library to send [DMX](https://en.wikipedia.org/wiki/DMX512) data over a serial port via the *Open-DMX(RS-485)* protocol

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

`DMXSerial` updates its channels automatically to the Serial Port for a stable connection. For strobe effects `DMXSerial.update()` can be used, which blocks the main thread until a packet is sent over serial.

The automatic sending can also be disabled with `DMXSerial::open_sync(path)` or `DMXSerial.set_sync()` 

Works with COM-Ports on Windows and TTYPorts on Unix systems.

### Dependencies

For linux `pkg-config` and `libudev` are required.