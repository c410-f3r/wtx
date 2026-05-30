use crate::{
  collection::Clear,
  http::Headers,
  misc::{Lease, LeaseMut, Uri, UriRef},
};
use alloc::boxed::Box;

static EMPTY_URI_STRING: UriRef<'static> = UriRef::empty("");

/// An HTTP message data can refer a request or a response.
///
/// Groups the elements of an HTTP request/response.
pub trait MsgData {
  /// See [`Self::body`].
  type Body: ?Sized;

  /// Can be a sequence of bytes, a string, a deserialized element or any other desired type.
  fn body(&self) -> &Self::Body;

  /// See [Headers].
  fn headers(&self) -> &Headers;

  /// See [`UriRef<'_>`].
  fn uri(&self) -> UriRef<'_>;
}

impl<T> MsgData for &T
where
  T: MsgData,
{
  type Body = T::Body;

  #[inline]
  fn body(&self) -> &Self::Body {
    (*self).body()
  }

  #[inline]
  fn headers(&self) -> &Headers {
    (*self).headers()
  }

  #[inline]
  fn uri(&self) -> UriRef<'_> {
    (*self).uri()
  }
}

impl<T> MsgData for &mut T
where
  T: MsgData,
{
  type Body = T::Body;

  #[inline]
  fn body(&self) -> &Self::Body {
    (**self).body()
  }

  #[inline]
  fn headers(&self) -> &Headers {
    (**self).headers()
  }

  #[inline]
  fn uri(&self) -> UriRef<'_> {
    (**self).uri()
  }
}

impl MsgData for &[u8] {
  type Body = [u8];

  #[inline]
  fn body(&self) -> &Self::Body {
    self
  }

  #[inline]
  fn headers(&self) -> &Headers {
    const { &Headers::new() }
  }

  #[inline]
  fn uri(&self) -> UriRef<'_> {
    EMPTY_URI_STRING
  }
}

impl<const N: usize> MsgData for [u8; N] {
  type Body = [u8; N];

  #[inline]
  fn body(&self) -> &Self::Body {
    self
  }

  #[inline]
  fn headers(&self) -> &Headers {
    const { &Headers::new() }
  }

  #[inline]
  fn uri(&self) -> UriRef<'_> {
    EMPTY_URI_STRING
  }
}

impl MsgData for () {
  type Body = [u8];

  #[inline]
  fn body(&self) -> &Self::Body {
    &[]
  }

  #[inline]
  fn headers(&self) -> &Headers {
    const { &Headers::new() }
  }

  #[inline]
  fn uri(&self) -> UriRef<'_> {
    EMPTY_URI_STRING
  }
}

impl<B, H> MsgData for (B, H)
where
  H: Lease<Headers>,
{
  type Body = B;

  #[inline]
  fn body(&self) -> &Self::Body {
    &self.0
  }

  #[inline]
  fn headers(&self) -> &Headers {
    self.1.lease()
  }

  #[inline]
  fn uri(&self) -> UriRef<'_> {
    EMPTY_URI_STRING
  }
}

impl<B, H, S> MsgData for (B, H, Uri<S>)
where
  H: Lease<Headers>,
  S: Lease<str>,
{
  type Body = B;

  #[inline]
  fn body(&self) -> &Self::Body {
    &self.0
  }

  #[inline]
  fn headers(&self) -> &Headers {
    self.1.lease()
  }

  #[inline]
  fn uri(&self) -> UriRef<'_> {
    self.2.lease().to_ref()
  }
}

impl<T> MsgData for Box<T>
where
  T: MsgData,
{
  type Body = T::Body;

  #[inline]
  fn body(&self) -> &Self::Body {
    (**self).body()
  }

  #[inline]
  fn headers(&self) -> &Headers {
    (**self).headers()
  }

  #[inline]
  fn uri(&self) -> UriRef<'_> {
    (**self).uri()
  }
}

impl MsgData for Headers {
  type Body = [u8];

  #[inline]
  fn body(&self) -> &Self::Body {
    &[]
  }

