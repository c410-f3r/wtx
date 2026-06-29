/// Client pool resource
#[derive(Debug)]
pub struct Http2ClientPoolResource<C> {
  /// Client
  pub client: C,
}
