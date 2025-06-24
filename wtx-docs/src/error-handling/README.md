# Error Handling

The majority of operations performed by `WTX` is fallible, in other words, most functions or methods return a `Result` enum instead of panicking under the hood. A considerable effort is put to hint the compiler that a branch is unreachable to optimize code generation but that is another topic.

Due to this characteristic downstream users are encouraged to create their own `Error` enum with a `WTX` variant along side a `From` trait implementation. Not to mention the unlocking of the useful `?` operator that performs the automatically conversion of any supported error element.

```rust,edition2024
extern crate wtx;

use wtx::de::FromRadix10;

#[derive(Debug)]
pub enum Error {
    MyDogAteMyHomework,
    RanOutOfCoffee,
    Wtx(wtx::Error)
}

impl From<wtx::Error> for Error {
    fn from(from: wtx::Error) -> Self {
        Self::Wtx(from)
    }
}

fn main() -> Result<(), Error> {
    let _u16_from_bytes = u16::from_radix_10(&[49][..])?;
    let _u16_from_i8 = u16::try_from(1i8).map_err(wtx::Error::from)?;
    Ok(())
}
```

All these conventions are of course optional. If desired everything can be unwrapped using the `Result::unwrap` method.

When you encounter an error, try take a look at the available documentation of that specific error (<https://docs.rs/wtx/latest/wtx/enum.Error.html>). If the documentation didn't help, feel free to reach out for potential improvements.

