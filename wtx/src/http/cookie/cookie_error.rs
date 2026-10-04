/// Cookie error
#[derive(Debug)]
pub enum CookieError {
  /// It is not possible to set more than one cookie in a header
  NonUniqueHeaderCookie,
  /// Cookie does not contain a `=` separator
  IrregularCookie,
  /// Cookie has an empty name
  MissingName,
}
