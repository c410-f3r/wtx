// https://datatracker.ietf.org/doc/html/rfc9846#section-7.1

use crate::{
  collections::ArrayVectorCopy,
  misc::{Lease as _, TryArithmetic as _},
  tls::{
    CipherSuite, IV_LEN, MAX_CIPHER_KEY_LEN, MAX_HASH_LEN, MAX_LABEL_LEN, tls_hash::TlsDigest,
    tls_hkdf::TlsHkdf,
  },
};
use core::fmt::{Debug, Formatter};

/// Responsible for deriving keys used for encryption.
pub struct KeySchedule {
  cipher_suite: CipherSuite,
  common_hkdf: TlsHkdf,
  common_secret: ArrayVectorCopy<u8, MAX_HASH_LEN>,
  exporter_secret: ArrayVectorCopy<u8, MAX_HASH_LEN>,
  read: KeyScheduleRead,
  write: KeyScheduleWrite,
}

impl KeySchedule {
  #[inline]
  pub(crate) fn from_cipher_suite(cipher_suite: CipherSuite) -> Self {
    let counter = 0;
    let iv = [0; IV_LEN];
    Self {
      cipher_suite,
      common_hkdf: cipher_suite.hkdf_extract(None, &[]),
      common_secret: cipher_suite.zeroed_hash(),
      exporter_secret: ArrayVectorCopy::new(),
      read: KeyScheduleRead {
        state: KeyScheduleState {
          cipher_key: ArrayVectorCopy::new(),
          cipher_suite,
          counter,
          iv,
          raw_traffic_secret: ArrayVectorCopy::new(),
          traffic_secret: cipher_suite.hkdf_extract(None, &[]),
        },
      },
      write: KeyScheduleWrite {
        state: KeyScheduleState {
          cipher_key: ArrayVectorCopy::new(),
          cipher_suite,
          counter,
          iv,
          raw_traffic_secret: ArrayVectorCopy::new(),
          traffic_secret: cipher_suite.hkdf_extract(None, &[]),
        },
      },
    }
  }

  #[inline]
  pub(crate) const fn cipher_suite(&self) -> CipherSuite {
    self.cipher_suite
  }

  #[inline]
  pub(crate) fn early_secret(&mut self) -> crate::Result<()> {
    self.hkdf_extract(&self.cipher_suite.zeroed_hash());
    self.common_secret = derive_secret_derived(self.cipher_suite, &self.common_hkdf)?;
    Ok(())
  }

  #[inline]
  pub(crate) fn export_keying_material(
    cipher_suite: CipherSuite,
    context: Option<&[u8]>,
    exporter_secret: &ArrayVectorCopy<u8, MAX_HASH_LEN>,
    label: &[u8],
    output: &mut [u8],
  ) -> crate::Result<()> {
    let exporter_hkdf = cipher_suite.hkdf_from_prk(exporter_secret)?;
    let empty_hash = cipher_suite.hash_digest([&[][..]]);
    let derived_secret = hkdf_expand_label::<MAX_HASH_LEN>(
      Some(empty_hash.lease()),
      label,
      cipher_suite.hash_len(),
      &exporter_hkdf,
    )?;
    let context_hash = cipher_suite.hash_digest([context.unwrap_or(&[])]);
    let derived_hkdf = cipher_suite.hkdf_from_prk(&derived_secret)?;
    let concatenated =
      hkdf_array(Some(context_hash.lease()), b"exporter", output.len().try_into()?)?;
    derived_hkdf.expand(concatenated.as_slice(), output)?;
    Ok(())
  }

  #[inline]
  pub(crate) const fn exporter_secret(&self) -> &ArrayVectorCopy<u8, MAX_HASH_LEN> {
    &self.exporter_secret
  }