  #[inline]
  fn headers(&self) -> &Headers {
    self
  }

  #[inline]
  fn uri(&self) -> UriRef<'_> {
    EMPTY_URI_STRING
  }
}

impl<S> MsgData for Uri<S>
where
  S: Lease<str>,
{
  type Body = [u8];

  #[inline]
  fn body(&self) -> &Self::Body {
    &[]
  }

  #[inline]
  fn headers(&self) -> &Headers {
    const { &Headers::new() }
  }

  #[inline]
  fn uri(&self) -> UriRef<'_> {
    self.to_ref()
  }
}

/// Mutable version of [`MsgData`].
pub trait MsgDataMut: MsgData {
  /// Can be a sequence of mutable bytes, a mutable string or any other desired type.
  #[inline]
  fn body_mut(&mut self) -> &mut Self::Body {
    self.parts_mut().0
  }

  /// Removes all values.
  fn clear(&mut self);

  /// Removes all but URI values.
  fn clear_body_and_headers(&mut self);

  /// Mutable version of [`MsgData::headers`].
  #[inline]
  fn headers_mut(&mut self) -> &mut Headers {
    self.parts_mut().1
  }

  /// Mutable parts
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, UriRef<'_>);
}

impl<T> MsgDataMut for &mut T
where
  T: MsgDataMut,
{
  #[inline]
  fn body_mut(&mut self) -> &mut Self::Body {
    (**self).body_mut()
  }

  #[inline]
  fn clear(&mut self) {
    (**self).clear();
  }

  #[inline]
  fn clear_body_and_headers(&mut self) {
    (**self).clear_body_and_headers();
  }

  #[inline]
  fn headers_mut(&mut self) -> &mut Headers {
    (**self).headers_mut()
  }

  #[inline]
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, UriRef<'_>) {
    (**self).parts_mut()
  }
}

impl<T> MsgDataMut for Box<T>
where
  T: MsgDataMut,
{
  #[inline]
  fn body_mut(&mut self) -> &mut Self::Body {
    (**self).body_mut()
  }

  #[inline]
  fn clear(&mut self) {
    (**self).clear();
  }

  #[inline]
  fn clear_body_and_headers(&mut self) {
    (**self).clear_body_and_headers();
  }

  #[inline]
  fn headers_mut(&mut self) -> &mut Headers {
    (**self).headers_mut()
  }

  #[inline]
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, UriRef<'_>) {
    (**self).parts_mut()
  }
}

impl MsgDataMut for Headers {
  #[inline]
  fn clear(&mut self) {
    self.headers_mut().clear();
  }

  #[inline]
  fn clear_body_and_headers(&mut self) {
    self.headers_mut().clear();
  }

  #[inline]
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, UriRef<'_>) {
    (&mut [], self, EMPTY_URI_STRING)
  }
}

impl<B, H> MsgDataMut for (B, H)
where
  B: Clear,
  H: LeaseMut<Headers>,
{
  #[inline]
  fn clear(&mut self) {
    self.body_mut().clear();
    self.headers_mut().clear();
  }

  #[inline]
  fn clear_body_and_headers(&mut self) {
    self.body_mut().clear();
    self.headers_mut().clear();
  }

  #[inline]
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, UriRef<'_>) {
    (&mut self.0, self.1.lease_mut(), EMPTY_URI_STRING)
  }
}

impl<B, H, S> MsgDataMut for (B, H, Uri<S>)
where
  B: Clear,
  H: LeaseMut<Headers>,
  S: Clear + Lease<str>,
{
  #[inline]
  fn clear(&mut self) {
    self.0.clear();
    self.1.lease_mut().clear();
    self.2.clear();
  }

  #[inline]
  fn clear_body_and_headers(&mut self) {
    self.0.clear();
    self.1.lease_mut().clear();
  }

  #[inline]
  fn parts_mut(&mut self) -> (&mut Self::Body, &mut Headers, UriRef<'_>) {
    (&mut self.0, self.1.lease_mut(), self.2.to_ref())
  }
}
