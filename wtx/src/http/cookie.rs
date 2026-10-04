//! HTTP Cookie

mod cookie_error;
mod cookie_generic;
mod cookie_str;
mod same_site;

use crate::calendar::CalendarToken;
pub use cookie_error::CookieError;
pub use cookie_generic::{CookieGeneric, SetCookieGeneric};
pub use cookie_str::CookieStr;
pub use same_site::SameSite;

static FMT1: &[CalendarToken] = &[
  CalendarToken::AbbreviatedWeekdayName,
  CalendarToken::Comma,
  CalendarToken::Space,
  CalendarToken::TwoDigitDay,
  CalendarToken::Space,
  CalendarToken::AbbreviatedMonthName,
  CalendarToken::Space,
  CalendarToken::FourDigitYear,
  CalendarToken::Space,
  CalendarToken::TwoDigitHour,
  CalendarToken::Colon,
  CalendarToken::TwoDigitMinute,
  CalendarToken::Colon,
  CalendarToken::TwoDigitSecond,
  CalendarToken::Space,
  CalendarToken::Gmt,
];
