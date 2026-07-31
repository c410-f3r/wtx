use crate::http2::Http2;

/// Client pool resource
#[derive(Debug)]
pub struct Http2ClientPoolResource<AUX, SW, TM> {
  /// Auxiliary data
  pub aux: AUX,
  /// Client
  pub client: Http2<SW, TM, true>,
}
