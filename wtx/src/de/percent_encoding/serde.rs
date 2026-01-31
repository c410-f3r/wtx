use crate::{
  collection::Vector,
  de::{
    AsciiSet, UrlEncode, i8_string, i16_string, i32_string, i64_string, u8_string, u16_string,
    u32_string, u64_string,
  },
};
use core::fmt::Write;
use serde::ser;

/// Type alias for `application/x-www-form-urlencoded` (spaces as `+`).
pub type FormUrlSerializer<'buffer> = UrlSerializer<'buffer, false>;
/// Type alias for percent encoding (spaces as `%20`).
pub type PercentSerializer<'buffer> = UrlSerializer<'buffer, true>;

/// Serializes data into a `Vector`.
#[derive(Debug)]
pub struct UrlSerializer<'buffer, const IS_PERCENT: bool> {
  ascii_set: AsciiSet,
  buffer: &'buffer mut Vector<u8>,
}

impl<'buffer, const IS_PERCENT: bool> UrlSerializer<'buffer, IS_PERCENT> {
  /// New instance
  pub const fn new(ascii_set: AsciiSet, buffer: &'buffer mut Vector<u8>) -> Self {
    Self { ascii_set, buffer }
  }
}

impl<'buffer, const IS_PERCENT: bool> ser::Serializer for UrlSerializer<'buffer, IS_PERCENT> {
  type Error = crate::Error;
  type Ok = ();
  type SerializeMap = ser::Impossible<(), Self::Error>;
  type SerializeSeq = ser::Impossible<(), Self::Error>;
  type SerializeStruct = StructSerializer<'buffer, IS_PERCENT>;
  type SerializeStructVariant = ser::Impossible<(), Self::Error>;
  type SerializeTuple = ser::Impossible<(), Self::Error>;
  type SerializeTupleStruct = ser::Impossible<(), Self::Error>;
  type SerializeTupleVariant = ser::Impossible<(), Self::Error>;

  fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
    self.serialize_str(if v { "true" } else { "false" })
  }

  fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
    for chunk in UrlEncode::<IS_PERCENT>::new(v, self.ascii_set) {
      self.buffer.extend_from_copyable_slice(chunk)?;
    }
    Ok(())
  }

  fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
    self.serialize_str(v.encode_utf8(&mut [0u8; 4]))
  }

  fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
    write!(
      UrlEncodeWriter::<IS_PERCENT> { ascii_set: self.ascii_set, buffer: self.buffer },
      "{v}"
    )?;
    Ok(())
  }

  fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
    write!(
      UrlEncodeWriter::<IS_PERCENT> { ascii_set: self.ascii_set, buffer: self.buffer },
      "{v}"
    )?;
    Ok(())
  }

  fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(i8_string(v).as_bytes())?;
    Ok(())
  }

  fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(i16_string(v).as_bytes())?;
    Ok(())
  }

  fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(i32_string(v).as_bytes())?;
    Ok(())
  }

  fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(i64_string(v).as_bytes())?;
    Ok(())
  }

  fn serialize_newtype_struct<T: ?Sized>(
    self,
    _: &'static str,
    _: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: ser::Serialize,
  {
    Err(crate::Error::UnsupportedOperation)
  }

  fn serialize_newtype_variant<T: ?Sized>(
    self,
    _: &'static str,
    _: u32,
    _: &'static str,
    _: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: ser::Serialize,
  {
    Err(crate::Error::UnsupportedOperation)
  }

  fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
    Err(crate::Error::UnsupportedOperation)
  }

  fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
    Ok(())
  }

  fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
    Err(crate::Error::UnsupportedOperation)
  }

  fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
  where
    T: ser::Serialize,
  {
    value.serialize(self)
  }

  fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
    self.serialize_bytes(v.as_bytes())
  }

  fn serialize_struct(
    self,
    _: &'static str,
    _: usize,
  ) -> Result<Self::SerializeStruct, Self::Error> {
    Ok(StructSerializer { ascii_set: self.ascii_set, buffer: self.buffer, is_first: true })
  }

  fn serialize_struct_variant(
    self,
    _: &'static str,
    _: u32,
    _: &'static str,
    _: usize,
  ) -> Result<Self::SerializeStructVariant, Self::Error> {
    Err(crate::Error::UnsupportedOperation)
  }

  fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
    Err(crate::Error::UnsupportedOperation)
  }

  fn serialize_tuple_struct(
    self,
    _: &'static str,
    _: usize,
  ) -> Result<Self::SerializeTupleStruct, Self::Error> {
    Err(crate::Error::UnsupportedOperation)
  }

  fn serialize_tuple_variant(
    self,
    _: &'static str,
    _: u32,
    _: &'static str,
    _: usize,
  ) -> Result<Self::SerializeTupleVariant, Self::Error> {
    Err(crate::Error::UnsupportedOperation)
  }

  fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(u8_string(v).as_bytes())?;
    Ok(())
  }

  fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(u16_string(v).as_bytes())?;
    Ok(())
  }

  fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(u32_string(v).as_bytes())?;
    Ok(())
  }

  fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(u64_string(v).as_bytes())?;
    Ok(())
  }

  fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
    Ok(())
  }

  fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
    Ok(())
  }

  fn serialize_unit_variant(
    self,
    _: &'static str,
    _: u32,
    variant: &'static str,
  ) -> Result<Self::Ok, Self::Error> {
    self.serialize_str(variant)
  }
}

/// Struct serializer
#[derive(Debug)]
pub struct StructSerializer<'buffer, const IS_PERCENT: bool> {
  ascii_set: AsciiSet,
  buffer: &'buffer mut Vector<u8>,
  is_first: bool,
}

impl<'buffer, const IS_PERCENT: bool> ser::SerializeStruct
  for StructSerializer<'buffer, IS_PERCENT>
{
  type Error = crate::Error;
  type Ok = ();

  fn end(self) -> crate::Result<Self::Ok> {
    Ok(())
  }

  fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> crate::Result<()>
  where
    T: ser::Serialize + ?Sized,
  {
    if !self.is_first {
      self.buffer.extend_from_copyable_slice(b"&")?;
    }
    self.is_first = false;
    for chunk in UrlEncode::<IS_PERCENT>::new(key.as_bytes(), self.ascii_set) {
      self.buffer.extend_from_copyable_slice(chunk)?;
    }
    self.buffer.extend_from_copyable_slice(b"=")?;
    value.serialize(UrlSerializer::<IS_PERCENT>::new(self.ascii_set, self.buffer))?;
    Ok(())
  }
}

struct UrlEncodeWriter<'buffer, const IS_PERCENT: bool> {
  ascii_set: AsciiSet,
  buffer: &'buffer mut Vector<u8>,
}

impl<const IS_PERCENT: bool> Write for UrlEncodeWriter<'_, IS_PERCENT> {
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    for chunk in UrlEncode::<IS_PERCENT>::new(s.as_bytes(), self.ascii_set) {
      self.buffer.extend_from_copyable_slice(chunk).map_err(|_| core::fmt::Error)?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    collection::Vector,
    de::{AsciiSet, FormUrlSerializer},
  };
  use serde::Serialize;

  #[derive(serde::Serialize)]
  struct Foo {
    bar: i32,
    baz: &'static str,
  }

  #[test]
  fn serialize() {
    let mut buffer = Vector::new();
    let serializer = FormUrlSerializer::new(AsciiSet::NON_ALPHANUMERIC, &mut buffer);
    Foo { bar: 123, baz: "hello there!" }.serialize(serializer).unwrap();
    assert_eq!(&buffer, b"bar=123&baz=hello+there%21");
  }
}