  #[inline]
  pub(crate) fn handshake_secret<const IS_CLIENT: bool>(
    &mut self,
    ikm: &[u8],
    transcript_hash: &TlsDigest,
  ) -> crate::Result<()> {
    let tuple = if IS_CLIENT {
      ("c hs traffic".as_bytes(), "s hs traffic".as_bytes())
    } else {
      ("s hs traffic".as_bytes(), "c hs traffic".as_bytes())
    };
    self.hkdf_extract(ikm);
    self.calculate_traffic_secrets(tuple, transcript_hash)?;
    self.common_secret = derive_secret_derived(self.cipher_suite, &self.common_hkdf)?;
    Ok(())
  }

  #[inline]
  pub(crate) fn into_split(self) -> (KeyScheduleRead, KeyScheduleWrite) {
    (self.read, self.write)
  }

  #[inline]
  pub(crate) fn master_secret<const IS_CLIENT: bool>(
    &mut self,
    transcript_hash: &TlsDigest,
  ) -> crate::Result<()> {
    let tuple = if IS_CLIENT {
      ("c ap traffic".as_bytes(), "s ap traffic".as_bytes())
    } else {
      ("s ap traffic".as_bytes(), "c ap traffic".as_bytes())
    };
    self.hkdf_extract(&self.cipher_suite.zeroed_hash());
    self.calculate_traffic_secrets(tuple, transcript_hash)?;
    self.exporter_secret = derive_secret(
      self.cipher_suite,
      Some(transcript_hash.lease()),
      b"exp master",
      &self.common_hkdf,
    )?;
    Ok(())
  }

  #[inline]
  pub(crate) const fn read(&self) -> &KeyScheduleRead {
    &self.read
  }

  #[inline]
  pub(crate) const fn read_mut(&mut self) -> &mut KeyScheduleRead {
    &mut self.read
  }

  #[inline]
  pub(crate) fn set_cipher_suite(&mut self, cipher_suite: CipherSuite) {
    if self.cipher_suite == cipher_suite {
      self.cipher_suite = cipher_suite;
      self.read.state.cipher_suite = cipher_suite;
      self.write.state.cipher_suite = cipher_suite;
    } else {
      *self = Self::from_cipher_suite(cipher_suite);
    }
  }

  #[inline]
  pub(crate) const fn split_mut(&mut self) -> (&mut KeyScheduleRead, &mut KeyScheduleWrite) {
    (&mut self.read, &mut self.write)
  }

  #[inline]
  pub(crate) const fn write(&self) -> &KeyScheduleWrite {
    &self.write
  }

  #[inline]
  pub(crate) const fn write_mut(&mut self) -> &mut KeyScheduleWrite {
    &mut self.write
  }

  #[inline]
  fn calculate_traffic_secrets(
    &mut self,
    (lhs, rhs): (&'static [u8], &'static [u8]),
    transcript_hash: &TlsDigest,
  ) -> crate::Result<()> {
    self.read.state.update(Some(&self.common_hkdf), rhs, Some(transcript_hash))?;
    self.write.state.update(Some(&self.common_hkdf), lhs, Some(transcript_hash))?;
    Ok(())
  }

  #[inline]
  fn hkdf_extract(&mut self, ikm: &[u8]) {
    let hkdf = self.cipher_suite.hkdf_extract(Some(&self.common_secret), ikm);
    self.common_hkdf = hkdf;
  }
}

impl Debug for KeySchedule {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("KeySchedule").finish()
  }
}

impl Default for KeySchedule {
  #[inline]
  fn default() -> Self {
    Self::from_cipher_suite(CipherSuite::default())
  }
}

pub(crate) struct KeyScheduleRead {
  state: KeyScheduleState,
}

impl KeyScheduleRead {
  #[inline]
  pub(crate) const fn state(&self) -> &KeyScheduleState {
    &self.state
  }

  #[inline]
  pub(crate) const fn state_mut(&mut self) -> &mut KeyScheduleState {
    &mut self.state
  }
}

