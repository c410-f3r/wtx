// https://datatracker.ietf.org/doc/html/rfc8446#section-7.1

use crate::{
  collections::ArrayVectorU8,
  crypto::{Hkdf as _, Hmac as _},
  misc::{Either, Lease as _, TryArithmetic as _},
  tls::{
    CipherSuite, IV_LEN, MAX_CIPHER_KEY_LEN, MAX_HASH_LEN, MAX_LABEL_LEN, PskTy, tls_hash::TlsHash,
    tls_hkdf::TlsHkdf,
  },
};
use core::fmt::{Debug, Formatter};

/// Responsible for deriving keys used for encryption.
#[derive(Debug)]
pub struct KeySchedule {
  cipher_suite: CipherSuite,
  common_hkdf: TlsHkdf,
  common_secret: ArrayVectorU8<u8, MAX_HASH_LEN>,
  read: KeyScheduleRead,
  write: KeyScheduleWrite,
}

impl KeySchedule {
  pub(crate) fn from_cipher_suite(cipher_suite: CipherSuite) -> Self {
    let common_secret = zeroed_hash(cipher_suite.hash_len());
    let counter = 0;
    let iv = [0; IV_LEN];
    Self {
      cipher_suite,
      common_hkdf: cipher_suite.hkdf_extract(None, &[]),
      common_secret,
      read: KeyScheduleRead {
        state: KeyScheduleState {
          counter,
          cipher_key: ArrayVectorU8::new(),
          cipher_suite,
          iv,
          traffic_secret: cipher_suite.hkdf_extract(None, &[]),
        },
        transcript_hash: ArrayVectorU8::new(),
      },
      write: KeyScheduleWrite {
        binder_key: cipher_suite.hkdf_extract(None, &[]),
        state: KeyScheduleState {
          counter,
          cipher_key: ArrayVectorU8::new(),
          cipher_suite,
          iv,
          traffic_secret: cipher_suite.hkdf_extract(None, &[]),
        },
      },
    }
  }

  pub(crate) fn cipher_suite(&self) -> CipherSuite {
    self.cipher_suite
  }

  pub(crate) fn early_secret(&mut self, psk: Option<(&[u8], PskTy)>) -> crate::Result<()> {
    if let Some(el) = psk {
      self.hkdf_extract(el.0);
      let label = match el.1 {
        PskTy::External => b"ext binder",
        PskTy::Resumption => b"res binder",
      };
      let mut context_buffer = ArrayVectorU8::<_, MAX_HASH_LEN>::new();
      context_buffer.extend_from_copyable_slice(self.cipher_suite.hash_digest([]).lease())?;
      let binder_key = derive_secret(
        self.cipher_suite,
        Some(context_buffer.as_slice()),
        label,
        &self.common_hkdf,
      )?;
      self.write.binder_key = self.cipher_suite.hkdf_from_prk(&binder_key)?;
    } else {
      self.hkdf_extract(zeroed_hash(self.cipher_suite.hash_len()).as_slice());
    }
    self.common_secret = derive_secret_derived(self.cipher_suite, &self.common_hkdf)?;
    Ok(())
  }

  pub(crate) fn handshake_secret(&mut self, ikm: &[u8]) -> crate::Result<()> {
    self.hkdf_extract(ikm);
    self.calculate_traffic_secrets(b"c hs traffic", b"s hs traffic")?;
    self.common_secret = derive_secret_derived(self.cipher_suite, &self.common_hkdf)?;
    Ok(())
  }

  pub(crate) fn into_split(self) -> (KeyScheduleRead, KeyScheduleWrite) {
    (self.read, self.write)
  }

  pub(crate) fn master_secret(&mut self) -> crate::Result<()> {
    self.hkdf_extract(zeroed_hash(self.cipher_suite.hash_len()).as_slice());
    self.calculate_traffic_secrets(b"c ap traffic", b"s ap traffic")?;
    Ok(())
  }

  pub(crate) fn read_mut(&mut self) -> &mut KeyScheduleRead {
    &mut self.read
  }

