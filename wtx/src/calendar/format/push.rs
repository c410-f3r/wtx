use crate::{calendar::Date, collection::ArrayStringU8, de::i16_string};

#[inline]
pub(crate) fn push_four_digit_year<const N: usize>(
  date: Date,
  string: &mut ArrayStringU8<N>,
) -> crate::Result<()> {
  let year = i16_string(date.year().num());
  let (num, zeros) = if year.len() <= 4 {
    if let [b'-', rest @ ..] = year.as_bytes() {
      string.push('-')?;
      (rest, 5u8.wrapping_sub(year.len()))
    } else {
      (year.as_bytes(), 4u8.wrapping_sub(year.len()))
    }
  } else {
    (year.as_bytes(), 0)
  };
  for _ in 0..zeros {
    string.push('0')?;
  }
  for elem in num {
    string.push((*elem).into())?;
  }
  Ok(())
}

#[inline]
pub(crate) fn push_two_space_day<const N: usize>(
  date: Date,
  string: &mut ArrayStringU8<N>,
) -> crate::Result<()> {
  let [a, b] = date.day().num_str().as_bytes() else {
    return Ok(());
  };
  if *a == b'0' {
    string.push_str(date.day().num_str())?;
  } else {
    string.push((*a).into())?;
    string.push((*b).into())?;
  }
  Ok(())
}
