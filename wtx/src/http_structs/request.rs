use crate::http_structs::{Header, HeaderSlice, ParseStatus};

/// Raw request that can be converted to other high-level requests.
#[derive(Debug)]
pub struct Request<'buffer, 'headers> {
  req: httparse::Request<'headers, 'buffer>,
}

impl<'buffer, 'headers> Request<'buffer, 'headers> {
  pub(crate) fn new(headers: &'headers mut [Header<'buffer>]) -> Self {
    Self { req: httparse::Request::new(HeaderSlice::from(headers).0) }
  }

  /// Method
  #[inline]
  pub fn method(&self) -> Option<&'buffer str> {
    self.req.method
  }

  /// Version
  #[inline]
  pub fn version(&self) -> Option<u8> {
    self.req.version
  }

  pub(crate) fn headers(&self) -> &[Header<'buffer>] {
    HeaderSlice::from(&*self.req.headers).0
  }

  pub(crate) fn parse(&mut self, buffer: &'buffer [u8]) -> crate::Result<ParseStatus> {
    Ok(self.req.parse(buffer)?.into())
  }
}

#[cfg(feature = "http")]
mod http {
  use crate::http_structs::Request;
  use http::{HeaderMap, HeaderName, HeaderValue, Method};

  impl<'buffer, 'headers> TryFrom<Request<'buffer, 'headers>> for http::Request<&'buffer [u8]> {
    type Error = crate::Error;

    #[inline]
    fn try_from(from: Request<'buffer, 'headers>) -> Result<Self, Self::Error> {
      let method =
        Method::try_from(from.req.method.ok_or(crate::Error::UnexpectedHttpVersion)?).unwrap();
      let version = if let Some(1) = from.req.version {
        http::Version::HTTP_11
      } else {
        return Err(crate::Error::UnexpectedHttpVersion);
      };
      let mut headers = HeaderMap::with_capacity(from.req.headers.len());
      for h in from.req.headers {
        let key = HeaderName::from_bytes(h.name.as_bytes())?;
        let value = HeaderValue::from_bytes(h.value)?;
        let _ = headers.append(key, value);
      }
      let mut req = http::Request::new(&[][..]);
      *req.headers_mut() = headers;
      *req.method_mut() = method;
      *req.uri_mut() = from.req.path.unwrap().parse().unwrap();
      *req.version_mut() = version;
      Ok(req)
    }
  }

  impl<'buffer, 'headers> TryFrom<Request<'buffer, 'headers>> for http::Request<()> {
    type Error = crate::Error;

    #[inline]
    fn try_from(from: Request<'buffer, 'headers>) -> Result<Self, Self::Error> {
      let (parts, _) = http::Request::<&'buffer [u8]>::try_from(from)?.into_parts();
      Ok(http::Request::from_parts(parts, ()))
    }
  }

  impl<'buffer, 'headers> TryFrom<Request<'buffer, 'headers>> for http::Request<Vec<u8>> {
    type Error = crate::Error;

    #[inline]
    fn try_from(from: Request<'buffer, 'headers>) -> Result<Self, Self::Error> {
      let (parts, body) = http::Request::<&'buffer [u8]>::try_from(from)?.into_parts();
      Ok(http::Request::from_parts(parts, body.to_vec()))
    }
  }
}
