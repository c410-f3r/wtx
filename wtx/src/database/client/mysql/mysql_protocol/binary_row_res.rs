use crate::{
  collection::Vector,
  database::client::mysql::{
    MysqlError, MysqlStatement, Ty,
    mysql_protocol::{
      MysqlProtocol, decode_wrapper_protocol::DecodeWrapperProtocol, lenenc::Lenenc,
    },
  },
  misc::{Decode, Usize},
};
use core::ops::Range;

type Params<'any> = (&'any MysqlStatement<'any>, &'any mut Vector<(bool, Range<usize>)>);

#[derive(Debug)]
pub(crate) struct BinaryRowRes<'de>(pub(crate) &'de [u8]);

impl<'de, E> Decode<'de, MysqlProtocol<Params<'_>, E>> for BinaryRowRes<'de>
where
  E: From<crate::Error>,
{
  #[inline]
  fn decode(aux: &mut (), dw: &mut DecodeWrapperProtocol<'de, '_, Params<'_>>) -> Result<Self, E> {
    let initial = *dw.bytes;
    let [a, rest0 @ ..] = dw.bytes else {
      return Err(E::from(MysqlError::InvalidBinaryRowBytes.into()));
    };
    *dw.bytes = rest0;
    if *a != 0 {
      return Err(E::from(MysqlError::InvalidBinaryRowBytes.into()));
    }
    let bitmap_len = dw.other.0.columns_len.wrapping_add(9) / 8;
    let Some((bitmap, rest1)) = dw.bytes.split_at_checked(bitmap_len) else {
      return Err(E::from(MysqlError::InvalidBinaryRowBytes.into()));
    };
    *dw.bytes = rest1;
    dw.other.1.reserve(dw.other.0.columns_len)?;
    let mut idx: usize = bitmap_len.wrapping_add(1);
    for (column_idx, column) in dw.other.0.columns().enumerate() {
      let column_null_idx = column_idx.wrapping_add(2);
      let byte_idx = column_null_idx / 8;
      let bit_idx = column_null_idx % 8;
      let is_null = bitmap.get(byte_idx).copied().unwrap_or_default() & (1u8 << bit_idx) != 0;
      if is_null {
        dw.other.1.push((true, idx..idx))?;
        continue;
      }
      let len = match column.ty_params.ty {
        Ty::Double => 8,
        Ty::Float => 4,
        Ty::Long | Ty::Int24 => 4,
        Ty::LongLong => 8,
        Ty::Short | Ty::Year => 2,
        Ty::Tiny => 1,

        Ty::Bit
        | Ty::Blob
        | Ty::Decimal
        | Ty::Enum
        | Ty::Geometry
        | Ty::Json
        | Ty::LongBlob
        | Ty::MediumBlob
        | Ty::NewDecimal
        | Ty::Set
        | Ty::String
        | Ty::TinyBlob
        | Ty::VarChar
        | Ty::VarString => {
          let before = dw.bytes.len();
          let rslt = Usize::try_from(Lenenc::decode(aux, dw)?.0)?.into_usize();
          let diff = before.wrapping_sub(dw.bytes.len());
          idx = idx.wrapping_add(diff);
          rslt
        }

        Ty::Date | Ty::Datetime | Ty::Time | Ty::Timestamp => {
          usize::from(dw.bytes.first().copied().unwrap_or_default()).wrapping_add(1)
        }

        Ty::Null => return Err(E::from(MysqlError::InvalidBinaryRowBytes.into())),
      };
      let begin = idx;
      idx = idx.wrapping_add(len);
      *dw.bytes = dw.bytes.get(len..).unwrap_or_default();
      dw.other.1.push((false, begin..idx))?;
    }
    Ok(Self(initial))
  }
}
