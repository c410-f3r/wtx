mod incomplete_utf8_char;
mod rng;
mod traits;
pub(crate) mod uri_parts;
mod utf8_errors;

pub(crate) use incomplete_utf8_char::{CompleteErr, IncompleteUtf8Char};
pub(crate) use rng::Rng;
pub(crate) use traits::{AsyncBounds, Expand, SingleTypeStorage};
pub(crate) use utf8_errors::{ExtUtf8Error, StdUtf8Error};

pub(crate) fn from_utf8_opt(bytes: &[u8]) -> Option<&str> {
    #[cfg(feature = "simdutf8")]
    return simdutf8::basic::from_utf8(bytes).ok();
    #[cfg(not(feature = "simdutf8"))]
    return core::str::from_utf8(bytes).ok();
}

pub(crate) fn from_utf8_ext_rslt(bytes: &[u8]) -> Result<&str, ExtUtf8Error> {
    let err = match from_utf8_std_rslt(bytes) {
        Ok(elem) => return Ok(elem),
        Err(error) => error,
    };
    let (_valid_bytes, after_valid) = bytes.split_at(err.valid_up_to);
    match err.error_len {
        None => Err(ExtUtf8Error::Incomplete {
            incomplete_ending_char: {
                let opt = IncompleteUtf8Char::new(after_valid);
                opt.ok_or(ExtUtf8Error::Invalid)?
            },
        }),
        Some(_) => Err(ExtUtf8Error::Invalid),
    }
}

pub(crate) fn from_utf8_std_rslt(bytes: &[u8]) -> Result<&str, StdUtf8Error> {
    #[cfg(feature = "simdutf8")]
    return simdutf8::compat::from_utf8(bytes).map_err(|element| StdUtf8Error {
        valid_up_to: element.valid_up_to(),
        error_len: element.error_len(),
    });
    #[cfg(not(feature = "simdutf8"))]
    return core::str::from_utf8(bytes).map_err(|element| StdUtf8Error {
        valid_up_to: element.valid_up_to(),
        error_len: element.error_len(),
    });
}

#[cfg(test)]
mod tests {
    use crate::UriParts;

    #[test]
    fn uri_parts_generates_correct_output() {
        assert_eq!(
            UriParts::from("foo://user:pass@sub.domain.com:80/pa/th?query=value#hash"),
            UriParts {
                authority: "user:pass@sub.domain.com:80",
                host: "sub.domain.com:80",
                hostname: "sub.domain.com",
                href: "/pa/th?query=value#hash"
            }
        );
    }
}
