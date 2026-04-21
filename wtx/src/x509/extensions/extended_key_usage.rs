use crate::{
  asn1::{
    Asn1DecodeWrapper, Asn1EncodeWrapper, Len, OID_PKIX_KP_CLIENT_AUTH, OID_PKIX_KP_CODE_SIGNING,
    OID_PKIX_KP_EMAIL_PROTECTION, OID_PKIX_KP_OCSP_SIGNING, OID_PKIX_KP_SERVER_AUTH,
    OID_PKIX_KP_TIMESTAMPING, OID_X509_EXT_ANY_EXTENDED_KEY_USAGE, Oid, SEQUENCE_TAG,
    SequenceDecodeCb, SequenceEncodeIter,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collection::ArrayVectorU8,
  x509::X509Error,
};

/// This extension indicates one or more purposes for which the certified public key may be used,
/// in addition to or in place of the basic purposes indicated in the key usage extension.
#[derive(Debug, Default, PartialEq)]
pub struct ExtendedKeyUsage {
  any: bool,
  server_auth: bool,
  client_auth: bool,
  code_signing: bool,
  email_protection: bool,
  time_stamping: bool,
  ocsp_signing: bool,
  others: ArrayVectorU8<Oid, 1>,
}

impl ExtendedKeyUsage {
  /// An instance with the any OID.
  pub const ANY: Self = {
    let mut this = Self::EMPTY;
    *this.any_mut() = true;
    this
  };
  /// An instance with the client OID.
  pub const CLIENT: Self = {
    let mut this = Self::EMPTY;
    *this.client_auth_mut() = true;
    this
  };
  /// An instance without OIDs
  pub const EMPTY: Self = Self {
    any: true,
    server_auth: false,
    client_auth: false,
    code_signing: false,
    email_protection: false,
    time_stamping: false,
    ocsp_signing: false,
    others: ArrayVectorU8::new(),
  };
  /// An instance with the server OID.
  pub const SERVER: Self = {
    let mut this = Self::EMPTY;
    *this.server_auth_mut() = true;
    this
  };

  /// Generic usage.
  #[inline]
  pub const fn any(&self) -> bool {
    self.any
  }

  /// Mutable version of [`Self::any`].
  #[inline]
  pub const fn any_mut(&mut self) -> &mut bool {
    &mut self.any
  }

  /// TLS server authentication.
  #[inline]
  pub const fn server_auth(&self) -> bool {
    self.server_auth
  }

  /// Mutable version of [`Self::server_auth`].
  #[inline]
  pub const fn server_auth_mut(&mut self) -> &mut bool {
    &mut self.server_auth
  }

  /// TLS client authentication.
  #[inline]
  pub const fn client_auth(&self) -> bool {
    self.client_auth
  }

  /// Mutable version of [`Self::client_auth`].
  #[inline]
  pub const fn client_auth_mut(&mut self) -> &mut bool {
    &mut self.client_auth
  }

  /// Code signature verification.
  #[inline]
  pub const fn code_signing(&self) -> bool {
    self.code_signing
  }

  /// Mutable version of [`Self::code_signing`].
  #[inline]
  pub const fn code_signing_mut(&mut self) -> &mut bool {
    &mut self.code_signing
  }

  /// Email protection.
  #[inline]
  pub const fn email_protection(&self) -> bool {
    self.email_protection
  }

  /// Mutable version of [`Self::email_protection`].
  #[inline]
  pub const fn email_protection_mut(&mut self) -> &mut bool {
    &mut self.email_protection
  }

  /// RFC 3161 time-stamping.
  #[inline]
  pub const fn time_stamping(&self) -> bool {
    self.time_stamping
  }

  /// Mutable version of [`Self::time_stamping`].
  #[inline]
  pub const fn time_stamping_mut(&mut self) -> &mut bool {
    &mut self.time_stamping
  }

  /// Sign OCSP responses.
  #[inline]
  pub const fn ocsp_signing(&self) -> bool {
    self.ocsp_signing
  }

  /// Mutable version of [`Self::ocsp_signing`].
  #[inline]
  pub const fn ocsp_signing_mut(&mut self) -> &mut bool {
    &mut self.ocsp_signing
  }

  /// Custom Oids
  #[inline]
  pub const fn others(&self) -> &ArrayVectorU8<Oid, 1> {
    &self.others
  }

  /// Mutable version of [`Self::others`].
  #[inline]
  pub const fn others_mut(&mut self) -> &mut ArrayVectorU8<Oid, 1> {
    &mut self.others
  }

  /// The number of registered OIDs
  pub fn len(&self) -> u16 {
    let mut len = 0u16;
    len = len.wrapping_add(u16::from(self.any));
    len = len.wrapping_add(u16::from(self.server_auth));
    len = len.wrapping_add(u16::from(self.client_auth));
    len = len.wrapping_add(u16::from(self.code_signing));
    len = len.wrapping_add(u16::from(self.email_protection));
    len = len.wrapping_add(u16::from(self.time_stamping));
    len = len.wrapping_add(u16::from(self.ocsp_signing));
    len = len.wrapping_add(self.others.len().into());
    len
  }
}

impl<'de> Decode<'de, GenericCodec<Asn1DecodeWrapper, ()>> for ExtendedKeyUsage {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapper>) -> crate::Result<Self> {
    let mut this = ExtendedKeyUsage {
      any: false,
      server_auth: false,
      client_auth: false,
      code_signing: false,
      email_protection: false,
      time_stamping: false,
      ocsp_signing: false,
      others: ArrayVectorU8::new(),
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
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>) -> crate::Result<()> {
    if self.len() == 0 {
      return Err(X509Error::InvalidExtendedKeyUsage.into());
    }
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
