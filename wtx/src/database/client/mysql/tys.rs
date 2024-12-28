macro_rules! kani {
  ($name:ident, $ty:ty) => {
    #[cfg(kani)]
    #[kani::proof]
    fn $name(instance: $ty) {
      let mut vec = &mut crate::misc::FilledBuffer::_new();
      {
        let mut sw = crate::misc::FilledBufferWriter::new(0, &mut vec);
        let mut ev = EncodeValue::new(&mut sw);
        Encode::<Mysql<crate::Error>>::encode(&instance, &mut ev).unwrap();
        let decoded: $ty = Decode::<Mysql<crate::Error>>::decode(&DecodeValue::new(
          ev.sw()._curr_bytes(),
          crate::database::client::mysql::Ty::Null,
        ))
        .unwrap();
        assert_eq!(instance, decoded);
      }
      vec._clear();
    }
  };
}

macro_rules! test {
  ($name:ident, $ty:ty, $instance:expr) => {
    #[cfg(test)]
    #[test]
    fn $name() {
      let vec = &mut crate::misc::FilledBuffer::_new();
      let mut sw = crate::misc::SuffixWriter::new(0, vec._vector_mut());
      let mut ev = EncodeWrapper::new(&mut sw);
      let instance: $ty = $instance;
      Encode::<Mysql<crate::Error>>::encode(&instance, &mut ev).unwrap();
      let decoded: $ty = Decode::<Mysql<crate::Error>>::decode(&mut DecodeWrapper::new(
        ev.sw()._curr_bytes(),
        crate::database::client::mysql::Ty::Tiny,
      ))
      .unwrap();
      assert_eq!(instance, decoded);
    }
  };
}

#[cfg(feature = "chrono")]
mod chrono {
  use crate::{
    database::{
      client::mysql::{DecodeWrapper, EncodeWrapper, Mysql, Ty},
      DatabaseError, Typed,
    },
    misc::{Decode, Encode, Usize},
  };
  use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
  use core::any::type_name;

  impl<E> Decode<'_, Mysql<E>> for DateTime<Utc>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      let naive = <NaiveDateTime as Decode<Mysql<E>>>::decode(dv)?;
      Ok(Utc.from_utc_datetime(&naive))
    }
  }
  impl<E> Encode<Mysql<E>> for DateTime<Utc>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, ev: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      Encode::<Mysql<E>>::encode(&self.naive_utc(), ev)
    }
  }
  impl<E> Typed<Mysql<E>> for DateTime<Utc>
  where
    E: From<crate::Error>,
  {
    const TY: Ty = Ty::Timestamp;
  }

  impl<E> Decode<'_, Mysql<E>> for NaiveDate
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      date_decode(dv).map(|el| el.1).map_err(E::from)
    }
  }
  impl<E> Encode<Mysql<E>> for NaiveDate
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, ev: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      date_encode(self, ev, 4).map_err(E::from)
    }
  }
  impl<E> Typed<Mysql<E>> for NaiveDate
  where
    E: From<crate::Error>,
  {
    const TY: Ty = Ty::Date;
  }

  impl<E> Decode<'_, Mysql<E>> for NaiveDateTime
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      let (len, date) = date_decode(dv).map_err(E::from)?;
      Ok(if len > 4 {
        let bytes = if let [_, _, _, _, _, bytes @ ..] = dv.bytes() { bytes } else { &[] };
        date.and_time(time_decode(len - 4, bytes)?)
      } else {
        date.and_time(NaiveTime::default())
      })
    }
  }
  impl<E> Encode<Mysql<E>> for NaiveDateTime
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, ev: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      let len = date_len(self);
      date_encode(&self.date(), ev, len)?;
      if len > 4 {
        time_encode(&self.time(), len > 7, ev)?;
      }
      Ok(())
    }
  }
  impl<E> Typed<Mysql<E>> for NaiveDateTime
  where
    E: From<crate::Error>,
  {
    const TY: Ty = Ty::Datetime;
  }

  #[inline]
  fn date_decode(dv: &mut DecodeWrapper<'_>) -> crate::Result<(u8, NaiveDate)> {
    let [len, year_a, year_b, month, day] = dv.bytes() else {
      return Err(
        DatabaseError::UnexpectedBufferSize {
          expected: 5,
          received: Usize::from(dv.bytes().len()).into(),
        }
        .into(),
      );
    };
    let year = i16::from_be_bytes([*year_a, *year_b]);
    let date = NaiveDate::from_ymd_opt(year.into(), (*month).into(), (*day).into())
      .ok_or(DatabaseError::UnexpectedValueFromBytes { expected: type_name::<NaiveDate>() })?;
    Ok((*len, date))
  }

  #[inline]
  fn date_encode(date: &NaiveDate, ev: &mut EncodeWrapper<'_, '_>, len: u8) -> crate::Result<()> {
    let year = u16::try_from(date.year()).map_err(|_err| {
      DatabaseError::UnexpectedValueFromBytes { expected: type_name::<NaiveDate>() }
    })?;
    let [year_a, year_b] = year.to_le_bytes();
    ev.sw().extend_from_slice(&[
      len,
      year_a,
      year_b,
      date.month().try_into().unwrap_or_default(),
      date.day().try_into().unwrap_or_default(),
    ])?;
    Ok(())
  }

  fn date_len(time: &NaiveDateTime) -> u8 {
    // to save space the packet can be compressed:
    match (
      time.hour(),
      time.minute(),
      time.second(),
      #[allow(deprecated)]
      time.timestamp_subsec_nanos(),
    ) {
      (0, 0, 0, 0) => 4,
      (_, _, _, 0) => 7,
      (_, _, _, _) => 11,
    }
  }

  #[inline]
  fn pg_epoch_nd() -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(2000, 1, 1)
  }

  #[inline]
  fn pg_epoch_ndt() -> Option<NaiveDateTime> {
    pg_epoch_nd()?.and_hms_opt(0, 0, 0)
  }

  #[inline]
  fn time_decode(len: u8, bytes: &[u8]) -> crate::Result<NaiveTime> {
    let (hours, minutes, seconds, micros) = if len > 3 {
      let [hours, minutes, seconds, a, b, c, d] = bytes else {
        return Err(
          DatabaseError::UnexpectedBufferSize {
            expected: 7,
            received: Usize::from(bytes.len()).into(),
          }
          .into(),
        );
      };
      (*hours, *minutes, *seconds, u32::from_be_bytes([*a, *b, *c, *d]))
    } else {
      let [hours, minutes, seconds] = bytes else {
        return Err(
          DatabaseError::UnexpectedBufferSize {
            expected: 3,
            received: Usize::from(bytes.len()).into(),
          }
          .into(),
        );
      };
      (*hours, *minutes, *seconds, 0)
    };
    NaiveTime::from_hms_micro_opt(hours.into(), minutes.into(), seconds.into(), micros).ok_or_else(
      || DatabaseError::UnexpectedValueFromBytes { expected: type_name::<NaiveTime>() }.into(),
    )
  }

  fn time_encode(
    time: &NaiveTime,
    include_micros: bool,
    ev: &mut EncodeWrapper<'_, '_>,
  ) -> crate::Result<()> {
    let hour = time.hour().try_into().unwrap_or_default();
    let minute = time.minute().try_into().unwrap_or_default();
    let second = time.second().try_into().unwrap_or_default();
    if include_micros {
      let [a, b, c, d] = (time.nanosecond() / 1000).to_le_bytes();
      ev.sw().extend_from_slice(&[hour, minute, second, a, b, c, d])?;
    } else {
      ev.sw().extend_from_slice(&[hour, minute, second])?;
    }
    Ok(())
  }

  test!(datetime_utc, DateTime<Utc>, Utc.from_utc_datetime(&pg_epoch_ndt().unwrap()));
}

