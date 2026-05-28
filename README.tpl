# {{crate}}

{{readme}}

## Usage

Add this crate as a dependency in your `Cargo.toml`:

```toml
[dependencies]
embedded-io = "0.7.1"
embedded-io-async = "0.7.0"

[dev-dependencies]
mock-serial-async = "{{version}}"
```

## Alternatives
- [mock-embedded-io](https://crates.io/crates/mock-embedded-io)
  This provides a mock for `embedded-io` and `embedded-io-async`, but does not have a
  similar API to `embedded-hal-mock`

# License

{{license}}