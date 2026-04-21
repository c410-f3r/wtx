// https://datatracker.ietf.org/doc/html/rfc8446#section-7.1

use crate::{
  collection::ArrayVectorU8,
  crypto::{Hash, Hkdf},
  tls::{
    CipherSuiteTy, CurrHash, CurrHkdf, IV_LEN, MAX_CIPHER_KEY_LEN, MAX_HASH_LEN, MAX_LABEL_LEN,
    PskTy,
  },
};
use core::fmt::{Debug, Formatter};

#[derive(Debug)]
pub(crate) struct KeySchedule {
  cipher_suite_ty: CipherSuiteTy,
  common_hkdf: CurrHkdf,
  common_secret: ArrayVectorU8<u8, MAX_HASH_LEN>,
  client: KeyScheduleClient,
  server: KeyScheduleServer,
}

impl KeySchedule {
  pub(crate) fn from_cipher_suite_ty(cipher_suite_ty: CipherSuiteTy) -> Self {
    let common_secret = zeroed_hash(cipher_suite_ty.hash_len());
    Self {
      cipher_suite_ty,
      client: KeyScheduleClient {
        binder_key: CurrHkdf::extract(None, &[]).1,
        state: KeyScheduleState::default(),
      },
      common_hkdf: CurrHkdf::extract(None, &[]).1,
      common_secret,
      server: KeyScheduleServer {
        state: KeyScheduleState::default(),
        transcript_hash: ArrayVectorU8::default(),
      },
    }
  }

  pub(crate) fn early_secret(&mut self, psk: Option<(&[u8], PskTy)>) -> crate::Result<()> {
    if let Some(el) = psk {
      self.hkdf_extract(el.0);
      let label = match el.1 {
        PskTy::External => b"ext binder",
        PskTy::Resumption => b"res binder",
      };
      let mut context_buffer = ArrayVectorU8::<_, MAX_HASH_LEN>::new();
      context_buffer.extend_from_copyable_slice(CurrHash::digest(&[]).as_ref())?;
      let binder_key = derive_secret(
        self.cipher_suite_ty,
        Some(context_buffer.as_slice()),
        label,
        &self.common_hkdf,
      )?;
      self.client.binder_key = CurrHkdf::from_prk(&binder_key)?;
    } else {
      self.hkdf_extract(zeroed_hash(self.cipher_suite_ty.hash_len()).as_slice());
    }
    self.common_secret = derive_secret_derived(self.cipher_suite_ty, &self.common_hkdf)?;
    Ok(())
  }

  pub(crate) fn handshake_secret(&mut self, ikm: &[u8]) -> crate::Result<()> {
    self.hkdf_extract(ikm);
    self.calculate_traffic_secrets(b"c hs traffic", b"s hs traffic")?;
    self.common_secret = derive_secret_derived(self.cipher_suite_ty, &self.common_hkdf)?;
    Ok(())
  }

  pub(crate) fn master_secret(
    &mut self,
    server_transcript_hash: ArrayVectorU8<u8, MAX_HASH_LEN>,
  ) -> crate::Result<()> {
    self.server.transcript_hash = server_transcript_hash;
    self.hkdf_extract(zeroed_hash(self.cipher_suite_ty.hash_len()).as_slice());
    self.calculate_traffic_secrets(b"c ap traffic", b"s ap traffic")?;
    Ok(())
  }

  pub(crate) fn split_mut(&mut self) -> (&mut KeyScheduleClient, &mut KeyScheduleServer) {
    (&mut self.client, &mut self.server)
  }

  pub(crate) fn set_cipher_suite_ty(&mut self, cipher_suite_ty: CipherSuiteTy) {
    self.cipher_suite_ty = cipher_suite_ty;
  }

  fn calculate_traffic_secrets(
    &mut self,
    client_label: &[u8],
    server_label: &[u8],
  ) -> crate::Result<()> {
    self.client.state.calculate_traffic_secret(
      self.cipher_suite_ty,
      client_label,
      &self.common_hkdf,
      &self.server.transcript_hash,
    )?;
    self.server.state.calculate_traffic_secret(
      self.cipher_suite_ty,
      server_label,
      &self.common_hkdf,
      &self.server.transcript_hash,
    )?;
    Ok(())
  }

  fn hkdf_extract(&mut self, ikm: &[u8]) {
    let (_, hkdf) = CurrHkdf::extract(Some(self.common_secret.as_ref()), ikm);
    self.common_hkdf = hkdf;
  }
}

#[derive(Debug)]
pub(crate) struct KeyScheduleClient {
  binder_key: CurrHkdf,
  state: KeyScheduleState,
}

