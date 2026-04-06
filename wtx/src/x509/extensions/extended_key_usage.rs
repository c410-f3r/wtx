use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Len, OID_PKIX_KP_CLIENT_AUTH, OID_PKIX_KP_CODE_SIGNING,
    OID_PKIX_KP_EMAIL_PROTECTION, OID_PKIX_KP_OCSP_SIGNING, OID_PKIX_KP_SERVER_AUTH,
    OID_PKIX_KP_TIMESTAMPING, OID_X509_EXT_ANY_EXTENDED_KEY_USAGE, Oid, SEQUENCE_TAG,
    SequenceDecodeCb, SequenceEncodeIter,
  },
  codec::{Decode, Encode, GenericCodec, GenericDecodeWrapper, GenericEncodeWrapper},
  collection::Vector,
  x509::X509Error,
};

/// This extension indicates one or more purposes for which the certified public key may be used,
/// in addition to or in place of the basic purposes indicated in the key usage extension.
#[derive(Debug, PartialEq)]
pub struct ExtendedKeyUsage {
  any: bool,
  server_auth: bool,
  client_auth: bool,
  code_signing: bool,
  email_protection: bool,
  time_stamping: bool,
  ocsp_signing: bool,
  others: Vector<Oid>,
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for ExtendedKeyUsage {
  #[inline]
  fn decode(dw: &mut GenericDecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let mut this = ExtendedKeyUsage {
      any: false,
      server_auth: false,
      client_auth: false,
      code_signing: false,
      email_protection: false,
      time_stamping: false,
      ocsp_signing: false,
      others: Vector::new(),
    };
    let mut has_at_least_one = false;
    SequenceDecodeCb::new(|oid: Oid| {
      if oid == OID_X509_EXT_ANY_EXTENDED_KEY_USAGE {
        has_at_least_one = true;
        this.any = true;
      } else if oid == OID_PKIX_KP_SERVER_AUTH {
        has_at_least_one = true;
        this.server_auth = true;
      } else if oid == OID_PKIX_KP_CLIENT_AUTH {
        has_at_least_one = true;
        this.client_auth = true;
      } else if oid == OID_PKIX_KP_CODE_SIGNING {
        has_at_least_one = true;
        this.code_signing = true;
      } else if oid == OID_PKIX_KP_EMAIL_PROTECTION {
        has_at_least_one = true;
        this.email_protection = true;
      } else if oid == OID_PKIX_KP_TIMESTAMPING {
        has_at_least_one = true;
        this.time_stamping = true;
      } else if oid == OID_PKIX_KP_OCSP_SIGNING {
        has_at_least_one = true;
        this.ocsp_signing = true;
      } else {
        has_at_least_one = true;
        this.others.push(oid)?;
      }
      Ok(())
    })
    .decode(dw, SEQUENCE_TAG)?;
    if !has_at_least_one {
      return Err(X509Error::InvalidExtendedKeyUsage.into());
    }
    Ok(this)
  }
}

impl Encode<GenericCodec<(), Asn1EncodeWrapper>> for ExtendedKeyUsage {
  #[inline]
  fn encode(&self, ew: &mut GenericEncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    let iter = [
      self.any.then_some(&OID_X509_EXT_ANY_EXTENDED_KEY_USAGE),
      self.server_auth.then_some(&OID_PKIX_KP_SERVER_AUTH),
      self.client_auth.then_some(&OID_PKIX_KP_CLIENT_AUTH),
      self.code_signing.then_some(&OID_PKIX_KP_CODE_SIGNING),
      self.email_protection.then_some(&OID_PKIX_KP_EMAIL_PROTECTION),
      self.time_stamping.then_some(&OID_PKIX_KP_TIMESTAMPING),
      self.ocsp_signing.then_some(&OID_PKIX_KP_OCSP_SIGNING),
    ]
    .into_iter()
    .flatten()
    .chain(self.others.iter());
    SequenceEncodeIter(iter).encode(ew, Len::MAX_THREE_BYTES, SEQUENCE_TAG)?;
    Ok(())
  }
}

/// Builder for [`ExtendedKeyUsage`].
#[derive(Debug, Default)]
pub struct ExtendedKeyUsageBuilder {
  any: bool,
  server_auth: bool,
  client_auth: bool,
  code_signing: bool,
  email_protection: bool,
  time_stamping: bool,
  ocsp_signing: bool,
  others: Vector<Oid>,
}

impl ExtendedKeyUsageBuilder {
  /// Generic usage.
  pub fn any(mut self, elem: bool) -> Self {
    self.any = elem;
    self
  }

  /// TLS server authentication.
  pub fn server_auth(mut self, elem: bool) -> Self {
    self.server_auth = elem;
    self
  }

  /// TLS client authentication.
  pub fn client_auth(mut self, elem: bool) -> Self {
    self.client_auth = elem;
    self
  }

  /// Code signature verification.
  pub fn code_signing(mut self, elem: bool) -> Self {
    self.code_signing = elem;
    self
  }

  /// Email protection.
  pub fn email_protection(mut self, elem: bool) -> Self {
    self.email_protection = elem;
    self
  }

  /// RFC 3161 time-stamping.
  pub fn time_stamping(mut self, elem: bool) -> Self {
    self.time_stamping = elem;
    self
  }

  /// Sign OCSP responses.
  pub fn ocsp_signing(mut self, elem: bool) -> Self {
    self.ocsp_signing = elem;
    self
  }

  /// Custom Oids
  pub fn others(mut self, elem: Vector<Oid>) -> Self {
    self.others = elem;
    self
  }

  /// Consume the instance to produce an [`ExtendedKeyUsage`].
  pub fn build(self) -> crate::Result<ExtendedKeyUsage> {
    if !self.any
      && !self.server_auth
      && !self.client_auth
      && !self.code_signing
      && !self.email_protection
      && !self.time_stamping
      && !self.ocsp_signing
      && self.others.is_empty()
    {
      return Err(X509Error::InvalidExtendedKeyUsage.into());
    }
    Ok(ExtendedKeyUsage {
      any: self.any,
      server_auth: self.server_auth,
      client_auth: self.client_auth,
      code_signing: self.code_signing,
      email_protection: self.email_protection,
      time_stamping: self.time_stamping,
      ocsp_signing: self.ocsp_signing,
      others: self.others,
    })
  }
}
