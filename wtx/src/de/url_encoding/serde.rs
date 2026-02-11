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
pub type FormUrlSerializer<'buffer> = UrlSerializer<'buffer, false, true>;
/// Type alias for percent encoding (spaces as `%20`).
pub type PercentSerializer<'buffer> = UrlSerializer<'buffer, true, true>;

/// Serializes data into a `Vector`.
#[derive(Debug)]
pub struct UrlSerializer<'buffer, const IS_PERCENT: bool, const IS_TOP_LEVEL: bool> {
  ascii_set: AsciiSet,
  buffer: &'buffer mut Vector<u8>,
}

impl<'buffer, const IS_PERCENT: bool, const IS_TOP_LEVEL: bool>
  UrlSerializer<'buffer, IS_PERCENT, IS_TOP_LEVEL>
{
  /// New instance
  ///
  /// Defaults to [`AsciiSet::UNRESERVED`] if `ascii_set` is `None`.
  pub fn new(ascii_set: Option<AsciiSet>, buffer: &'buffer mut Vector<u8>) -> Self {
    if IS_TOP_LEVEL {
      buffer.clear();
    }
    let actual = if let Some(elem) = ascii_set { elem } else { AsciiSet::UNRESERVED };
    Self { ascii_set: actual, buffer }
  }
}

impl<'buffer, const IS_PERCENT: bool, const IS_TOP_LEVEL: bool> ser::Serializer
  for UrlSerializer<'buffer, IS_PERCENT, IS_TOP_LEVEL>
{
  type Error = crate::Error;
  type Ok = &'buffer str;
  type SerializeMap = MapSerializer<'buffer, IS_PERCENT>;
  type SerializeSeq = SeqSerializer<'buffer, IS_PERCENT>;
  type SerializeStruct = StructSerializer<'buffer, IS_PERCENT>;
  type SerializeStructVariant = ser::Impossible<&'buffer str, Self::Error>;
  type SerializeTuple = SeqSerializer<'buffer, IS_PERCENT>;
  type SerializeTupleStruct = SeqSerializer<'buffer, IS_PERCENT>;
  type SerializeTupleVariant = TupleVariantSerializer<'buffer, IS_PERCENT>;

  fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
    self.serialize_str(if v { "true" } else { "false" })
  }

  fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
    for chunk in UrlEncode::<IS_PERCENT>::new(v, self.ascii_set) {
      self.buffer.extend_from_copyable_slice(chunk)?;
    }
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
    self.serialize_str(v.encode_utf8(&mut [0u8; 4]))
  }

  fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
    write!(
      UrlEncodeWriter::<IS_PERCENT> { ascii_set: self.ascii_set, buffer: self.buffer },
      "{v}"
    )?;
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
    write!(
      UrlEncodeWriter::<IS_PERCENT> { ascii_set: self.ascii_set, buffer: self.buffer },
      "{v}"
    )?;
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(i8_string(v).as_bytes())?;
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(i16_string(v).as_bytes())?;
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(i32_string(v).as_bytes())?;
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(i64_string(v).as_bytes())?;
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_newtype_struct<T: ?Sized>(
    self,
    _: &'static str,
    value: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: ser::Serialize,
  {
    value.serialize(self)
  }

  fn serialize_newtype_variant<T: ?Sized>(
    self,
    _: &'static str,
    _: u32,
    variant: &'static str,
    value: &T,
  ) -> Result<Self::Ok, Self::Error>
  where
    T: ser::Serialize,
  {
    for chunk in UrlEncode::<IS_PERCENT>::new(variant.as_bytes(), self.ascii_set) {
      self.buffer.extend_from_copyable_slice(chunk)?;
    }
    self.buffer.extend_from_copyable_slice(b"=")?;
    let _ = value
      .serialize(UrlSerializer::<IS_PERCENT, false>::new(Some(self.ascii_set), self.buffer))?;
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
    if !IS_TOP_LEVEL {
      return Err(crate::Error::UnsupportedOperation);
    }
    Ok(MapSerializer { ascii_set: self.ascii_set, buffer: self.buffer, is_first: true })
  }

  fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
    if IS_TOP_LEVEL {
      return Err(crate::Error::UnsupportedOperation);
    }
    Ok(SeqSerializer { ascii_set: self.ascii_set, buffer: self.buffer, is_first: true })
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
    if !IS_TOP_LEVEL {
      return Err(crate::Error::UnsupportedOperation);
    }
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
    if IS_TOP_LEVEL {
      return Err(crate::Error::UnsupportedOperation);
    }
    Ok(SeqSerializer { ascii_set: self.ascii_set, buffer: self.buffer, is_first: true })
  }

  fn serialize_tuple_struct(
    self,
    _: &'static str,
    _: usize,
  ) -> Result<Self::SerializeTupleStruct, Self::Error> {
    if IS_TOP_LEVEL {
      return Err(crate::Error::UnsupportedOperation);
    }
    Ok(SeqSerializer { ascii_set: self.ascii_set, buffer: self.buffer, is_first: true })
  }

  fn serialize_tuple_variant(
    self,
    _: &'static str,
    _: u32,
    variant: &'static str,
    _: usize,
  ) -> Result<Self::SerializeTupleVariant, Self::Error> {
    if !IS_TOP_LEVEL {
      return Err(crate::Error::UnsupportedOperation);
    }
    for chunk in UrlEncode::<IS_PERCENT>::new(variant.as_bytes(), self.ascii_set) {
      self.buffer.extend_from_copyable_slice(chunk)?;
    }
    self.buffer.extend_from_copyable_slice(b"=")?;
    Ok(TupleVariantSerializer { ascii_set: self.ascii_set, buffer: self.buffer, is_first: true })
  }

  fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(u8_string(v).as_bytes())?;
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(u16_string(v).as_bytes())?;
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(u32_string(v).as_bytes())?;
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
    self.buffer.extend_from_copyable_slice(u64_string(v).as_bytes())?;
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_unit_variant(
    self,
    _: &'static str,
    _: u32,
    variant: &'static str,
  ) -> Result<Self::Ok, Self::Error> {
    if IS_TOP_LEVEL {
      return Err(crate::Error::UnsupportedOperation);
    }
    self.serialize_str(variant)
  }
}

