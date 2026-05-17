// FIXME(STABLE): Constant traits

use crate::{
  codec::{U32String, u32_string_pad},
  misc::AsciiGraphic,
};

pub(crate) fn nanosecond_string(nanosecond: u32) -> U32String {
  u32_string_pad(nanosecond, AsciiGraphic::ZERO, 9)
}