  pub(crate) fn set_cipher_suite(&mut self, cipher_suite: CipherSuite) {
    self.cipher_suite = cipher_suite;
    self.read.state.cipher_suite = cipher_suite;
    self.write.state.cipher_suite = cipher_suite;
  }

  pub(crate) fn split_mut(&mut self) -> (&mut KeyScheduleRead, &mut KeyScheduleWrite) {
    (&mut self.read, &mut self.write)
  }

  pub(crate) fn write_mut(&mut self) -> &mut KeyScheduleWrite {
    &mut self.write
  }

  fn calculate_traffic_secrets(
    &mut self,
    read_label: &'static [u8],
    write_label: &'static [u8],
  ) -> crate::Result<()> {
    self.read.state.update(
      Some(&self.common_hkdf),
      write_label,
      Some(&self.read.transcript_hash),
    )?;
    self.write.state.update(
      Some(&self.common_hkdf),
      read_label,
      Some(&self.read.transcript_hash),
    )?;
    Ok(())
  }

  fn hkdf_extract(&mut self, ikm: &[u8]) {
    let hkdf = self.cipher_suite.hkdf_extract(Some(self.common_secret.as_ref()), ikm);
    self.common_hkdf = hkdf;
  }
}

impl Default for KeySchedule {
  #[inline]
  fn default() -> Self {
    Self::from_cipher_suite(CipherSuite::default())
  }
}
/// Data received from the server, not data used by server.
pub(crate) struct KeyScheduleRead {
  state: KeyScheduleState,
  transcript_hash: ArrayVectorU8<u8, MAX_HASH_LEN>,
}

impl KeyScheduleRead {
  pub(crate) const fn state_mut(&mut self) -> &mut KeyScheduleState {
    &mut self.state
  }
}

impl Debug for KeyScheduleRead {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("KeyScheduleRead").finish()
  }
}

pub(crate) struct KeyScheduleState {
  counter: u64,
  cipher_key: ArrayVectorU8<u8, MAX_CIPHER_KEY_LEN>,
  cipher_suite: CipherSuite,
  iv: [u8; IV_LEN],
  traffic_secret: TlsHkdf,
}

impl KeyScheduleState {
  pub(crate) fn create_finished_verify_data(
    &self,
    hash: &[u8],
  ) -> crate::Result<ArrayVectorU8<u8, MAX_HASH_LEN>> {
    let hash_len = self.cipher_suite.hash_len();
    let key = hkdf_expand_label::<MAX_HASH_LEN>(None, b"finished", hash_len, &self.traffic_secret)?;
    let mut rslt = ArrayVectorU8::new();
    match self.cipher_suite.hmac_from_key(&key)? {
      Either::Left(mut el) => {
        el.update(hash);
        rslt.extend_from_copyable_slice(&el.digest())?;
      }
      Either::Right(mut el) => {
        el.update(hash);
        rslt.extend_from_copyable_slice(&el.digest())?;
      }
    }
    Ok(rslt)
  }

  pub(crate) fn rotate(&mut self) -> crate::Result<()> {
    self.update(None, b"traffic upd", None)
  }

  pub(crate) fn verify_finished_record(
    &self,
    hash: &[u8],
    verify_data: &[u8],
  ) -> crate::Result<()> {
    let hash_len = self.cipher_suite.hash_len();
    let key = hkdf_expand_label::<MAX_HASH_LEN>(None, b"finished", hash_len, &self.traffic_secret)?;
    match self.cipher_suite.hmac_from_key(&key)? {
      Either::Left(mut el) => {
        el.update(hash);
        el.verify(verify_data)?;
      }
      Either::Right(mut el) => {
        el.update(hash);
        el.verify(verify_data)?;
      }
    }
    Ok(())
  }