impl Debug for KeyScheduleRead {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("KeyScheduleRead").finish()
  }
}

pub(crate) struct KeyScheduleState {
  cipher_key: ArrayVectorCopy<u8, MAX_CIPHER_KEY_LEN>,
  cipher_suite: CipherSuite,
  counter: u64,
  iv: [u8; IV_LEN],
  raw_traffic_secret: ArrayVectorCopy<u8, MAX_HASH_LEN>,
  traffic_secret: TlsHkdf,
}

impl KeyScheduleState {
  #[inline]
  pub(crate) fn create_finished_verify_data(
    &self,
    hash: &[u8],
  ) -> crate::Result<ArrayVectorCopy<u8, MAX_HASH_LEN>> {
    let hash_len = self.cipher_suite.hash_len();
    let key = hkdf_expand_label::<MAX_HASH_LEN>(None, b"finished", hash_len, &self.traffic_secret)?;
    let mut rslt = ArrayVectorCopy::new();
    let mut hmac = self.cipher_suite.hmac_from_key(&key)?;
    hmac.update(hash);
    rslt.extend_from_copyable_slice(hmac.finalize().lease())?;
    Ok(rslt)
  }

  #[inline]
  pub(crate) fn rotate(&mut self) -> crate::Result<()> {
    self.update(None, b"traffic upd", None)
  }

  #[inline]
  pub(crate) fn verify_finished_record(
    &self,
    hash: &[u8],
    verify_data: &[u8],
  ) -> crate::Result<()> {
    let hash_len = self.cipher_suite.hash_len();
    let key = hkdf_expand_label::<MAX_HASH_LEN>(None, b"finished", hash_len, &self.traffic_secret)?;
    let mut hmac = self.cipher_suite.hmac_from_key(&key)?;
    hmac.update(hash);
    hmac.verify(verify_data)?;
    Ok(())
  }

  #[inline]
  pub(crate) const fn cipher_key(&self) -> &ArrayVectorCopy<u8, MAX_CIPHER_KEY_LEN> {
    &self.cipher_key
  }

  #[inline]
  pub(crate) const fn cipher_suite(&self) -> CipherSuite {
    self.cipher_suite
  }

  #[inline]
  pub(crate) const fn increment_counter(&mut self) {
    self.counter = self.counter.wrapping_add(1);
  }

  #[inline]
  pub(crate) fn nonce(&self) -> [u8; IV_LEN] {
    nonce(self.counter, &self.iv)
  }

  #[inline]
  pub(crate) fn raw_traffic_secret(&self) -> &[u8] {
    self.raw_traffic_secret.as_slice()
  }

  #[inline(always)]
  fn update(
    &mut self,
    hkdf: Option<&TlsHkdf>,
    label: &'static [u8],
    transcript_hash: Option<&TlsDigest>,
  ) -> crate::Result<()> {
    let cipher_suite = self.cipher_suite;
    let secret = match (hkdf, transcript_hash) {
      (None, None) => derive_secret(cipher_suite, None, label, &self.traffic_secret)?,
      (None, Some(el)) => {
        derive_secret(cipher_suite, Some(el.lease()), label, &self.traffic_secret)?
      }
      (Some(el), None) => derive_secret(cipher_suite, None, label, el)?,
      (Some(lhs), Some(rhs)) => derive_secret(cipher_suite, Some(rhs.lease()), label, lhs)?,
    };
    self.raw_traffic_secret = secret;
    self.traffic_secret = cipher_suite.hkdf_from_prk(&secret)?;
    let cipher_key_len = cipher_suite.cipher_key_len();
    let iv = hkdf_expand_label(None, b"iv", IV_LEN.try_into()?, &self.traffic_secret)?;
    self.cipher_key = hkdf_expand_label(None, b"key", cipher_key_len, &self.traffic_secret)?;
    self.counter = 0;
    self.iv = iv.into_inner()?;
    Ok(())
  }
}

