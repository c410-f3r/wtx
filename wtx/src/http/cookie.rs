//! HTTP Cookie

mod cookie_generic;
mod cookie_str;

pub use cookie_generic::{CookieGeneric, SetCookieGeneric};
pub use cookie_str::CookieStr;
