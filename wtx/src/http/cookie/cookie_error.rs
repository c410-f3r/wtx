/// Cookie error
#[derive(Debug)]
pub enum CookieError {
  /// Cookie does not contain a `=` separator
  IrregularCookie,
  /// Cookie has an empty name
  MissingName,
}
