/// Raw request that can be converted to other high-level requests.
#[derive(Debug)]
pub struct Request<'buffer, 'headers> {
    body: &'buffer [u8],
    req: httparse::Request<'headers, 'buffer>,
}

impl<'buffer> Request<'buffer, '_> {
    /// Body
    #[inline]
    pub fn body(&self) -> &'buffer [u8] {
        self.body
    }

    /// Method
    #[inline]
    pub fn method(&self) -> Option<&'buffer str> {
        self.req.method
    }
}

#[cfg(feature = "http")]
mod http {
    use crate::Request;
    use http::{HeaderMap, HeaderName, HeaderValue, Method};

    impl<'buffer, 'headers> TryFrom<Request<'buffer, 'headers>> for http::Request<&'buffer [u8]> {
        type Error = crate::Error;

        #[inline]
        fn try_from(from: Request<'buffer, 'headers>) -> Result<Self, Self::Error> {
            let method =
                Method::try_from(from.req.method.ok_or(crate::Error::UnexpectedHttpVersion)?)
                    .unwrap();
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
            let mut req = http::Request::new(from.body);
            *req.headers_mut() = headers;
            *req.method_mut() = method;
            *req.uri_mut() = from.req.path.unwrap().parse().unwrap();
            *req.version_mut() = version;
            //*req.status_mut() = status;
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
