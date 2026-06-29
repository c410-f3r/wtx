use crate::{calendar::Date, collections::ArrayStringU8};

#[inline]
pub(crate) fn push_four_digits_year<const N: usize>(
  date: Date,
  string: &mut ArrayStringU8<N>,
) -> crate::Result<()> {
  string.push_str(&date.year().num_str())?;
  Ok(())
}

#[inline]
pub(crate) fn push_two_spaces_day<const N: usize>(
  date: Date,
  string: &mut ArrayStringU8<N>,
) -> crate::Result<()> {
  let [b0, b1] = date.day().num_str().as_bytes() else {
    return Ok(());
  };
  if *b0 == b'0' {
    string.push(' ')?;
    string.push((*b1).into())?;
  } else {
    string.push((*b0).into())?;
    string.push((*b1).into())?;
  }
  Ok(())
}
