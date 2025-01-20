/// Client pool resource
#[derive(Debug)]
pub struct ClientPoolResource<AUX, C> {
  /// Auxiliary structure
  pub aux: AUX,
  /// Client
  pub client: C,
}
