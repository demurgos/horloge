# Horloge - clock abstraction for Rust

The `horloge` Rust crate defines trait for time sources (`Now`) and sleeping
(`Sleep`). These traits allow to abstract over the concrete implementation.
This enables writing generic code using the system clock when deployed or a
virtual clock during tests.

This crate provides trait definitions, and implementation providing support for
the major time libraries in the Rust ecosystem.

The name "horloge" means _clock_ in french.

## Integration

The intent is to provide integration with most libraries in the ecosystem.
Here are the currently support libraries. Contributions are welcome.

### Time model

- [`::std::time::Instant`](https://doc.rust-lang.org/stable/std/time/struct.Instant.html)
- [`::chrono::DateTime<::chono::Utc>`, version 0.4](https://docs.rs/chrono/0.4/chrono/struct.DateTime.html)

### Sleep scheduler

- [`::tokio::time::Sleep`, version 1.x](https://docs.rs/tokio/latest/tokio/time/fn.sleep.html)

## Licence

MIT
