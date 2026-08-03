//! The TLS feature allows the creation of custom signers.

extern crate tokio;
extern crate wtx;

use wtx::{
  collections::Vector,
  misc::SecretContext,
  rng::{ChaCha20, CryptoRng, CryptoSeedableRng},
  tls::{SignatureScheme, TlsConfig, TlsCtx, TlsCtxSk, TlsMode},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let mut rng = ChaCha20::from_getrandom()?;
  let secret_context = SecretContext::new(&mut rng)?;

  // Secure connection with an encrypted secret key
  let _enc_sk = TlsConfig::from_keys_der([], &mut rng, (secret_context, &mut [][..]))?;
  // Unencrypted connection
  let _plaintext_ctx = TlsConfig::plaintext();
  // Secure connection with a plaintext secret key
  let _sk = TlsConfig::from_keys_der([], &mut rng, Vector::new())?;
  // Encrypted connection that does not verify certificates
  let _unverified_ctx = TlsConfig::unverified();

  // TLS behavior is up to the implementation
  let mut _top_secret_super_secure_signer = TlsConfig::new(TopSecretSuperSecureSigner)?;
  _top_secret_super_secure_signer.set_public_keys_pem(&[])?;
  Ok(())
}

#[derive(Debug)]
pub struct TopSecretSuperSecureSigner;

impl TlsCtx for TopSecretSuperSecureSigner {
  const TY: TlsMode = TlsMode::Verified;
}

impl TlsCtxSk for TopSecretSuperSecureSigner {
  type Signature = [u8; 0];

  fn sign<RNG>(
    &self,
    _: &mut Vector<u8>,
    _: &[u8],
    _: &mut RNG,
    _: SignatureScheme,
  ) -> wtx::Result<Self::Signature>
  where
    RNG: CryptoRng,
  {
    Ok([]) // Oh no...
  }
}
