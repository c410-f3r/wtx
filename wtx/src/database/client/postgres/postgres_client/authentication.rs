use crate::{
  codec::{Base64Alphabet, base64_decode},
  collections::{ArrayVectorCopy, Vector},
  crypto::{Hmac as _, HmacSha256Global},
  database::{
    Identifier,
    client::{
      postgres::{
        Config, PostgresClient, PostgresError,
        authentication::Authentication,
        client_buffer::ClientBuffer,
        config::ChannelBinding,
        message::MessageTy,
        protocol::{sasl_first, sasl_second},
      },
      rdbms::common_client_buffer::CommonClientBuffer,
    },
  },
  misc::{bytes_split1, from_utf8_basic},
  net::{BufStreamReader, ConnectionState, Stream, StreamWriter as _},
  rng::CryptoRng,
  tls::{TlsMode, TlsServerEndPoint, TlsStream},
};

impl<E, S, TM> PostgresClient<E, S, TM>
where
  S: Stream,
  TM: TlsMode,
{
  /// Connection parameters
  ///
  /// Extra parameters received from the database.
  #[inline]
  pub fn conn_params(&self) -> impl Iterator<Item = (&Identifier, &Identifier)> {
    self.cb.conn_params.iter()
  }

  #[expect(clippy::wildcard_enum_match_arm, reason = "too many variants")]
  pub(crate) async fn manage_authentication<RNG>(
    &mut self,
    config: &Config<'_>,
    rng: &mut RNG,
    tls_server_end_point: TlsServerEndPoint,
  ) -> crate::Result<()>
  where
    RNG: CryptoRng,
  {
    let ClientBuffer { common, .. } = &mut self.cb;
    let CommonClientBuffer { read_buffer, .. } = common;
    let msg0 = Self::fetch_msg(&mut self.cs, read_buffer, &mut self.stream).await?;
    match msg0.ty {
      MessageTy::Authentication(Authentication::Ok) => {
        return Ok(());
      }
      MessageTy::Authentication(Authentication::Sasl(data)) => {
        macro_rules! scram_sha_256 {
          () => {
            b"SCRAM-SHA-256"
          };
        }
        macro_rules! scram_sha_256_plus {
          () => {
            b"SCRAM-SHA-256-PLUS"
          };
        }

        let mut is_scram = false;
        let mut is_scram_plus = false;
        for elem in bytes_split1(data, b'\0') {
          match elem {
            scram_sha_256!() => {
              is_scram = true;
            }
            scram_sha_256_plus!() => {
              is_scram_plus = true;
            }
            _ => {}
          }
        }
        let (method_bytes, method_header) = match (is_scram, is_scram_plus, config.channel_binding)
        {
          (false, false, _) => return Err(PostgresError::UnknownAuthenticationMethod.into()),
          (true, _, ChannelBinding::Disable) | (true, false, ChannelBinding::Prefer) => {
            (scram_sha_256!().as_slice(), b"n,,".as_slice())
          }
          (_, true, ChannelBinding::Prefer | ChannelBinding::Require) => {
            (scram_sha_256_plus!().as_slice(), b"p=tls-server-end-point,,".as_slice())
          }
          (false, true, ChannelBinding::Disable) => {
            return Err(PostgresError::RequiredChannel.into());
          }
          (true, false, ChannelBinding::Require) => {
            return Err(PostgresError::MissingChannel.into());
          }
        };
        Self::sasl_authenticate(
          config,
          &mut self.cs,
          (method_bytes, method_header),
          read_buffer,
          rng,
          &mut self.stream,
          tls_server_end_point,
        )
        .await?;
      }
      _ => {
        return Err(PostgresError::UnexpectedDatabaseMessage { received: msg0.tag }.into());
      }
    }
    let msg1 = Self::fetch_msg(&mut self.cs, read_buffer, &mut self.stream).await?;
    if let MessageTy::Authentication(Authentication::Ok) = msg1.ty {
      Ok(())
    } else {
      Err(PostgresError::UnexpectedDatabaseMessage { received: msg1.tag }.into())
    }
  }

  #[expect(clippy::wildcard_enum_match_arm, reason = "too many variants")]
  pub(crate) async fn read_after_authentication_data(&mut self) -> crate::Result<()> {
    loop {
      let ClientBuffer { common, conn_params } = &mut self.cb;
      let CommonClientBuffer { read_buffer, .. } = common;
      let msg = Self::fetch_msg(&mut self.cs, read_buffer, &mut self.stream).await?;
      match msg.ty {
        MessageTy::BackendKeyData => {}
        MessageTy::ParameterStatus(name_slice, value_slice) => {
          let name = from_utf8_basic(name_slice)?.try_into()?;
          let value = from_utf8_basic(value_slice)?.try_into()?;
          let _ = conn_params.insert(name, value);
        }
        MessageTy::ReadyForQuery => return Ok(()),
        _ => {
          return Err(PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into());
        }
      }
    }
  }

  // The 'null' case of `tls_server_end_point` is already handled by `method_header`, as such,
  // it is fine to use an empty slice.
  async fn sasl_authenticate<RNG>(
    config: &Config<'_>,
    cs: &mut ConnectionState,
    (method_bytes, method_header): (&[u8], &[u8]),
    read_buffer: &mut BufStreamReader,
    rng: &mut RNG,
    stream: &mut TlsStream<S, TM, true>,
    tls_server_end_point: TlsServerEndPoint,
  ) -> crate::Result<()>
  where
    RNG: CryptoRng,
  {
    let local_nonce = nonce(rng);
    {
      let mut sw = read_buffer.suffix_pusher();
      sasl_first(sw.inner_mut(), (method_bytes, method_header), &local_nonce)?;
      stream.write_all(sw.curr()).await?;
    }
    let (mut auth_data, response_nonce, salted_password) = {
      let msg = Self::fetch_msg(cs, &mut *read_buffer, stream).await?;
      let MessageTy::Authentication(Authentication::SaslContinue {
        iterations,
        nonce,
        payload,
        salt,
      }) = msg.ty
      else {
        return Err(PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into());
      };
      let mut decoded_salt_buffer = [0; 128];
      let decoded_salt = base64_decode(Base64Alphabet::Standard, salt, &mut decoded_salt_buffer)?;
      let salted_passworded = salted_password(iterations, decoded_salt, config.password)?;
      let nonce_array = ArrayVectorCopy::<u8, 68>::from_copyable_slice(nonce)?;
      (
        {
          let mut vec = Vector::with_capacity(64)?;
          let arrays = [&b"n=,r="[..], &local_nonce, &b","[..], payload];
          let _ = vec.extend_from_copyable_slices(arrays)?;
          vec
        },
        nonce_array,
        salted_passworded,
      )
    };
    {
      let mut sw = read_buffer.suffix_pusher();
      sasl_second(
        &mut auth_data,
        (sw.idx(), sw.inner_mut()),
        method_header,
        &response_nonce,
        &salted_password,
        &tls_server_end_point,
      )?;
      stream.write_all(sw.curr()).await?;
    }
    {
      let msg = Self::fetch_msg(cs, &mut *read_buffer, stream).await?;
      let MessageTy::Authentication(Authentication::SaslFinal(verifier_slice)) = msg.ty else {
        return Err(PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into());
      };
      let mut decoded_buffer = [0; 68];
      let decoded = base64_decode(Base64Alphabet::Standard, verifier_slice, &mut decoded_buffer)?;
      let server_key = {
        let mut mac = HmacSha256Global::from_key(&salted_password)?;
        mac.update(b"Server Key");
        mac.finalize()
      };

      let mut mac_verifier = HmacSha256Global::from_key(&server_key[..])?;
      mac_verifier.update(&auth_data);
      mac_verifier.verify(decoded)?;
    }
    Ok(())
  }
}

