# Calendar

Provides basic primitives to work with time-related operations.

* `Date`: Proleptic Gregorian calendar. Can represent years from -32767 to 32767.

* `DateTime`: ISO-8601 representation with timezones.

* `Duration`: Time span in nanoseconds. Can be negative unlike `core::time::Duration`.

* `Instant`: A specific point in time. Contains the underlying mechanism that provides a timestamp.

* `Time` Clock time with nanosecond precision.

Also supports arithmetic operations and flexible formatting.

## Embedded devices

`no_std` users that utilize the `embassy` crate should first make a UDP request to a NTP server and then call the `wtx::calendar::set_epoch_offset` function once at start-up, otherwise timestamps will represent the time since boot.

## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-examples/examples/calendar.rs}}
```
