use crate::tls::{CipherSuiteTy, PskTy};

/// Pre Shared Key
#[derive(Clone, Copy, Debug)]
pub struct Psk<'data> {
  /// See [`CipherSuiteTy`]
  pub cipher_suite_ty: CipherSuiteTy,
  /// Data
  pub data: &'data [u8],
  /// See [`PskTy`]
  pub psk_ty: PskTy,
}

impl<'data> Psk<'data> {
  /// Shortcut
  pub fn new(cipher_suite_ty: CipherSuiteTy, data: &'data [u8], psk_ty: PskTy) -> Self {
    Self { cipher_suite_ty, data, psk_ty }
  }
}
