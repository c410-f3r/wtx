mod dyn_tz;
mod local;
mod utc;

use crate::collection::ArrayStringU8;
pub use dyn_tz::DynTz;
pub use local::Local;
pub use utc::Utc;

/// Timezone
pub trait TimeZone: Copy {
  /// If the instance is of a literal `Local` type.
  const IS_LOCAL: bool;
  /// If the instance is of a literal `UTC` type.
  const IS_UTC: bool;

  /// Tries to create a new instance from the number of minutes.
  fn from_minutes(minutes: i16) -> crate::Result<Self>;

  /// ISO-8601 string representation
  fn iso8601(self) -> ArrayStringU8<6>;

  /// The number of minutes represented by this time zone
  fn minutes(&self) -> i16;
}
