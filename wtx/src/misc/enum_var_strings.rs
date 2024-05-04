/// Enum Variant Strings
#[derive(Debug)]
pub struct EnumVarStrings<const N: usize> {
  /// Custom
  pub custom: [&'static str; N],
  /// Identifier
  pub ident: &'static str,
  /// Number
  pub number: &'static str,
}
