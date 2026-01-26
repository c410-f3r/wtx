// https://datatracker.ietf.org/doc/html/rfc8446#section-4.1.2

use crate::{
  collection::ArrayVectorU8,
  de::{Decode, Encode},
  misc::{Lease, SuffixWriterMut},
  rng::CryptoRng,
  tls::{
    MAX_KEY_SHARES_LEN, TlsError,
    cipher_suite::CipherSuiteTy,
    de::De,
    ephemeral_secret_key::EphemeralSecretKey,
    misc::{u8_chunk, u16_list},
    protocol::{
      client_hello_extensions::ClientHelloExtensions,
      protocol_version::ProtocolVersion,
      protocol_versions::SupportedVersions,
      psk_key_exchange_modes::{PskKeyExchangeMode, PskKeyExchangeModes},
    },
    tls_config::TlsConfigInner,
  },
};

#[derive(Debug)]
pub(crate) struct ClientHello<S, TC> {
  legacy_compression_methods: [u8; 2],
  legacy_session_id: ArrayVectorU8<u8, 32>,
  legacy_version: ProtocolVersion,
  psk_key_exchange_modes: Option<PskKeyExchangeModes>,
  random: [u8; 32],
  secrets: S,
  supported_versions: SupportedVersions,
  tls_config: TC,
}

impl<'any, S, TC> ClientHello<S, TC>
where
  TC: Lease<TlsConfigInner<'any>>,
{
  pub(crate) fn new<RNG>(rng: &mut RNG, secrets: S, tls_config: TC) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    Ok(Self {
      legacy_compression_methods: [1, 0],
      legacy_session_id: ArrayVectorU8::from_array({
        let mut array = [0; 32];
        rng.fill_slice(&mut array[0..4]);
        array
      }),
      legacy_version: ProtocolVersion::Tls12,
      psk_key_exchange_modes: Some(PskKeyExchangeModes::new(ArrayVectorU8::from_array([
        PskKeyExchangeMode::PskDheKe,
      ]))),
      random: {
        let mut array = [0u8; 32];
        rng.fill_slice(&mut array);
        array
      },
      secrets,
      supported_versions: SupportedVersions::new(ArrayVectorU8::from_array([
        ProtocolVersion::Tls13,
      ])),
      tls_config,
    })
  }

  pub(crate) fn legacy_session_id(&self) -> &ArrayVectorU8<u8, 32> {
    &self.legacy_session_id
  }

  pub(crate) fn tls_config(&self) -> &TlsConfigInner<'any> {
    self.tls_config.lease()
  }
}

impl<'de> Decode<'de, De> for ClientHello<(), TlsConfigInner<'de>> {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let legacy_version = ProtocolVersion::decode(dw)?;
    let random = <[u8; 32]>::decode(dw)?;
    let legacy_session_id = u8_chunk(dw, TlsError::InvalidLegacySessionId, |el| Ok(*el))?;
    let mut cipher_suites = ArrayVectorU8::new();
    u16_list(&mut cipher_suites, dw, TlsError::InvalidCipherSuite)?;
    let legacy_compression_methods = <[u8; 2]>::decode(dw)?;
    let mut client_hello_extensions = ClientHelloExtensions::decode(dw)?;
    client_hello_extensions.tls_config.cipher_suites = cipher_suites;
    Ok(Self {
      legacy_compression_methods,
      legacy_session_id: legacy_session_id.try_into()?,
      legacy_version,
      psk_key_exchange_modes: client_hello_extensions.psk_key_exchange_modes,
      random,
      secrets: client_hello_extensions.secrets,
      supported_versions: client_hello_extensions.supported_versions,
      tls_config: client_hello_extensions.tls_config,
    })
  }
}

impl<'any, 'secrets, ES, TC> Encode<De>
  for ClientHello<&'secrets ArrayVectorU8<ES, MAX_KEY_SHARES_LEN>, TC>
where
  ES: EphemeralSecretKey,
  TC: Lease<TlsConfigInner<'any>>,
{
  #[inline]
  fn encode(&self, ew: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    ew.extend_from_slices([
      u16::from(self.legacy_version).to_be_bytes().as_slice(),
      &self.random[..],
      &self.legacy_session_id,
      u16::try_from(self.tls_config.lease().cipher_suites.len().wrapping_mul(2))
        .unwrap_or_default()
        .to_be_bytes()
        .as_slice(),
      {
        let mut cipher_suites = ArrayVectorU8::<_, { 2 * CipherSuiteTy::len() }>::new();
        for cipher_suite in &self.tls_config.lease().cipher_suites {
          cipher_suites.extend_from_copyable_slice(&u16::from(*cipher_suite).to_be_bytes())?;
        }
        cipher_suites
      }
      .as_slice(),
      &self.legacy_compression_methods,
    ])?;
    ClientHelloExtensions::new(self.secrets, &self.tls_config).encode(ew)?;
    Ok(())
  }
}

fn duplicated_error(is_some: bool) -> crate::Result<()> {
  if is_some {
    return Err(TlsError::DuplicatedClientHelloParameters.into());
  }
  Ok(())
}
