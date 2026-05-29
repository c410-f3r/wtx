mod req_builder;
mod res_builder;

use crate::{
  codec::u32_string,
  collection::Vector,
  http::{Header, KnownHeaderName, Mime, ReqResDataMut},
  misc::{Either, LeaseMut},
};
use core::fmt::Arguments;
pub use req_builder::ReqBuilder;
pub use res_builder::ResBuilder;

/// A tailored [`Either`].
#[derive(Debug)]
pub struct ReqResBuilderInput<'left, 'right>(Either<&'left [&'left str], Arguments<'right>>);

impl From<()> for ReqResBuilderInput<'_, '_> {
  #[inline]
  fn from(_: ()) -> Self {
    Self(Either::Left(&[]))
  }
}

impl<'left, const N: usize> From<&'left [&'left str; N]> for ReqResBuilderInput<'left, '_> {
  #[inline]
  fn from(value: &'left [&'left str; N]) -> Self {
    Self(Either::Left(value))
  }
}

impl<'left> From<&'left [&'left str]> for ReqResBuilderInput<'left, '_> {
  #[inline]
  fn from(value: &'left [&'left str]) -> Self {
    Self(Either::Left(value))
  }
}

impl<'right> From<Arguments<'right>> for ReqResBuilderInput<'_, 'right> {
  #[inline]
  fn from(value: Arguments<'right>) -> Self {
    Self(Either::Right(value))
  }
}

/// Request/Response Builder
#[derive(Debug)]
pub struct ReqResBuilder<RRD> {
  rrd: RRD,
}

impl<RRD> ReqResBuilder<RRD> {
  /// Constructor shortcut
  #[inline]
  pub const fn new(rrd: RRD) -> Self {
    Self { rrd }
  }

  /// Owned version of [`Self::rrd`].
  #[inline]
  pub fn into_rrd(self) -> RRD {
    self.rrd
  }

  /// Request/Response Data
  #[inline]
  pub const fn rrd(&self) -> &RRD {
    &self.rrd
  }

  /// Mutable version of [`Self::rrd`].
  #[inline]
  pub const fn rrd_mut(&mut self) -> &mut RRD {
    &mut self.rrd
  }
}

impl<RRD> ReqResBuilder<RRD>
where
  RRD: ReqResDataMut,
{
  /// Which formats the request should receive.
  #[inline]
  pub fn accept<'left, 'right, I>(&mut self, value: I) -> crate::Result<&mut Self>
  where
    I: Into<ReqResBuilderInput<'left, 'right>>,
  {
    self.push_header(KnownHeaderName::Accept.into(), value)?;
    Ok(self)
  }

  /// Applies a header field in the form of `Authorization: Bearer <token>`.
  #[inline]
  pub fn auth_bearer<'left, 'right, I>(&mut self, value: I) -> crate::Result<&mut Self>
  where
    I: Into<ReqResBuilderInput<'left, 'right>>,
  {
    let headers = self.rrd.headers_mut();
    let name = KnownHeaderName::Authorization.into();
    match value.into().0 {
      Either::Left(el) => {
        headers.push_from_iter(Header::from_name_and_value(
          name,
          ["Bearer "].into_iter().chain(el.iter().copied()),
        ))?;
      }
      Either::Right(el) => {
        headers.push_from_fmt(Header::from_name_and_value(name, format_args!("Bearer {el}")))?;
      }
    }
    Ok(self)
  }

  /// Body request/response length
  #[inline]
  pub fn content_length(&mut self, value: u32) -> crate::Result<&mut Self> {
    self.rrd.headers_mut().push_from_iter(Header::from_name_and_value(
      KnownHeaderName::ContentLength.into(),
      [u32_string(value).as_str()],
    ))?;
    Ok(self)
  }

  /// Media type of the resource.
  #[inline]
  pub fn content_type(&mut self, value: Mime) -> crate::Result<&mut Self> {
    self.rrd.headers_mut().push_from_iter(Header::from_name_and_value(
      KnownHeaderName::ContentType.into(),
      [value.as_str()],
    ))?;
    Ok(self)
  }

  /// Adds a header with a value composed by an iterator.
  #[inline]
  pub fn header_iter<'kv, I>(&mut self, name: &'kv str, value: I) -> crate::Result<&mut Self>
  where
    I: IntoIterator<Item = &'kv str>,
    I::IntoIter: Clone,
  {
    self.rrd.headers_mut().push_from_iter(Header::from_name_and_value(name, value))?;
    Ok(self)
  }

  /// Adds a header with a value composed by [`Arguments`].
  #[inline]
  pub fn header_fmt(&mut self, name: &str, value: Arguments<'_>) -> crate::Result<&mut Self> {
    self.rrd.headers_mut().push_from_fmt(Header::from_name_and_value(name, value))?;
    Ok(self)
  }

  /// The host and port number of the server to which the request is being sent.
  ///
  /// Uses the underlying URI if `value` is `None`.
  #[inline]
  pub fn host<'left, 'right, I>(&mut self, value: Option<I>) -> crate::Result<&mut Self>
  where
    I: Into<ReqResBuilderInput<'left, 'right>>,
  {
    let (_, headers, uri) = self.rrd.parts_mut();
    let name = KnownHeaderName::Host;
    if let Some(elem) = value {
      self.push_header(name.into(), elem)?;
    } else {
      headers.push_from_iter(Header::from_name_and_value(name.into(), [uri.host()]))?;
    }
    Ok(self)
  }

  /// Characteristic string that lets servers and network peers identify the application.
  #[inline]
  pub fn user_agent<'left, 'right, I>(&mut self, value: I) -> crate::Result<&mut Self>
  where
    I: Into<ReqResBuilderInput<'left, 'right>>,
  {
    self.push_header(KnownHeaderName::UserAgent.into(), value)?;
    Ok(self)
  }

  fn push_header<'left, 'right, I>(&mut self, name: &str, value: I) -> crate::Result<()>
  where
    I: Into<ReqResBuilderInput<'left, 'right>>,
  {
    let headers = self.rrd.headers_mut();
    match value.into().0 {
      Either::Left(el) => {
        headers.push_from_iter(Header::from_name_and_value(name, el.iter().copied()))?;
      }
      Either::Right(el) => {
        headers.push_from_fmt(Header::from_name_and_value(name, el))?;
      }
    }
    Ok(())
  }
}

