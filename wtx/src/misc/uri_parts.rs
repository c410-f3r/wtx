/// Elements that compose an URI.
///
/// ```txt
/// foo://user:pass@sub.domain.com:80/pa/th?query=value#hash
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct UriParts<'uri> {
    /// `user:pass@sub.domain.com:80`
    pub authority: &'uri str,
    /// `sub.domain.com:80`
    pub host: &'uri str,
    /// `sub.domain.com`
    pub hostname: &'uri str,
    /// `/pa/th?query=value#hash`
    pub href: &'uri str,
}

impl<'str> From<&'str str> for UriParts<'str> {
    #[inline]
    fn from(from: &'str str) -> Self {
        let after_schema = from.split("://").nth(1).unwrap_or(from);
        let (authority, href) = after_schema
            .as_bytes()
            .iter()
            .position(|el| el == &b'/')
            .map_or((after_schema, "/"), |el| after_schema.split_at(el));
        let host = authority.split('@').nth(1).unwrap_or(authority);
        let hostname = host.rsplit(':').nth(1).unwrap_or(host);
        Self {
            authority,
            host,
            hostname,
            href,
        }
    }
}
