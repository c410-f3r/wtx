pub(crate) mod statement;
pub(crate) mod statement_builder;
pub(crate) mod statements;
pub(crate) mod statements_misc;

use crate::misc::{ArrayVector, UriRef, str_split_once1, str_split1};

pub(crate) type U64Array = ArrayVector<u8, 20>;

#[inline]
pub(crate) fn query_walker<'uri>(
  uri: &'uri UriRef<'_>,
  mut cb: impl FnMut(&'uri str, &'uri str) -> crate::Result<()>,
) -> crate::Result<()> {
  let mut pair_iter = str_split1(uri.query_and_fragment(), b'&');
  if let Some(mut key_value) = pair_iter.next() {
    key_value = key_value.get(1..).unwrap_or_default();
    if let Some((key, value)) = str_split_once1(key_value, b'=') {
      cb(key, value)?;
    }
  }
  for key_value in pair_iter {
    if let Some((key, value)) = str_split_once1(key_value, b'=') {
      cb(key, value)?;
    }
  }
  Ok(())
}

#[inline]
pub(crate) fn u64_array(mut value: u64) -> U64Array {
  let mut idx: u8 = 20;
  let mut buffer = [0u8; 20];
  for local_idx in 1..=20 {
    idx = 20u8.wrapping_sub(local_idx);
    let Some(elem) = buffer.get_mut(usize::from(idx)) else {
      break;
    };
    *elem = u8::try_from(value % 10).unwrap_or_default().wrapping_add(48);
    value /= 10;
    if value == 0 {
      break;
    }
  }
  let mut data = [0u8; 20];
  let len = 20u16.wrapping_sub(idx.into());
  let slice = data.get_mut(..usize::from(len)).unwrap_or_default();
  slice.copy_from_slice(buffer.get(usize::from(idx)..).unwrap_or_default());
  ArrayVector::from_parts(data, Some(len.into()))
}

#[cfg(test)]
pub(crate) mod tests {
  use crate::database::client::rdbms::u64_array;

  #[test]
  fn has_correct_stmt_number() {
    assert_eq!(u64_array(0).as_slice(), b"0");
    assert_eq!(u64_array(12).as_slice(), b"12");
    assert_eq!(u64_array(1844674407370955161).as_slice(), b"1844674407370955161");
    assert_eq!(u64_array(18446744073709551615).as_slice(), b"18446744073709551615");
  }
}
