/// Wrapper around JSON values
#[derive(Debug)]
#[cfg_attr(feature = "serde_json", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde_json", serde(transparent))]
pub struct Json<T: ?Sized>(
  /// Value
  pub T,
);
