//! HTTP Cookie

mod cookie_generic;
mod cookie_str;

pub use cookie_generic::{CookieGeneric, SetCookieGeneric};
pub use cookie_str::CookieStr;

/// Cookie name type
pub type CookieNameTy = crate::collections::ArrayStrU8<23>;
