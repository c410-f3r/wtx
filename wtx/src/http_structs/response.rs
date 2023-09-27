use crate::http_structs::{Header, HeaderSlice, ParseStatus};

/// Raw response that can be converted to other high-level responses.
#[derive(Debug)]
pub struct Response<'buffer, 'headers> {
  res: httparse::Response<'headers, 'buffer>,
}

impl<'buffer, 'headers> Response<'buffer, 'headers>
where
  'buffer: 'headers,
{
  pub(crate) fn new(headers: &'headers mut [Header<'buffer>]) -> Self {
    Self { res: httparse::Response::new(HeaderSlice::from(headers).0) }
  }

  /// Status code
  #[inline]
  pub fn code(&self) -> Option<u16> {
    self.res.code
  }

  #[inline]
  pub(crate) fn code_mut(&mut self) -> &mut Option<u16> {
    &mut self.res.code
  }

  pub(crate) fn headers(&self) -> &[Header<'buffer>] {
    HeaderSlice::from(&*self.res.headers).0
  }

  pub(crate) fn parse(&mut self, buffer: &'buffer [u8]) -> crate::Result<ParseStatus> {
    Ok(self.res.parse(buffer)?.into())
  }

  pub(crate) fn version_mut(&mut self) -> &mut Option<u8> {
    &mut self.res.version
  }
}

#[cfg(feature = "http")]
mod http {
  use crate::http_structs::Response;
  use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};

  impl<'buffer, 'headers> TryFrom<Response<'buffer, 'headers>> for http::Response<&'buffer [u8]> {
    type Error = crate::Error;

    #[inline]
    fn try_from(from: Response<'buffer, 'headers>) -> Result<Self, Self::Error> {
      let status = StatusCode::from_u16(from.res.code.ok_or(crate::Error::UnexpectedHttpVersion)?)?;
      let version = if let Some(1) = from.res.version {
        http::Version::HTTP_11
      } else {
        return Err(crate::Error::UnexpectedHttpVersion);
      };
      let mut headers = HeaderMap::with_capacity(from.res.headers.len());
      for h in from.res.headers {
        let key = HeaderName::from_bytes(h.name.as_bytes())?;
        let value = HeaderValue::from_bytes(h.value)?;
        let _ = headers.append(key, value);
      }
      let mut res = http::Response::new(&[][..]);
      *res.headers_mut() = headers;
      *res.status_mut() = status;
      *res.version_mut() = version;
      Ok(res)
    }
  }

  impl<'buffer, 'headers> TryFrom<Response<'buffer, 'headers>> for http::Response<()> {
    type Error = crate::Error;

    #[inline]
    fn try_from(from: Response<'buffer, 'headers>) -> Result<Self, Self::Error> {
      let (parts, _) = http::Response::<&'buffer [u8]>::try_from(from)?.into_parts();
      Ok(http::Response::from_parts(parts, ()))
    }
  }

  impl<'buffer, 'headers> TryFrom<Response<'buffer, 'headers>> for http::Response<Vec<u8>> {
    type Error = crate::Error;

    #[inline]
    fn try_from(from: Response<'buffer, 'headers>) -> Result<Self, Self::Error> {
      let (parts, body) = http::Response::<&'buffer [u8]>::try_from(from)?.into_parts();
      Ok(http::Response::from_parts(parts, body.to_vec()))
    }
  }
}