mod collections {
  use crate::{
    database::{
      client::mysql::{DecodeWrapper, EncodeWrapper, Mysql, Ty},
      Typed,
    },
    misc::{from_utf8_basic, ArrayString, Decode, Encode},
  };
  use alloc::string::String;

  // &[u8]

  impl<'exec, E> Decode<'exec, Mysql<E>> for &'exec [u8]
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(dv: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
      Ok(dv.bytes())
    }
  }
  impl<E> Encode<Mysql<E>> for &[u8]
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, ev: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      ev.sw().extend_from_slice(self).map_err(Into::into)?;
      Ok(())
    }
  }
  impl<E> Typed<Mysql<E>> for &[u8]
  where
    E: From<crate::Error>,
  {
    const TY: Ty = Ty::Blob;
  }
  test!(bytes, &[u8], &[1, 2, 3, 4]);

  // String

  impl<E, const N: usize> Decode<'_, Mysql<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      Ok(from_utf8_basic(dv.bytes()).map_err(Into::into)?.try_into().map_err(Into::into)?)
    }
  }
  impl<E, const N: usize> Encode<Mysql<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, ev: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      ev.sw().extend_from_slice(self.as_str().as_bytes()).map_err(Into::into)?;
      Ok(())
    }
  }
  impl<E, const N: usize> Typed<Mysql<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    const TY: Ty = Ty::VarString;
  }
  test!(array_string, ArrayString<4>, ArrayString::try_from("123").unwrap());

  impl<'exec, E> Decode<'exec, Mysql<E>> for &'exec str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(dv: &mut DecodeWrapper<'exec>) -> Result<Self, E> {
      Ok(from_utf8_basic(dv.bytes()).map_err(crate::Error::from)?)
    }
  }
  impl<E> Encode<Mysql<E>> for &str
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, ev: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      ev.sw().extend_from_slice(self.as_bytes()).map_err(Into::into)?;
      Ok(())
    }
  }
  impl<E> Typed<Mysql<E>> for &str
  where
    E: From<crate::Error>,
  {
    const TY: Ty = Ty::VarString;
  }
  test!(str, &str, "1234");

  // String

  impl<E> Decode<'_, Mysql<E>> for String
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      match from_utf8_basic(dv.bytes()).map_err(crate::Error::from) {
        Ok(elem) => Ok(elem.into()),
        Err(err) => Err(err.into()),
      }
    }
  }
  impl<E> Encode<Mysql<E>> for String
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, ev: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      ev.sw().extend_from_slice(self.as_bytes()).map_err(Into::into)?;
      Ok(())
    }
  }
  impl<E> Typed<Mysql<E>> for String
  where
    E: From<crate::Error>,
  {
    const TY: Ty = Ty::VarString;
  }
  kani!(string, String);
}

