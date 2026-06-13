use crate::{
  codec::{Decode as _, Encode},
  collection::Vector,
  misc::{PartitionedFilledBuffer, SuffixWriter},
  rng::CryptoRng,
  stream::Stream,
  tls::{
    CipherSuite, TlsConfig, TlsError, TlsMode, TlsStream,
    decode_wrapper::DecodeWrapper,
    encode_wrapper::EncodeWrapper,
    misc::fetch_rec_from_stream,
    protocol::{
      client_hello::ClientHello,
      handshake::{Handshake, HandshakeType},
      key_share_entry::KeyShareEntry,
      offered_psks::OfferedPsks,
      record::Record,
      record_content_type::RecordContentType,
      server_hello::ServerHello,
    },
  },
};

///
#[derive(Debug)]
pub struct TlsAcceptor<S, TM> {
  stream: S,
  tm: TM,
}

impl<S, TM> TlsAcceptor<S, TM> {
  /// All parameters are provided by the user.
  #[inline]
  pub const fn new(stream: S, tm: TM) -> Self {
    Self { stream, tm }
  }

  #[inline]
  pub fn tls_mode<_TM>(self, tm: _TM) -> TlsAcceptor<S, _TM> {
    TlsAcceptor { stream: self.stream, tm }
  }
}

impl<S, TM> TlsAcceptor<S, TM>
where
  S: Stream,
  TM: TlsMode,
{
  #[inline]
  pub async fn accept<RNG>(
    mut self,
    network_buffer: &mut PartitionedFilledBuffer,
    rng: &mut RNG,
    tls_config: &TlsConfig<'_>,
    write_buffer: &mut Vector<u8>,
  ) -> crate::Result<TlsStream<S, TM, false>>
  where
    RNG: CryptoRng,
    S: Stream,
  {
    if TM::TY.is_plain_text() {
      return Ok(TlsStream::new(self.stream, self.tm));
    }
    let client_hello = {
      let ty = fetch_rec_from_stream(network_buffer, &mut self.stream).await?.1;
      let RecordContentType::Handshake = ty else {
        return Err(TlsError::InvalidHandshake.into());
      };
      Handshake::<ClientHello<(), _>>::decode(&mut DecodeWrapper::from_bytes(
        network_buffer.current(),
      ))?
    };
    {
      let record = Record::new(
        RecordContentType::Handshake,
        Handshake {
          data: ServerHello::new(
            seek_cipher_suite(
              &client_hello.data.tls_config().cipher_suites,
              &tls_config.cipher_suites,
            )?,
            false,
            seek_keyshare(&client_hello.data.tls_config().key_shares, &tls_config.key_shares)?,
            client_hello.data.legacy_session_id().clone(),
            rng,
            seek_psk(&client_hello.data.tls_config().offered_psks, &[]),
          )?,
          msg_type: HandshakeType::ServerHello,
        },
      );
      let mut ew = EncodeWrapper::from_buffer(SuffixWriter::new(0, write_buffer));
      record.encode(&mut ew)?;
      self.stream.write_all(ew.buffer().curr_bytes()).await?;
    }

    Ok(TlsStream::new(self.stream, self.tm))
  }
}

fn seek_cipher_suite(client: &[CipherSuite], server: &[CipherSuite]) -> crate::Result<CipherSuite> {
  for elem in server {
    if client.contains(elem) {
      return Ok(*elem);
    }
  }
  Err(TlsError::ServerNoCompatibleCypherSuite.into())
}

fn seek_keyshare<'client, 'rslt, 'server>(
  client: &[KeyShareEntry<'client>],
  server: &[KeyShareEntry<'server>],
) -> crate::Result<KeyShareEntry<'rslt>>
where
  'client: 'rslt,
  'server: 'rslt,
{
  for elem in server {
    if client.contains(elem) {
      return Ok(elem.clone());
    }
  }
  Err(TlsError::ServerNoCompatibleKeyShare.into())
}

fn seek_psk(offered: &OfferedPsks, stored: &[&[u8]]) -> Option<u16> {
  offered
    .offered_psks
    .iter()
    .position(|offered_psk| stored.contains(&offered_psk.identity.identity))?
    .try_into()
    .ok()
}