fn nonce<RNG>(rng: &mut RNG) -> [u8; 24]
where
  RNG: CryptoRng,
{
  let mut rslt = [0; 24];
  let mut idx = 0;
  while idx < rslt.len() {
    for elem in rng.u8_16() {
      if idx >= rslt.len() {
        break;
      }
      let has_valid_char = matches!(elem, b'\x21'..=b'\x2b' | b'\x2d'..=b'\x7e');
      if let (true, Some(byte)) = (has_valid_char, rslt.get_mut(idx)) {
        *byte = elem;

        idx = idx.wrapping_add(1);
      }
    }
  }
  rslt
}

fn salted_password(len: u32, salt: &[u8], str: &str) -> crate::Result<[u8; 32]> {
  let mut array: [u8; 32] = {
    let mut hmac = HmacSha256Global::from_key(str.as_bytes())?;
    hmac.update(salt);
    hmac.update(&[0, 0, 0, 1]);
    hmac.finalize()
  };
  let mut salted_password = array;
  for _ in 1..len {
    let mut mac = HmacSha256Global::from_key(str.as_bytes())?;
    mac.update(&array);
    array = mac.finalize();
    for (sp_elem, array_elem) in salted_password.iter_mut().zip(array) {
      *sp_elem ^= array_elem;
    }
  }
  Ok(salted_password)
}
