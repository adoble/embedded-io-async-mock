# embedded-io-async-mock

A small mock serial communication helper using `embedded-io-async`.

This crate provides an asynchronous mock serial peripheral that can be used in tests to verify expected read/write/flush transactions.

It attempts to provide the missing async serial functionality in [embedded-hal-mock](https://docs.rs/embedded-hal-mock/0.11.1/embedded_hal_mock/index.html).
As such, it orientates itself around the API, but differs in some respects.

### Features

- [Mock] records expected serial transactions.
- Supports `Read` and `Write` semantics from `embedded-io-async`.
- Useful for unit testing async serial drivers.

### Example

```rust
use embedded_io_async_mock::{Mock as SerialAsyncMock, Transaction as SerialTransaction};
use embedded_io_async::{Read, Write};

    let expectations = [
        SerialTransaction::write(b"VOL;"),
        SerialTransaction::flush(),
        SerialTransaction::read(b"42"),
    ];

    let mut serial = SerialAsyncMock::new(&expectations);

    assert!(serial.write(b"VOL;").await.is_ok());
    assert!(serial.flush().await.is_ok());

    let mut buf = [0 as u8; 2];
    assert!(serial.read(&mut buf).await.is_ok());

    serial.done();
```


## Usage

Add this crate as a dependency in your `Cargo.toml`:

```toml
[dependencies]
embedded-io = "0.7.1"
embedded-io-async = "0.7.0"

[dev-dependencies]
mock-serial-async = "0.1.0"
```

## Alternatives
- [mock-embedded-io](https://crates.io/crates/mock-embedded-io)
  This provides a mock for `embedded-io` and `embedded-io-async`, but does not have a
  similar API to `embedded-hal-mock`

# License

MIT
