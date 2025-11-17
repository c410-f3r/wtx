use crate::{
  collection::Vector,
  database::client::mysql::{
    MysqlError,
    protocol::{Protocol, decode_wrapper_protocol::DecodeWrapperProtocol, lenenc::Lenenc},
  },
  de::Decode,
  misc::Usize,
};
use core::ops::Range;

type Params<'any> = (usize, &'any mut Vector<(bool, Range<usize>)>);

#[derive(Debug)]
pub(crate) struct TextRowRes<'de>(pub(crate) &'de [u8]);

impl<'de, E> Decode<'de, Protocol<Params<'_>, E>> for TextRowRes<'de>
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapperProtocol<'de, '_, Params<'_>>) -> Result<Self, E> {
    let columns = dw.other.0;
    dw.other.1.reserve(columns)?;
    let initial = *dw.bytes;
    let mut idx = 0;
    for _ in 0..columns {
      let [first, rest @ ..] = dw.bytes else {
        return Err(E::from(MysqlError::InvalidTextRowBytes.into()));
      };
      if *first == 251 {
        dw.other.1.push((true, idx..idx))?;
        *dw.bytes = rest;
        idx = idx.wrapping_add(1);
      } else {
        let len = {
          let begin = dw.bytes.len();
          let len = Usize::from(Lenenc::decode(dw)?.0).into_usize();
          let consumed_bytes = begin.wrapping_sub(dw.bytes.len());
          idx = idx.wrapping_add(consumed_bytes);
          len
        };
        let begin = idx;
        idx = idx.wrapping_add(len);
        *dw.bytes = dw.bytes.get(len..).unwrap_or_default();
        dw.other.1.push((false, begin..idx))?;
      }
    }
    Ok(Self(initial))
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    collection::Vector,
    database::client::mysql::protocol::{
      decode_wrapper_protocol::DecodeWrapperProtocol, text_row_res::TextRowRes,
    },
    de::Decode,
  };

  #[test]
  fn has_correct_indices() {
    let mut vector = Vector::new();
    let bytes = &mut &[4, b't', b'e', b's', b't'][..];
    let mut dw = DecodeWrapperProtocol { bytes, other: (1, &mut vector) };
    let rslt: crate::Result<_> = TextRowRes::decode(&mut dw);
    let _text_row_res = rslt.unwrap();
    assert_eq!(vector.as_ref(), &[(false, 1..5)]);
  }
}