impl<RRD> ReqResBuilder<RRD>
where
  RRD: ReqResDataMut,
  RRD::Body: LeaseMut<Vector<u8>>,
{
  /// Injects a sequence of bytes into the internal buffer.
  ///
  /// `content-length` is applied without `content-type`.
  #[inline]
  pub fn bytes(&mut self, data: &[u8]) -> crate::Result<&mut Self> {
    let before = self.rrd.body_mut().lease_mut().len();
    self.rrd.body_mut().lease_mut().extend_from_copyable_slice(data)?;
    let length = self.rrd.body_mut().lease_mut().len().wrapping_sub(before);
    self.content_length(length.try_into()?)
  }

  /// Injects a sequence of bytes into the internal buffer.
  ///
  /// `content-length` and a `content-type` header of type `text/html` is also applied.
  #[inline]
  pub fn html(&mut self, data: &[u8]) -> crate::Result<&mut Self> {
    let before = self.rrd.body_mut().lease_mut().len();
    self.rrd.body_mut().lease_mut().extend_from_copyable_slice(data)?;
    let length = self.rrd.body_mut().lease_mut().len().wrapping_sub(before);
    self.content_type(Mime::TextHtml)?.content_length(length.try_into()?)
  }

  /// Uses `serde_json` to inject a raw structure as JSON into the internal buffer.
  ///
  /// `content-length` and a `content-type` header of type `application/json` is also applied.
  #[cfg(feature = "serde_json")]
  #[inline]
  pub fn serde_json<T>(&mut self, data: &T) -> crate::Result<&mut Self>
  where
    T: serde::Serialize,
  {
    let before = self.rrd.body_mut().lease_mut().len();
    serde_json::to_writer(self.rrd.body_mut().lease_mut(), data)?;
    let length = self.rrd.body_mut().lease_mut().len().wrapping_sub(before);
    self.content_type(Mime::ApplicationJson)?.content_length(length.try_into()?)
  }

  /// Injects a sequence of bytes into the internal buffer.
  ///
  /// `content-length` and a `content-type` header of type `text/plain` is also applied.
  #[inline]
  pub fn text(&mut self, data: &[u8]) -> crate::Result<&mut Self> {
    let before = self.rrd.body_mut().lease_mut().len();
    self.rrd.body_mut().lease_mut().extend_from_copyable_slice(data)?;
    let length = self.rrd.body_mut().lease_mut().len().wrapping_sub(before);
    self.content_type(Mime::TextPlain)?.content_length(length.try_into()?)
  }
}
