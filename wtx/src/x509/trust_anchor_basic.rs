use crate::{
  asn1::{Asn1DecodeWrapper, OID_X509_EXT_NAME_CONSTRAINTS},
  codec::{Decode, GenericDecodeWrapper},
  misc::RefOrOwned,
  x509::{Certificate, Name, SubjectPublicKeyInfo, extensions::NameConstraints},
};

/// A trust anchor is the top-most certificate in a certificate chain. At the end of the day it
/// leads to the root CA, regardless of the amount of intermediates.
///
/// Full X.509 certificates are huge and because of that, this particular structure only has the fields
/// required to perform a chain validation.
#[derive(Debug)]
pub struct TrustAnchorBasic<'any, 'bytes> {
  pub(crate) subject: RefOrOwned<'any, Name<'bytes>>,
  pub(crate) subject_public_key_info: RefOrOwned<'any, SubjectPublicKeyInfo<'bytes>>,
  pub(crate) _name_constraints: Option<NameConstraints<'bytes>>,
}

impl<'any, 'bytes> TryFrom<Certificate<'bytes>> for TrustAnchorBasic<'any, 'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: Certificate<'bytes>) -> Result<Self, Self::Error> {
    let _name_constraints = nc(&value)?;
    Ok(Self {
      subject: RefOrOwned::Right(value.tbs_certificate.subject),
      subject_public_key_info: RefOrOwned::Right(value.tbs_certificate.subject_public_key_info),
      _name_constraints,
    })
  }
}

impl<'any, 'bytes> TryFrom<&'any Certificate<'bytes>> for TrustAnchorBasic<'any, 'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: &'any Certificate<'bytes>) -> Result<Self, Self::Error> {
    let _name_constraints = nc(&value)?;
    Ok(Self {
      subject: RefOrOwned::Left(&value.tbs_certificate.subject),
      subject_public_key_info: RefOrOwned::Left(&value.tbs_certificate.subject_public_key_info),
      _name_constraints,
    })
  }
}

fn nc<'bytes>(value: &Certificate<'bytes>) -> crate::Result<Option<NameConstraints<'bytes>>> {
  let _name_constraints = if let Some(extensions) = &value.tbs_certificate.extensions
    && let Some(ext) = extensions.0.iter().find(|el| el.extn_id == OID_X509_EXT_NAME_CONSTRAINTS)
  {
    let nc = NameConstraints::decode(&mut GenericDecodeWrapper::new(
      ext.extn_value.bytes(),
      Asn1DecodeWrapper::default(),
    ))?;
    Some(nc)
  } else {
    None
  };
  Ok(_name_constraints)
}