/// Map serializer
#[derive(Debug)]
pub struct MapSerializer<'buffer, const IS_PERCENT: bool> {
  ascii_set: AsciiSet,
  buffer: &'buffer mut Vector<u8>,
  is_first: bool,
}

impl<'buffer, const IS_PERCENT: bool> ser::SerializeMap for MapSerializer<'buffer, IS_PERCENT> {
  type Error = crate::Error;
  type Ok = &'buffer str;

  fn end(self) -> crate::Result<Self::Ok> {
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_key<T>(&mut self, key: &T) -> crate::Result<()>
  where
    T: ser::Serialize + ?Sized,
  {
    if !self.is_first {
      self.buffer.extend_from_copyable_slice(b"&")?;
    }
    self.is_first = false;
    let _ =
      key.serialize(UrlSerializer::<IS_PERCENT, false>::new(Some(self.ascii_set), self.buffer))?;
    self.buffer.extend_from_copyable_slice(b"=")?;
    Ok(())
  }

  fn serialize_value<T>(&mut self, value: &T) -> crate::Result<()>
  where
    T: ser::Serialize + ?Sized,
  {
    let _ = value
      .serialize(UrlSerializer::<IS_PERCENT, false>::new(Some(self.ascii_set), self.buffer))?;
    Ok(())
  }
}

/// Sequence serializer
#[derive(Debug)]
pub struct SeqSerializer<'buffer, const IS_PERCENT: bool> {
  ascii_set: AsciiSet,
  buffer: &'buffer mut Vector<u8>,
  is_first: bool,
}

impl<'buffer, const IS_PERCENT: bool> ser::SerializeSeq for SeqSerializer<'buffer, IS_PERCENT> {
  type Error = crate::Error;
  type Ok = &'buffer str;

  fn end(self) -> crate::Result<Self::Ok> {
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_element<T>(&mut self, value: &T) -> crate::Result<()>
  where
    T: ser::Serialize + ?Sized,
  {
    if !self.is_first {
      self.buffer.extend_from_copyable_slice(b",")?;
    }
    self.is_first = false;
    let _ = value
      .serialize(UrlSerializer::<IS_PERCENT, false>::new(Some(self.ascii_set), self.buffer))?;
    Ok(())
  }
}

impl<'buffer, const IS_PERCENT: bool> ser::SerializeTuple for SeqSerializer<'buffer, IS_PERCENT> {
  type Error = crate::Error;
  type Ok = &'buffer str;

  fn end(self) -> crate::Result<Self::Ok> {
    ser::SerializeSeq::end(self)
  }

