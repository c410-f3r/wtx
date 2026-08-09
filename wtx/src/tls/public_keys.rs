use crate::{
  asn1::Asn1DecodeWrapperAux,
  codec::{Decode as _, DecodeWrapper, Pem},
  collections::{ArrayVectorCopy, ShortBoxSliceU16, Vector},
  tls::{MAX_CERTS, MAX_KEYS, TlsError},
  x509::{Certificate, KeyTy},
};
use core::mem;

/// A collection responsible for storing public keys
//
// ```
// [PK(0)|CERT(0), PK(0)|CERT(1)                ]
// [PK(1)|CERT(0), PK(1)|CERT(1), PK(1)|CERT(2)]
// [PK(2)|CERT(0)                                ]
// ```
#[derive(Debug, Default)]
pub struct PublicKeys {
  data: ShortBoxSliceU16<u8>,
  data_offsets: ArrayVectorCopy<u16, { MAX_CERTS * MAX_KEYS }>,
  public_keys_offsets: ArrayVectorCopy<(u8, KeyTy), MAX_KEYS>,
}

impl PublicKeys {
  /// Iterates over all public keys
  #[inline]
  pub const fn iter(&self) -> PublicKeysIter<'_> {
    PublicKeysIter {
      curr_cert_offset: 0,
      curr_public_key_offset: 0,
      public_key_idx: 0,
      public_keys: self,
    }
  }

  #[inline]
  pub(crate) fn clear(&mut self) {
    let Self { data, data_offsets, public_keys_offsets } = self;
    let _local_data = mem::take(data);
    data_offsets.clear();
    public_keys_offsets.clear();
  }

  #[inline]
  pub(crate) fn get(&self, public_key_idx: u8) -> Option<PublicKeyRef<'_>> {
    let end = *self.public_keys_offsets.get(usize::from(public_key_idx))?;
    let begin = if let Some(index) = public_key_idx.checked_sub(1) {
      self.public_keys_offsets.get(usize::from(index))?.0
    } else {
      0
    };
    let begin_offset = if let Some(index) = begin.checked_sub(1) {
      *self.data_offsets.get(usize::from(index))?
    } else {
      0
    };
    let certs = self.data_offsets.get(usize::from(begin)..usize::from(end.0))?;
    Some(PublicKeyRef { begin_offset, data_offsets: certs, data: &self.data, key_ty: end.1 })
  }

  #[inline]
  pub(crate) fn key_tys(&self) -> impl Clone + Iterator<Item = KeyTy> {
    self.public_keys_offsets.iter().map(|el| el.1)
  }

  #[inline]
  pub(crate) fn push_public_key_der<'blocks>(
    &mut self,
    public_key: impl IntoIterator<Item = &'blocks [u8]>,
  ) -> crate::Result<()> {
    let mut certs: u8 = 0;
    let mut curr_data_offset: u16 = self.data_offsets.last().copied().unwrap_or_default();
    let mut iter = public_key.into_iter();
    let mut local_data: Vector<u8> = mem::take(&mut self.data).into();

    let Some(first_cert_bytes) = iter.next() else {
      return Err(TlsError::NoLeafCertInChain.into());
    };
    let public_key_ty = KeyTy::try_from(&cert_from_der(first_cert_bytes)?)?;
    self.push_cert(first_cert_bytes, &mut certs, &mut local_data, &mut curr_data_offset)?;

    for cert_bytes in iter {
      let _cert = cert_from_der(cert_bytes)?;
      self.push_cert(cert_bytes, &mut certs, &mut local_data, &mut curr_data_offset)?;
    }

    self.finish_push(certs, local_data, public_key_ty)
  }

  #[inline]
  pub(crate) fn push_public_key_pem(
    &mut self,
    buffer: &mut Vector<u8>,
    pem_bytes: &[u8],
  ) -> crate::Result<()> {
    let pem = Pem::<_, MAX_CERTS>::decode(&mut DecodeWrapper::new(pem_bytes, &mut *buffer))?;
    let mut certs: u8 = 0;
    let mut curr_data_offset: u16 = self.data_offsets.last().copied().unwrap_or_default();
    let mut iter = pem.data.into_iter();
    let mut local_data: Vector<u8> = mem::take(&mut self.data).into();

    let Some(first) = iter.next() else {
      return Err(TlsError::NoLeafCertInChain.into());
    };
    let first_cert_bytes = buffer.get(first.1).unwrap_or_default();
    let public_key_ty = KeyTy::try_from(&cert_from_der(first_cert_bytes)?)?;
    self.push_cert(first_cert_bytes, &mut certs, &mut local_data, &mut curr_data_offset)?;

    for (_, range) in iter {
      let cert_bytes = buffer.get(range).unwrap_or_default();
      let _cert = cert_from_der(cert_bytes)?;
      self.push_cert(cert_bytes, &mut certs, &mut local_data, &mut curr_data_offset)?;
    }

    self.finish_push(certs, local_data, public_key_ty)
  }

  #[inline]
  fn finish_push(
    &mut self,
    certs: u8,
    data: Vector<u8>,
    public_key_ty: KeyTy,
  ) -> crate::Result<()> {
    self.data = data.try_into()?;
    self.public_keys_offsets.push((
      self.public_keys_offsets.last().copied().unwrap_or_default().0.wrapping_add(certs),
      public_key_ty,
    ))?;
    Ok(())
  }

  #[inline]
  fn push_cert(
    &mut self,
    cert_bytes: &[u8],
    certs: &mut u8,
    data: &mut Vector<u8>,
    curr_data_offset: &mut u16,
  ) -> crate::Result<()> {
    *certs = certs.wrapping_add(1);
    data.extend_from_copyable_slice(cert_bytes)?;
    *curr_data_offset = curr_data_offset.wrapping_add(cert_bytes.len().try_into()?);
    self.data_offsets.push(*curr_data_offset)?;
    Ok(())
  }
}