  #[inline]
  pub(crate) const fn cipher_key(&self) -> &ArrayVectorU8<u8, MAX_CIPHER_KEY_LEN> {
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

  #[inline(always)]
  fn update(
    &mut self,
    hkdf: Option<&TlsHkdf>,
    label: &'static [u8],
    transcript_hash: Option<&ArrayVectorU8<u8, MAX_HASH_LEN>>,
  ) -> crate::Result<()> {
    let cipher_suite = self.cipher_suite;
    let secret = match (hkdf, transcript_hash) {
      (None, None) => derive_secret(cipher_suite, None, label, &self.traffic_secret)?,
      (None, Some(el)) => derive_secret(cipher_suite, Some(el), label, &self.traffic_secret)?,
      (Some(el), None) => derive_secret(cipher_suite, None, label, el)?,
      (Some(lhs), Some(rhs)) => derive_secret(cipher_suite, Some(rhs), label, lhs)?,
    };
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
  binder_key: TlsHkdf,
  state: KeyScheduleState,
}

impl KeyScheduleWrite {
  pub(crate) fn create_psk_binder(
    &self,
    cipher_suite: CipherSuite,
    transcript_hash: &[u8],
  ) -> crate::Result<TlsHash> {
    let hash_len = cipher_suite.hash_len();
    let key = hkdf_expand_label::<MAX_HASH_LEN>(None, b"finished", hash_len, &self.binder_key)?;
    cipher_suite.hkdf_compute([transcript_hash], &key)
  }

  pub(crate) const fn state_mut(&mut self) -> &mut KeyScheduleState {
    &mut self.state
  }
}

impl Debug for KeyScheduleWrite {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("KeyScheduleWrite").finish()
  }
}

// Differently from the RFC, this function expects a hash instead of raw bytes
fn derive_secret(
  cipher_suite: CipherSuite,
  context: Option<&[u8]>,
  label: &'static [u8],
  secret: &TlsHkdf,
) -> crate::Result<ArrayVectorU8<u8, MAX_HASH_LEN>> {
  hkdf_expand_label(context, label, cipher_suite.hash_len(), secret)
}

fn derive_secret_derived(
  cipher_suite: CipherSuite,
  secret: &TlsHkdf,
) -> crate::Result<ArrayVectorU8<u8, MAX_HASH_LEN>> {
  let mut context_buffer = ArrayVectorU8::<_, MAX_HASH_LEN>::new();
  context_buffer.extend_from_copyable_slice(cipher_suite.hash_digest([]).lease())?;
  derive_secret(cipher_suite, Some(context_buffer.as_slice()), b"derived", secret)
}

fn hkdf_expand_label<const LENGTH: usize>(
  context: Option<&[u8]>,
  label: &'static [u8],
  output_len: u8,
  secret: &TlsHkdf,
) -> crate::Result<ArrayVectorU8<u8, LENGTH>> {
  let label_len = 6u8.try_add(label.len().try_into()?)?;
  let mut concatenated = ArrayVectorU8::<_, MAX_LABEL_LEN>::new();
  match context {
    None => {
      let _ = concatenated.extend_from_copyable_slices([
        u16::from(output_len).to_be_bytes().as_slice(),
        &label_len.to_be_bytes(),
        b"tls13 ",
        label,
        &[0][..],
      ])?;
    }
    Some(value) => {
      let _ = concatenated.extend_from_copyable_slices([
        u16::from(output_len).to_be_bytes().as_slice(),
        &label_len.to_be_bytes(),
        b"tls13 ",
        label,
        u8::try_from(value.len())?.to_be_bytes().as_slice(),
        value,
      ])?;
    }
  }
  let mut output = ArrayVectorU8::from_array([0; LENGTH]);
  match secret {
    Either::Left(elem) => {
      elem.expand(concatenated.as_slice(), &mut output)?;
    }
    Either::Right(elem) => {
      elem.expand(concatenated.as_slice(), &mut output)?;
    }
  }
  output.truncate(output_len);
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

fn zeroed_hash(hash_len: u8) -> ArrayVectorU8<u8, MAX_HASH_LEN> {
  let mut hash = ArrayVectorU8::from_array([0; MAX_HASH_LEN]);
  hash.truncate(hash_len);
  hash
}
