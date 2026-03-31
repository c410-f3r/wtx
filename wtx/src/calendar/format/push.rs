use crate::{calendar::Date, collection::ArrayStringU8};

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
  let [a, b] = date.day().num_str().as_bytes() else {
    return Ok(());
  };
  if *a == b'0' {
    string.push(' ')?;
    string.push((*b).into())?;
  } else {
    string.push((*a).into())?;
    string.push((*b).into())?;
  }
  Ok(())
}
