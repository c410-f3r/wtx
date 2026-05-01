use crate::{
  codec::{Decode, Encode},
  collection::LinearStorageLen,
  database::{
    Typed,
    client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, PostgresError, Ty},
  },
  misc::{
    Usize,
    counter_writer::{CounterWriterBytesTy, i32_write},
  },
};
use core::{
  fmt::{self, Debug, Display, Formatter},
  ops::{
    Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
  },
};

/// PostgreSQL range
#[derive(Debug)]
pub struct PgRange<T> {
  /// Start bound.
  pub start: Bound<T>,
  /// End bound.
  pub end: Bound<T>,
}

impl<T> PgRange<T> {
  /// Shortcut
  #[inline]
  pub const fn new(start: Bound<T>, end: Bound<T>) -> Self {
    Self { start, end }
  }
}

impl<'de, E, T> Decode<'de, Postgres<E>> for PgRange<T>
where
  E: From<crate::Error>,
  T: Decode<'de, Postgres<E>>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, '_>) -> Result<Self, E> {
    fn extract_elem_bytes<'bytes>(bytes: &mut &'bytes [u8]) -> crate::Result<&'bytes [u8]> {
      let [a, b, c, d, rest0 @ ..] = bytes else {
        return Err(crate::Error::from(PostgresError::InvalidRangeTy));
      };
      let len = Usize::from(u32::from_be_bytes([*a, *b, *c, *d])).usize();
      let Some((elem_bytes, rest1)) = rest0.split_at_checked(len) else {
        return Err(crate::Error::from(PostgresError::InvalidRangeTy));
      };
      *bytes = rest1;
      Ok(elem_bytes)
    }

    let [flags, rest @ ..] = dw.bytes() else {
      return Err(crate::Error::from(PostgresError::InvalidRangeTy).into());
    };
    let mut bytes = rest;
    let mut end = Bound::Unbounded;
    let mut start = Bound::Unbounded;
    if flags & u8::from(RangeFlags::Empty) != 0 {
      return Ok(PgRange { start, end });
    }
    if flags & u8::from(RangeFlags::LbInf) == 0 {
      *dw.bytes_mut() = extract_elem_bytes(&mut bytes)?;
      let value = T::decode(dw)?;
      start = if flags & u8::from(RangeFlags::LbInc) != 0 {
        Bound::Included(value)
      } else {
        Bound::Excluded(value)
      };
    }
    if flags & u8::from(RangeFlags::UbInf) == 0 {
      *dw.bytes_mut() = extract_elem_bytes(&mut bytes)?;
      let value = T::decode(dw)?;
      end = if flags & u8::from(RangeFlags::UbInc) != 0 {
        Bound::Included(value)
      } else {
        Bound::Excluded(value)
      };
    }
    Ok(PgRange { start, end })
  }
}

impl<E, T> Encode<Postgres<E>> for PgRange<T>
where
  E: From<crate::Error>,
  T: Encode<Postgres<E>> + PartialOrd,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
    let is_empty = match (&self.start, &self.end) {
      (Bound::Unbounded, _) | (_, Bound::Unbounded) => false,
      (Bound::Included(start), Bound::Excluded(end))
      | (Bound::Excluded(start), Bound::Included(end))
      | (Bound::Excluded(start), Bound::Excluded(end)) => start >= end,
      (Bound::Included(start), Bound::Included(end)) => start > end,
    };
    if is_empty {
      ew.buffer().extend_from_byte(RangeFlags::Empty.into())?;
      return Ok(());
    }
    let mut flags = 0u8;
    flags |= match self.start {
      Bound::Included(_) => u8::from(RangeFlags::LbInc),
      Bound::Unbounded => u8::from(RangeFlags::LbInf),
      Bound::Excluded(_) => 0,
    };
    flags |= match self.end {
      Bound::Included(_) => u8::from(RangeFlags::UbInc),
      Bound::Unbounded => u8::from(RangeFlags::UbInf),
      Bound::Excluded(_) => 0,
    };
    ew.buffer().extend_from_byte(flags)?;
    if let Bound::Excluded(elem) | Bound::Included(elem) = &self.start {
      i32_write(CounterWriterBytesTy::IgnoresLen, None, ew.buffer(), |local_sw| {
        elem.encode(&mut EncodeWrapper::new(local_sw))
      })?;
    }
    if let Bound::Excluded(elem) | Bound::Included(elem) = &self.end {
      i32_write(CounterWriterBytesTy::IgnoresLen, None, ew.buffer(), |local_sw| {
        elem.encode(&mut EncodeWrapper::new(local_sw))
      })?;
    }
    Ok(())
  }
}

