use crate::{
  database::client::postgres::PostgresError,
  misc::{bytes_split1, into_rslt, FromRadix10},
};
use core::any::type_name;

#[derive(Debug)]
pub(crate) enum Authentication<'bytes> {
  Ok,
  Sasl(&'bytes [u8]),
  SaslContinue { iterations: u32, payload: &'bytes [u8], nonce: &'bytes [u8], salt: &'bytes [u8] },
  SaslFinal(&'bytes [u8]),
}

impl<'bytes> TryFrom<&'bytes [u8]> for Authentication<'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from(bytes: &'bytes [u8]) -> Result<Self, Self::Error> {
    let (n, rest) = if let [a, b, c, d, rest @ ..] = bytes {
      (u32::from_be_bytes([*a, *b, *c, *d]), rest)
    } else {
      return Err(PostgresError::UnexpectedValueFromBytes { expected: type_name::<Self>() }.into());
    };
    Ok(match n {
      0 => Self::Ok,
      10 => Self::Sasl(rest),
      11 => {
        let mut iter = bytes_split1(rest, b',');
        let mut iterations = None;
        let mut nonce = None;
        let mut salt = None;
        while let Some([key, _, local_rest @ ..]) = iter.next() {
          match key {
            b'i' => {
              iterations = Some(u32::from_radix_10(local_rest)?);
            }
            b'r' => {
              nonce = Some(local_rest);
            }
            b's' => {
              salt = Some(local_rest);
            }
            _ => {}
          }
        }
        Self::SaslContinue {
          iterations: into_rslt(iterations)?,
          nonce: into_rslt(nonce)?,
          payload: rest,
          salt: into_rslt(salt)?,
        }
      }
      12 => {
        let mut iter = bytes_split1(rest, b',');
        let mut verifier = None;
        while let Some([b'v', _, local_rest @ ..]) = iter.next() {
          verifier = Some(local_rest);
        }
        Self::SaslFinal(into_rslt(verifier)?)
      }
      _ => {
        return Err(
          PostgresError::UnexpectedValueFromBytes { expected: type_name::<Self>() }.into(),
        );
      }
    })
  }
}
