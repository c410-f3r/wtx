# Calendar

Provides basic primitives to work with time-related operations.

## Main Components

* `Date`: Proleptic Gregorian calendar. Can represent years from -32767 to 32766.

* `DateTime`: ISO-8601 representation with timezones.

* `Duration`: Time span in nanoseconds. Can be negative unlike `core::time::Duration`.

* `Instant`: A specific point in time. Contains the underlying mechanism that provides a timestamp.

* `Time` Clock time with nanosecond precision.

Also supports arithmetic operations and flexible formatting.

## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/calendar.rs}}
```