impl<E, T> Typed<Postgres<E>> for PgRange<T>
where
  E: From<crate::Error>,
  T: Typed<Postgres<E>>,
{
  #[inline]
  fn runtime_ty(&self) -> Option<Ty> {
    T::static_ty().and_then(|el| el.range_ty())
  }

  #[inline]
  fn static_ty() -> Option<Ty> {
    T::static_ty().and_then(|el| el.range_ty())
  }
}

impl<T> Display for PgRange<T>
where
  T: Display,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match &self.start {
      Bound::Unbounded => f.write_str("(,")?,
      Bound::Excluded(el) => write!(f, "({el},")?,
      Bound::Included(el) => write!(f, "[{el},")?,
    }
    match &self.end {
      Bound::Unbounded => f.write_str(")")?,
      Bound::Excluded(el) => write!(f, "{el})")?,
      Bound::Included(el) => write!(f, "{el}]")?,
    }
    Ok(())
  }
}

impl<T> From<[Bound<T>; 2]> for PgRange<T> {
  #[inline]
  fn from(el: [Bound<T>; 2]) -> Self {
    let [start, end] = el;
    Self { start, end }
  }
}

impl<T> From<(Bound<T>, Bound<T>)> for PgRange<T> {
  #[inline]
  fn from(el: (Bound<T>, Bound<T>)) -> Self {
    Self { start: el.0, end: el.1 }
  }
}

impl<T> From<PgRange<T>> for [Bound<T>; 2] {
  #[inline]
  fn from(el: PgRange<T>) -> Self {
    [el.start, el.end]
  }
}

impl<T> From<PgRange<T>> for (Bound<T>, Bound<T>) {
  #[inline]
  fn from(el: PgRange<T>) -> Self {
    (el.start, el.end)
  }
}

impl<T> From<Range<T>> for PgRange<T> {
  #[inline]
  fn from(el: Range<T>) -> Self {
    Self { start: Bound::Included(el.start), end: Bound::Excluded(el.end) }
  }
}

impl<T> From<RangeFrom<T>> for PgRange<T> {
  #[inline]
  fn from(el: RangeFrom<T>) -> Self {
    Self { start: Bound::Included(el.start), end: Bound::Unbounded }
  }
}

impl<T> From<RangeInclusive<T>> for PgRange<T> {
  #[inline]
  fn from(el: RangeInclusive<T>) -> Self {
    let (start, end) = el.into_inner();
    Self { start: Bound::Included(start), end: Bound::Included(end) }
  }
}

impl<T> From<RangeTo<T>> for PgRange<T> {
  #[inline]
  fn from(el: RangeTo<T>) -> Self {
    Self { start: Bound::Unbounded, end: Bound::Excluded(el.end) }
  }
}

impl<T> From<RangeToInclusive<T>> for PgRange<T> {
  #[inline]
  fn from(el: RangeToInclusive<T>) -> Self {
    Self { start: Bound::Unbounded, end: Bound::Included(el.end) }
  }
}

