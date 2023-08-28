use httparse::Header;

/// Raw response that can be converted to other high-level responses.
#[derive(Debug)]
pub struct Response<'buffer, 'headers> {
    body: &'buffer [u8],
    res: httparse::Response<'headers, 'buffer>,
}

impl<'buffer, 'headers> Response<'buffer, 'headers> {
    pub(crate) fn new(body: &'buffer [u8], res: httparse::Response<'buffer, 'headers>) -> Self {
        Self { body, res }
    }

    /// Body
    #[inline]
    pub fn body(&self) -> &'buffer [u8] {
        self.body
    }

    /// Status code
    #[inline]
    pub fn code(&self) -> Option<u16> {
        self.res.code
    }

    pub(crate) fn headers(&self) -> &&'headers mut [Header<'buffer>] {
        &self.res.headers
    }
}

#[cfg(feature = "http")]
mod http {
    use crate::Response;
    use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};

    impl<'buffer, 'headers> TryFrom<Response<'buffer, 'headers>> for http::Response<&'buffer [u8]> {
        type Error = crate::Error;

        #[inline]
        fn try_from(from: Response<'buffer, 'headers>) -> Result<Self, Self::Error> {
            let status =
                StatusCode::from_u16(from.res.code.ok_or(crate::Error::UnexpectedHttpVersion)?)?;
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
            let mut res = http::Response::new(from.body);
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