mod primitives {
  use crate::{
    database::{
      client::mysql::{DecodeWrapper, EncodeWrapper, Mysql, Ty},
      DatabaseError, Typed,
    },
    misc::{Decode, Encode, Usize},
  };

  // bool

  impl<E> Decode<'_, Mysql<E>> for bool
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
      let &[byte] = dv.bytes() else {
        return Err(E::from(
          DatabaseError::UnexpectedBufferSize {
            expected: 1,
            received: Usize::from(dv.bytes().len()).into(),
          }
          .into(),
        ));
      };
      Ok(byte != 0)
    }
  }
  impl<E> Encode<Mysql<E>> for bool
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, ev: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
      ev.sw()._extend_from_byte((*self).into()).map_err(Into::into)?;
      Ok(())
    }
  }
  impl<E> Typed<Mysql<E>> for bool
  where
    E: From<crate::Error>,
  {
    const TY: Ty = Ty::Tiny;
  }

  macro_rules! impl_integer_from_array {
      ($instance:expr, [$($elem:ident),+], ($signed:ident, $signed_pg_ty:expr), ($unsigned:ident, $unsigned_pg_ty:expr)) => {
        impl_primitive_from_array!($instance, [$($elem),+], $signed, $signed_pg_ty);

        impl<E> Decode<'_, Mysql<E>> for $unsigned
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn decode(dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
            if let &[$($elem,)+] = dv.bytes() {
                return Ok(<Self>::from_be_bytes([$($elem),+]));
              }
              Err(E::from(DatabaseError::UnexpectedBufferSize {
                expected: Usize::from(size_of::<Self>()).into_u32().unwrap_or(u32::MAX),
                received: Usize::from(dv.bytes().len()).into()
              }.into()))
          }
        }
        impl<E> Encode<Mysql<E>> for $unsigned
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn encode(&self, ev: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
            ev.sw().extend_from_slice(&self.to_be_bytes()).map_err(Into::into)?;
            Ok(())
          }
        }
        impl<E> Typed<Mysql<E>> for $unsigned
        where
          E: From<crate::Error>
        {
          const TY: Ty = $unsigned_pg_ty;
        }

        test!($unsigned, $unsigned, $instance);
      };
    }

  macro_rules! impl_primitive_from_array {
      ($instance:expr, [$($elem:ident),+], $ty:ident, $pg_ty:expr) => {
        impl<E> Decode<'_, Mysql<E>> for $ty
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn decode(dv: &mut DecodeWrapper<'_>) -> Result<Self, E> {
            if let &[$($elem,)+] = dv.bytes() {
              return Ok(<Self>::from_be_bytes([$($elem),+]));
            }
            Err(E::from(DatabaseError::UnexpectedBufferSize {
              expected: Usize::from(size_of::<Self>()).into_u32().unwrap_or(u32::MAX),
              received: Usize::from(dv.bytes().len()).into()
            }.into()))
          }
        }

        impl<E> Encode<Mysql<E>> for $ty
        where
          E: From<crate::Error>,
        {
          #[inline]
          fn encode(&self, ev: &mut EncodeWrapper<'_, '_>) -> Result<(), E> {
            ev.sw().extend_from_slice(&self.to_be_bytes()).map_err(Into::into)?;
            Ok(())
          }
        }

        impl<E> Typed<Mysql<E>> for $ty
        where
          E: From<crate::Error>
        {
          const TY: Ty = $pg_ty;
        }

        test!($ty, $ty, $instance);
      }
    }

  impl_integer_from_array!(37, [a], (i8, Ty::Tiny), (u8, Ty::Tiny));
  impl_integer_from_array!(37, [a, b], (i16, Ty::Short), (u16, Ty::Short));
  impl_integer_from_array!(37, [a, b, c, d], (i32, Ty::Long), (u32, Ty::Long));
  impl_integer_from_array!(37, [a, b, c, d, e, f, g, h], (i64, Ty::LongLong), (u64, Ty::LongLong));

  impl_primitive_from_array!(37.0, [a, b, c, d], f32, Ty::Float);
  impl_primitive_from_array!(37.0, [a, b, c, d, e, f, g, h], f64, Ty::Double);
}
