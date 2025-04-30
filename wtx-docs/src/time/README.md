# Time

Provides basic primitives to work with time-related operations.

* `Time`: Clock time with nanosecond precision.
* `Date`: Proleptic Gregorian calendar. Can represent years from -32767 to +32767.
* `DateTime`: ISO-8601 representation with a fixed UTC timezone.

Web development generally requires time structures, as such, this feature isn't optional.

## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/time.rs}}
```
