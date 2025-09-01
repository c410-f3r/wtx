mod cookie_error;
#[cfg(feature = "http-session")]
pub(crate) mod cookie_generic;
#[cfg(all(feature = "http-server-framework", feature = "http-session"))]
pub(crate) mod cookie_str;
mod same_site;

#[cfg(feature = "http-session")]
use crate::calendar::CalendarToken;
pub use cookie_error::CookieError;
pub use same_site::SameSite;

#[cfg(feature = "http-session")]
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
