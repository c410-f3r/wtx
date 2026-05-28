// FIXME(STABLE): Constant traits

use crate::{
  codec::{U32String, u32_string_pad},
  misc::{AsciiGraphic, const_ok},
};

pub(crate) fn nanosecond_string(nanosecond: u32) -> U32String {
  u32_string_pad(nanosecond, const { const_ok(AsciiGraphic::new(b'0')).unwrap() }, 9)
}
