# Calendar

Provides basic primitives to work with time-related operations.

* `Time`: Clock time with nanosecond precision.
* `Date`: Proleptic Gregorian calendar. Can represent years from -32767 to 32766.
* `DateTime`: ISO-8601 representation with timezones.

To use this functionality, it is necessary to activate the `calendar` feature.

## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/calendar.rs}}
```
