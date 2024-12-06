use alloc::vec::Vec;

#[derive(Debug, PartialEq)]
pub(crate) enum Item {
  X509DerCertificate(Vec<u8>),
}

#[cfg(feature = "rustls-pki-types")]
mod rustls {
  use crate::tls::item::Item;
  use rustls_pki_types::pem::{PemObject, SectionKind};
  use std::vec::Vec;

  impl Item {
    #[inline]
    pub(crate) fn rustls_pki_types(
      pem: &[u8],
    ) -> impl Iterator<Item = Result<Self, rustls_pki_types::pem::Error>> + use<'_> {
      PemObject::pem_slice_iter(pem).filter_map(|rslt| {
        let (sk, bytes) = match rslt {
          Err(err) => return Some(Err(err)),
          Ok(elem) => elem,
        };
        Ok(Self::from_rustls_kind(bytes, sk)).transpose()
      })
    }

    #[inline]
    fn from_rustls_kind(bytes: Vec<u8>, sk: SectionKind) -> Option<Self> {
      match sk {
        SectionKind::Certificate => Some(Self::X509DerCertificate(bytes.into())),
        _ => None,
      }
    }
  }
}