impl<T> TryFrom<PgRange<T>> for Range<T> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: PgRange<T>) -> Result<Self, Self::Error> {
    if let (Bound::Included(start), Bound::Excluded(end)) = (value.start, value.end) {
      Ok(start..end)
    } else {
      Err(PostgresError::InvalidRangeTy.into())
    }
  }
}

impl<T> TryFrom<PgRange<T>> for RangeFrom<T> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: PgRange<T>) -> Result<Self, Self::Error> {
    if let (Bound::Included(start), Bound::Unbounded) = (value.start, value.end) {
      Ok(start..)
    } else {
      Err(PostgresError::InvalidRangeTy.into())
    }
  }
}

impl<T> TryFrom<PgRange<T>> for RangeFull {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: PgRange<T>) -> Result<Self, Self::Error> {
    if let (Bound::Unbounded, Bound::Unbounded) = (value.start, value.end) {
      Ok(..)
    } else {
      Err(PostgresError::InvalidRangeTy.into())
    }
  }
}

impl<T> TryFrom<PgRange<T>> for RangeInclusive<T> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: PgRange<T>) -> Result<Self, Self::Error> {
    if let (Bound::Included(start), Bound::Included(end)) = (value.start, value.end) {
      Ok(start..=end)
    } else {
      Err(PostgresError::InvalidRangeTy.into())
    }
  }
}

impl<T> TryFrom<PgRange<T>> for RangeTo<T> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: PgRange<T>) -> Result<Self, Self::Error> {
    if let (Bound::Unbounded, Bound::Excluded(end)) = (value.start, value.end) {
      Ok(..end)
    } else {
      Err(PostgresError::InvalidRangeTy.into())
    }
  }
}

impl<T> TryFrom<PgRange<T>> for RangeToInclusive<T> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: PgRange<T>) -> Result<Self, Self::Error> {
    if let (Bound::Unbounded, Bound::Included(end)) = (value.start, value.end) {
      Ok(..=end)
    } else {
      Err(PostgresError::InvalidRangeTy.into())
    }
  }
}

#[derive(Clone, Copy, Debug)]
enum RangeFlags {
  Empty,
  LbInc,
  UbInc,
  LbInf,
  UbInf,
}

impl From<RangeFlags> for u8 {
  #[inline]
  fn from(value: RangeFlags) -> Self {
    match value {
      RangeFlags::Empty => 0b0000_0001,
      RangeFlags::LbInc => 0b0000_0010,
      RangeFlags::UbInc => 0b0000_0100,
      RangeFlags::LbInf => 0b0000_1000,
      RangeFlags::UbInf => 0b0001_0000,
    }
  }
}

macro_rules! range {
  ($name:ident) => {
    impl<'de, E, T> Decode<'de, Postgres<E>> for $name<T>
    where
      E: From<crate::Error>,
      PgRange<T>: Decode<'de, Postgres<E>>,
    {
      #[inline]
      fn decode(dw: &mut DecodeWrapper<'de, '_>) -> Result<Self, E> {
        Ok(PgRange::<T>::decode(dw)?.try_into()?)
      }
    }

    impl<E, T> Encode<Postgres<E>> for $name<T>
    where
      E: From<crate::Error>,
      T: PartialOrd,
      for<'any> PgRange<&'any T>: Encode<Postgres<E>>,
    {
      #[inline]
      fn encode(&self, ew: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
        PgRange::new(self.start_bound(), self.end_bound()).encode(ew)
      }
    }

    impl<E, T> Typed<Postgres<E>> for $name<T>
    where
      E: From<crate::Error>,
      PgRange<T>: Typed<Postgres<E>>,
    {
      #[inline]
      fn runtime_ty(&self) -> Option<Ty> {
        <Self as Typed<Postgres<E>>>::static_ty()
      }

      #[inline]
      fn static_ty() -> Option<Ty> {
        PgRange::<T>::static_ty()
      }
    }
  };
}

range!(Range);
range!(RangeFrom);
range!(RangeInclusive);
range!(RangeTo);
range!(RangeToInclusive);