  fn serialize_element<T>(&mut self, value: &T) -> crate::Result<()>
  where
    T: ser::Serialize + ?Sized,
  {
    ser::SerializeSeq::serialize_element(self, value)
  }
}

impl<'buffer, const IS_PERCENT: bool> ser::SerializeTupleStruct
  for SeqSerializer<'buffer, IS_PERCENT>
{
  type Error = crate::Error;
  type Ok = &'buffer str;

  fn end(self) -> crate::Result<Self::Ok> {
    ser::SerializeSeq::end(self)
  }

  fn serialize_field<T>(&mut self, value: &T) -> crate::Result<()>
  where
    T: ser::Serialize + ?Sized,
  {
    ser::SerializeSeq::serialize_element(self, value)
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
  type Ok = &'buffer str;

  fn end(self) -> crate::Result<Self::Ok> {
    Ok(url_encode_str(self.buffer))
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
    let _ = value
      .serialize(UrlSerializer::<IS_PERCENT, false>::new(Some(self.ascii_set), self.buffer))?;
    Ok(())
  }
}

/// Tuple variant serializer
#[derive(Debug)]
pub struct TupleVariantSerializer<'buffer, const IS_PERCENT: bool> {
  ascii_set: AsciiSet,
  buffer: &'buffer mut Vector<u8>,
  is_first: bool,
}

impl<'buffer, const IS_PERCENT: bool> ser::SerializeTupleVariant
  for TupleVariantSerializer<'buffer, IS_PERCENT>
{
  type Error = crate::Error;
  type Ok = &'buffer str;

  fn end(self) -> crate::Result<Self::Ok> {
    Ok(url_encode_str(self.buffer))
  }

  fn serialize_field<T>(&mut self, value: &T) -> crate::Result<()>
  where
    T: ser::Serialize + ?Sized,
  {
    if !self.is_first {
      self.buffer.extend_from_copyable_slice(b",")?;
    }
    self.is_first = false;
    let _ = value
      .serialize(UrlSerializer::<IS_PERCENT, false>::new(Some(self.ascii_set), self.buffer))?;
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

fn url_encode_str(bytes: &[u8]) -> &str {
  // SAFETY: Url encoding is ASCII
  unsafe { str::from_utf8_unchecked(bytes) }
}

#[cfg(test)]
mod tests {
  use crate::{collection::Vector, de::FormUrlSerializer};
  use serde::Serialize;

  #[test]
  fn bool() {
    #[derive(serde::Serialize)]
    struct Foo {
      yes: bool,
      no: bool,
    }

    let mut buffer = Vector::new();
    let serializer = FormUrlSerializer::new(None, &mut buffer);
    let _ = Foo { yes: true, no: false }.serialize(serializer).unwrap();
    assert_eq!(&buffer, b"yes=true&no=false");
  }

  #[test]
  fn newtype_struct() {
    #[derive(serde::Serialize)]
    struct Foo(i32);
    #[derive(serde::Serialize)]
    struct FooTy {
      value: Foo,
    }

    let mut buffer = Vector::new();
    let serializer = FormUrlSerializer::new(None, &mut buffer);
    let _ = FooTy { value: Foo(42) }.serialize(serializer).unwrap();
    assert_eq!(&buffer, b"value=42");
  }

  #[test]
  fn newtype_variant() {
    #[derive(serde::Serialize)]
    enum Value {
      Count(u32),
    }
    #[derive(serde::Serialize)]
    struct Foo {
      value: Value,
    }

    let mut buffer = Vector::new();
    let serializer = FormUrlSerializer::new(None, &mut buffer);
    let _ = Foo { value: Value::Count(42) }.serialize(serializer).unwrap();
    assert_eq!(&buffer, b"value=Count=42");
  }

  #[test]
  fn option() {
    #[derive(serde::Serialize)]
    struct Foo {
      present: Option<i32>,
      absent: Option<i32>,
    }

    let mut buffer = Vector::new();
    let serializer = FormUrlSerializer::new(None, &mut buffer);
    let _ = Foo { present: Some(5), absent: None }.serialize(serializer).unwrap();
    assert_eq!(&buffer, b"present=5&absent=");
  }

  #[test]
  fn seq_field() {
    #[derive(serde::Serialize)]
    struct Foo {
      items: [i32; 3],
    }

    let mut buffer = Vector::new();
    let serializer = FormUrlSerializer::new(None, &mut buffer);
    let _ = Foo { items: [1, 2, 3] }.serialize(serializer).unwrap();
    assert_eq!(&buffer, b"items=1,2,3");
  }

  #[test]
  fn structure() {
    #[derive(serde::Serialize)]
    struct Foo {
      bar: i32,
      baz: &'static str,
    }

    let mut buffer = Vector::new();
    let serializer = FormUrlSerializer::new(None, &mut buffer);
    let _ = Foo { bar: 123, baz: "hello there!" }.serialize(serializer).unwrap();
    assert_eq!(&buffer, b"bar=123&baz=hello+there%21");
  }

  #[test]
  fn tuple_struct() {
    #[derive(serde::Serialize)]
    struct Foo(i32, &'static str);
    #[derive(serde::Serialize)]
    struct Bar {
      data: Foo,
    }

    let mut buffer = Vector::new();
    let serializer = FormUrlSerializer::new(None, &mut buffer);
    let _ = Bar { data: Foo(1, "a") }.serialize(serializer).unwrap();
    assert_eq!(&buffer, b"data=1,a");
  }

  #[test]
  fn tuple_variant() {
    #[derive(serde::Serialize)]
    enum Coords {
      Point(i32, i32),
    }

    let mut buffer = Vector::new();
    let serializer = FormUrlSerializer::new(None, &mut buffer);
    let _ = Coords::Point(10, 20).serialize(serializer).unwrap();
    assert_eq!(&buffer, b"Point=10,20");
  }

  #[test]
  fn unit_variant() {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "lowercase")]
    enum Status {
      Absent,
      Active,
    }
    #[derive(serde::Serialize)]
    struct Foo {
      status: Status,
    }

    let mut buffer = Vector::new();
    let _ =
      Foo { status: Status::Absent }.serialize(FormUrlSerializer::new(None, &mut buffer)).unwrap();
    assert_eq!(&buffer, b"status=absent");
    buffer.clear();
    let _ =
      Foo { status: Status::Active }.serialize(FormUrlSerializer::new(None, &mut buffer)).unwrap();
    assert_eq!(&buffer, b"status=active");
  }
}