pub(crate) struct KeyScheduleWrite {
  state: KeyScheduleState,
}

impl KeyScheduleWrite {
  #[inline]
  pub(crate) const fn state(&self) -> &KeyScheduleState {
    &self.state
  }

  #[inline]
  pub(crate) const fn state_mut(&mut self) -> &mut KeyScheduleState {
    &mut self.state
  }
}

impl Debug for KeyScheduleWrite {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("KeyScheduleWrite").finish()
  }
}

// Differently from the RFC, this function expects a hash instead of raw bytes
#[inline]
fn derive_secret(
  cipher_suite: CipherSuite,
  context: Option<&[u8]>,
  label: &'static [u8],
  secret: &TlsHkdf,
) -> crate::Result<ArrayVectorCopy<u8, MAX_HASH_LEN>> {
  hkdf_expand_label(context, label, cipher_suite.hash_len(), secret)
}

#[inline]
fn derive_secret_derived(
  cipher_suite: CipherSuite,
  secret: &TlsHkdf,
) -> crate::Result<ArrayVectorCopy<u8, MAX_HASH_LEN>> {
  let mut context_buffer = ArrayVectorCopy::<_, MAX_HASH_LEN>::new();
  context_buffer.extend_from_copyable_slice(cipher_suite.hash_digest([]).lease())?;
  derive_secret(cipher_suite, Some(context_buffer.as_slice()), b"derived", secret)
}

#[inline]
fn hkdf_array(
  context: Option<&[u8]>,
  label: &[u8],
  output_len: u16,
) -> crate::Result<ArrayVectorCopy<u8, MAX_LABEL_LEN>> {
  let label_len = 6u8.try_add(label.len().try_into()?)?;
  let mut concatenated = ArrayVectorCopy::<_, MAX_LABEL_LEN>::new();
  match context {
    None => {
      let _ = concatenated.extend_from_copyable_slices([
        output_len.to_be_bytes().as_slice(),
        &label_len.to_be_bytes(),
        b"tls13 ",
        label,
        &[0][..],
      ])?;
    }
    Some(value) => {
      let _ = concatenated.extend_from_copyable_slices([
        output_len.to_be_bytes().as_slice(),
        &label_len.to_be_bytes(),
        b"tls13 ",
        label,
        u8::try_from(value.len())?.to_be_bytes().as_slice(),
        value,
      ])?;
    }
  }
  Ok(concatenated)
}

#[inline]
fn hkdf_expand_label<const LENGTH: usize>(
  context: Option<&[u8]>,
  label: &[u8],
  output_len: u8,
  secret: &TlsHkdf,
) -> crate::Result<ArrayVectorCopy<u8, LENGTH>> {
  let concatenated = hkdf_array(context, label, output_len.into())?;
  let mut output = ArrayVectorCopy::from_array([0; LENGTH]);
  output.truncate(output_len);
  secret.expand(concatenated.as_slice(), &mut output)?;
  Ok(output)
}

#[inline]
fn nonce(counter: u64, iv: &[u8; IV_LEN]) -> [u8; IV_LEN] {
  let padded = pad_left::<IV_LEN>(&counter.to_be_bytes());
  let mut nonce = [0; IV_LEN];
  for (elem, (lhs, rhs)) in nonce.iter_mut().zip(iv.iter().zip(padded)) {
    *elem = lhs ^ rhs;
  }
  nonce
}

#[inline]
fn pad_left<const N: usize>(bytes: &[u8]) -> [u8; N] {
  let mut padded = [0u8; N];
  let len = bytes.len().min(N);
  let dst_idx = N.wrapping_sub(len);
  let src_idx = bytes.len().wrapping_sub(len);
  let Some(dst) = padded.get_mut(dst_idx..) else {
    return padded;
  };
  let Some(src) = bytes.get(src_idx..) else {
    return padded;
  };
  dst.copy_from_slice(src);
  padded
}
