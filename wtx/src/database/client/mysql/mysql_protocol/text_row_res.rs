use crate::{
  database::client::mysql::{
    MysqlError,
    mysql_protocol::{
      MysqlProtocol, decode_wrapper_protocol::DecodeWrapperProtocol, lenenc::Lenenc,
    },
  },
  misc::{Decode, Usize, Vector},
};
use core::ops::Range;

type Params<'any> = (usize, &'any mut Vector<(bool, Range<usize>)>);

#[derive(Debug)]
pub(crate) struct TextRowRes;

impl<'de, E> Decode<'de, MysqlProtocol<Params<'_>, E>> for TextRowRes
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(aux: &mut (), dw: &mut DecodeWrapperProtocol<'de, '_, Params<'_>>) -> Result<Self, E> {
    dw.other.1.reserve(dw.other.0)?;
    let mut idx = 0;
    for _ in 0..dw.other.0 {
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
          let len = Usize::try_from(Lenenc::decode(aux, dw)?.0)?.into_usize();
          idx = idx.wrapping_add(dw.bytes.len().wrapping_sub(begin));
          len
        };
        let begin = idx;
        idx = idx.wrapping_add(len);
        *dw.bytes = dw.bytes.get(len..).unwrap_or_default();
        dw.other.1.push((false, begin..idx))?;
      }
    }
    Ok(Self)
  }
}
