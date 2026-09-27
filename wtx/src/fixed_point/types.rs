macro_rules! doc {
  (signed, $bits:literal, $points:literal) => {
    concat!(
      "Signed decimal with a length of **",
      stringify!($bits),
      "** bits composed of **",
      stringify!($points),
      "** point(s)."
    )
  };
  (unsigned, $bits:literal, $points:literal) => {
    concat!(
      "Unsigned decimal with a length of **",
      stringify!($bits),
      "** bits composed of **",
      stringify!($points),
      "** point(s)."
    )
  };
}

use crate::fixed_point::FixedPoint;
use core::num::{NonZeroI64, NonZeroU64};

#[doc = doc!(signed, 64, 8)]
pub type Id64p8 = FixedPoint<i64, 8>;
#[doc = concat!(doc!(signed, 64, 8), " Non-zero.")]
pub type Id64p8nz = FixedPoint<NonZeroI64, 8>;
#[doc = doc!(unsigned, 64, 8)]
pub type Ud64p8 = FixedPoint<u64, 8>;
#[doc = concat!(doc!(unsigned, 64, 8), " Non-zero.")]
pub type Ud64p8nz = FixedPoint<NonZeroU64, 8>;