impl<'any> IntoIterator for &'any PublicKeys {
  type Item = PublicKeyRef<'any>;
  type IntoIter = PublicKeysIter<'any>;

  #[inline]
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

/// Iterator of public key references
#[derive(Debug)]
pub struct PublicKeysIter<'any> {
  curr_cert_offset: u16,
  curr_public_key_offset: u16,
  public_key_idx: u8,
  public_keys: &'any PublicKeys,
}

impl<'any> Iterator for PublicKeysIter<'any> {
  type Item = PublicKeyRef<'any>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    let begin = self.curr_public_key_offset;
    let end = *self.public_keys.public_keys_offsets.get(usize::from(self.public_key_idx))?;
    let certs = self.public_keys.data_offsets.get(usize::from(begin)..usize::from(end.0))?;
    let begin_offset = self.curr_cert_offset;
    self.curr_cert_offset = certs.last().copied().unwrap_or(begin_offset);
    self.curr_public_key_offset = u16::from(end.0);
    self.public_key_idx = self.public_key_idx.wrapping_add(1);
    Some(PublicKeyRef {
      begin_offset,
      data_offsets: certs,
      data: &self.public_keys.data,
      key_ty: end.1,
    })
  }
}

/// Public Key reference that is highly coupled to [`PublicKeys`].
#[derive(Debug)]
pub struct PublicKeyRef<'any> {
  begin_offset: u16,
  data: &'any [u8],
  data_offsets: &'any [u16],
  key_ty: KeyTy,
}

impl<'any> PublicKeyRef<'any> {
  /// All certificates or chain that composes this public key.
  #[inline]
  pub fn certs(&self) -> impl Iterator<Item = &'any [u8]> {
    let Self { begin_offset, data, data_offsets, key_ty: _ } = self;
    data_offsets.iter().scan(*begin_offset, |curr_offset, &offset| {
      let rslt = data.get(usize::from(*curr_offset)..usize::from(offset));
      *curr_offset = offset;
      rslt
    })
  }

  /// [`KeyTy`] of the leaf certificate
  #[inline]
  pub const fn key_ty(&self) -> KeyTy {
    self.key_ty
  }
}

fn cert_from_der(der: &[u8]) -> crate::Result<Certificate<&[u8]>> {
  Certificate::<&[u8]>::decode(&mut DecodeWrapper::new(der, Asn1DecodeWrapperAux::default()))
}
