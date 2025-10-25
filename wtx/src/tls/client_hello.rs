use crate::{
  collection::{ArrayVectorU8, Vector},
  de::Encode,
  misc::{
    SuffixWriter,
    counter_writer::{CounterWriter, U16Counter},
  },
  rng::CryptoRng,
  tls::{
    TlsConfig, TlsStream,
    cipher_suite::CipherSuiteTy,
    extensions::ClientHelloExtension,
    structs::{name_type::NameType, server_name::ServerName, server_name_list::ServerNameList},
  },
};

impl<S, TB, TM> TlsStream<S, TB, TM, true> {
  #[inline]
  pub(crate) fn client_hello<RNG>(
    config: &TlsConfig<'_>,
    rng: &mut RNG,
    writer_buffer: &mut Vector<u8>,
  ) -> crate::Result<()>
  where
    RNG: CryptoRng,
  {
    let mut random = [0u8; 32];
    rng.fill_slice(&mut random);
    writer_buffer.extend_from_copyable_slices([
      0x303u16.to_be_bytes().as_slice(),
      &random,
      &[0],
      u16::try_from(config.cipher_suites.len().wrapping_mul(2))
        .unwrap_or_default()
        .to_be_bytes()
        .as_slice(),
      &{
        let mut cipher_suites = ArrayVectorU8::<_, { 2 * CipherSuiteTy::len() }>::new();
        for cipher_suite in config.cipher_suites {
          cipher_suites.extend_from_copyable_slice(&u16::from(*cipher_suite).to_be_bytes())?;
        }
        cipher_suites
      },
      &[1, 0],
    ])?;
    U16Counter::default().write(&mut SuffixWriter::new(0, writer_buffer), false, None, |sw| {
      if let Some(name) = config.server_name {
        ClientHelloExtension::ServerNameList(ServerNameList {
          names: ServerName { name_type: NameType::HostName, name },
        })
        .encode(&mut (), sw)?;
      }
      crate::Result::Ok(())
    })?;
    Ok(())
  }
}
