use crate::{
  collection::{ArrayVectorU8, IndexedStorageMut as _, Vector},
  database::{
    Identifier,
    client::{
      postgres::{
        Config, PostgresError, PostgresExecutor,
        authentication::Authentication,
        config::ChannelBinding,
        executor_buffer::ExecutorBuffer,
        message::MessageTy,
        protocol::{sasl_first, sasl_second},
      },
      rdbms::common_executor_buffer::CommonExecutorBuffer,
    },
  },
  misc::{
    ConnectionState, LeaseMut, SuffixWriterFbvm, bytes_split1, from_utf8_basic,
    net::PartitionedFilledBuffer,
  },
  rng::CryptoRng,
  stream::Stream,
};
use base64::prelude::{BASE64_STANDARD, Engine as _};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

impl<E, EB, S> PostgresExecutor<E, EB, S>
where
  EB: LeaseMut<ExecutorBuffer>,
  S: Stream,
{
  /// Connection parameters
  ///
  /// Extra parameters received from the database.
  #[inline]
  pub fn conn_params(&self) -> impl Iterator<Item = (&Identifier, &Identifier)> {
    self.eb.lease().conn_params.iter()
  }

  pub(crate) async fn manage_authentication<RNG>(
    &mut self,
    config: &Config<'_>,
    rng: &mut RNG,
    tls_server_end_point: Option<&[u8]>,
  ) -> crate::Result<()>
  where
    RNG: CryptoRng,
  {
    let ExecutorBuffer { common, .. } = self.eb.lease_mut();
    let CommonExecutorBuffer { net_buffer, .. } = common;
    let msg0 = Self::fetch_msg_from_stream(&mut self.cs, net_buffer, &mut self.stream).await?;
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
          net_buffer,
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
    let msg1 = Self::fetch_msg_from_stream(&mut self.cs, net_buffer, &mut self.stream).await?;
    if let MessageTy::Authentication(Authentication::Ok) = msg1.ty {
      Ok(())
    } else {
      Err(PostgresError::UnexpectedDatabaseMessage { received: msg1.tag }.into())
    }
  }

  pub(crate) async fn read_after_authentication_data(&mut self) -> crate::Result<()> {
    loop {
      let ExecutorBuffer { common, conn_params } = self.eb.lease_mut();
      let CommonExecutorBuffer { net_buffer, .. } = common;
      let msg = Self::fetch_msg_from_stream(&mut self.cs, net_buffer, &mut self.stream).await?;
      match msg.ty {
        MessageTy::BackendKeyData => {}
        MessageTy::ParameterStatus(name, value) => {
          let name = from_utf8_basic(name)?.try_into()?;
          let value = from_utf8_basic(value)?.try_into()?;
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
    net_buffer: &mut PartitionedFilledBuffer,
    rng: &mut RNG,
    stream: &mut S,
    tls_server_end_point: Option<&[u8]>,
  ) -> crate::Result<()>
  where
    RNG: CryptoRng,
  {
    let tsep_data = tls_server_end_point.unwrap_or_default();
    let local_nonce = nonce(rng);
    {
      let mut sw = SuffixWriterFbvm::from(net_buffer.suffix_writer());
      sasl_first(&mut sw, (method_bytes, method_header), &local_nonce)?;
      stream.write_all(sw.curr_bytes()).await?;
    }

    let (mut auth_data, response_nonce, salted_password) = {
      let msg = Self::fetch_msg_from_stream(cs, &mut *net_buffer, stream).await?;
      let MessageTy::Authentication(Authentication::SaslContinue {
        iterations,
        nonce,
        payload,
        salt,
      }) = msg.ty
      else {
        return Err(PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into());
      };
      let mut decoded_salt = [0; 128];
      let n = BASE64_STANDARD.decode_slice(salt, &mut decoded_salt)?;
      (
        {
          let mut vec = Vector::with_capacity(64)?;
          let _ =
            vec.extend_from_copyable_slices([&b"n=,r="[..], &local_nonce, &b","[..], payload])?;
          vec
        },
        ArrayVectorU8::<u8, 68>::from_copyable_slice(nonce)?,
        salted_password(iterations, decoded_salt.get(..n).unwrap_or_default(), config.password)?,
      )
    };

    {
      let mut sw = SuffixWriterFbvm::from(net_buffer.suffix_writer());
      sasl_second(
        &mut auth_data,
        &mut sw,
        method_header,
        &response_nonce,
        &salted_password,
        tsep_data,
      )?;
      stream.write_all(sw.curr_bytes()).await?;
    }

    {
      let msg = Self::fetch_msg_from_stream(cs, &mut *net_buffer, stream).await?;
      let MessageTy::Authentication(Authentication::SaslFinal(verifier_slice)) = msg.ty else {
        return Err(PostgresError::UnexpectedDatabaseMessage { received: msg.tag }.into());
      };
      let mut buffer = [0; 68];
      let idx = BASE64_STANDARD.decode_slice(verifier_slice, &mut buffer)?;
      let server_key = {
        let mut mac = Hmac::<Sha256>::new_from_slice(&salted_password)?;
        mac.update(b"Server Key");
        mac.finalize().into_bytes()
      };
      let mut mac_verifier = Hmac::<Sha256>::new_from_slice(&server_key)?;
      mac_verifier.update(&auth_data);
      mac_verifier.verify_slice(buffer.get(..idx).unwrap_or_default())?;
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

  'outer: for _ in 0..rslt.len() {
    for elem in rng.u8_16() {
      if idx >= rslt.len() {
        break 'outer;
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
    let mut hmac = Hmac::<Sha256>::new_from_slice(str.as_bytes())?;
    hmac.update(salt);
    hmac.update(&[0, 0, 0, 1]);
    hmac.finalize().into_bytes().into()
  };
  let mut salted_password = array;
  for _ in 1..len {
    let mut mac = Hmac::<Sha256>::new_from_slice(str.as_bytes())?;
    mac.update(&array);
    array = mac.finalize().into_bytes().into();
    for (sp_elem, array_elem) in salted_password.iter_mut().zip(array) {
      *sp_elem ^= array_elem;
    }
  }
  Ok(salted_password)
}