impl KeyScheduleClient {
  pub fn create_psk_binder(
    &self,
    cipher_suite_ty: CipherSuiteTy,
    transcript_hash: &[u8],
  ) -> crate::Result<()> {
    let cipher_key_len = cipher_suite_ty.hash_len();
    let key =
      hkdf_expand_label::<_, MAX_HASH_LEN>(None, b"finished", cipher_key_len, &self.binder_key)?;
    let hash = CurrHkdf::compute([transcript_hash], &key)?;
    Ok(())
  }
}

pub(crate) struct KeyScheduleServer {
  transcript_hash: ArrayVectorU8<u8, MAX_HASH_LEN>,
  state: KeyScheduleState,
}

impl Debug for KeyScheduleServer {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("KeyScheduleServer").finish()
  }
}

#[derive(Debug)]
struct KeyScheduleState {
  counter: u64,
  cipher_key: ArrayVectorU8<u8, MAX_CIPHER_KEY_LEN>,
  iv: ArrayVectorU8<u8, IV_LEN>,
  traffic_secret: CurrHkdf,
}

impl KeyScheduleState {
  fn calculate_traffic_secret(
    &mut self,
    cipher_suite_ty: CipherSuiteTy,
    label: &[u8],
    hkdf: &CurrHkdf,
    transcript_hash: &ArrayVectorU8<u8, MAX_HASH_LEN>,
  ) -> crate::Result<()> {
    let secret = derive_secret(cipher_suite_ty, Some(transcript_hash.as_ref()), label, hkdf)?;
    self.traffic_secret = CurrHkdf::from_prk(&secret)?;
    let cipher_key_len = cipher_suite_ty.cipher_key_len();
    self.cipher_key = hkdf_expand_label(None, b"key", cipher_key_len, &self.traffic_secret)?;
    self.iv = hkdf_expand_label(None, b"iv", IV_LEN.try_into()?, &self.traffic_secret)?;
    self.counter = 0;
    Ok(())
  }
}

impl Default for KeyScheduleState {
  fn default() -> Self {
    Self {
      counter: 0,
      cipher_key: ArrayVectorU8::new(),
      iv: ArrayVectorU8::new(),
      traffic_secret: CurrHkdf::extract(None, &[]).1,
    }
  }
}

// Differently from the RFC, this function expects a hash instead of raw bytes
fn derive_secret(
  cipher_suite_ty: CipherSuiteTy,
  context: Option<&[u8]>,
  label: &[u8],
  secret: &CurrHkdf,
) -> crate::Result<ArrayVectorU8<u8, MAX_HASH_LEN>> {
  hkdf_expand_label(context, label, cipher_suite_ty.hash_len(), secret)
}

fn derive_secret_derived(
  cipher_suite_ty: CipherSuiteTy,
  secret: &CurrHkdf,
) -> crate::Result<ArrayVectorU8<u8, MAX_HASH_LEN>> {
  let mut context_buffer = ArrayVectorU8::<_, MAX_HASH_LEN>::new();
  context_buffer.extend_from_copyable_slice(CurrHash::digest(&[]).as_ref())?;
  derive_secret(cipher_suite_ty, Some(context_buffer.as_slice()), b"derived", secret)
}

fn hkdf_expand_label<H, const LENGTH: usize>(
  context: Option<&[u8]>,
  label: &[u8],
  output_len: u8,
  secret: &H,
) -> crate::Result<ArrayVectorU8<u8, LENGTH>>
where
  H: Hkdf,
{
  let label_len = 6u8.wrapping_add(label.len().try_into()?);
  let mut concatenated = ArrayVectorU8::<_, MAX_LABEL_LEN>::new();
  match context {
    None => {
      concatenated.extend_from_copyable_slices([
        u16::from(output_len).to_be_bytes().as_slice(),
        &label_len.to_be_bytes(),
        b"tls13 ",
        label,
        &[0][..],
      ])?;
    }
    Some(value) => {
      concatenated.extend_from_copyable_slices([
        u16::from(output_len).to_be_bytes().as_slice(),
        &label_len.to_be_bytes(),
        b"tls13 ",
        label,
        u8::try_from(value.len())?.to_be_bytes().as_slice(),
        value,
      ])?;
    }
  }
  let mut output = ArrayVectorU8::from_array([0; _]);
  secret.expand(concatenated.as_slice(), &mut output)?;
  output.truncate(output_len);
  Ok(output)
}

fn zeroed_hash(hash_len: u8) -> ArrayVectorU8<u8, MAX_HASH_LEN> {
  let mut hash = ArrayVectorU8::from_array([0; _]);
  hash.truncate(hash_len);
  hash
}
