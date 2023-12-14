use crate::{
  database::client::postgres::{
    executor_buffer::{ExecutorBuffer, ExecutorBufferPartsMut},
    password, sasl_first, sasl_second, Authentication, Config, Executor, MessageTy,
  },
  misc::{FilledBufferWriter, PartitionedFilledBuffer, Stream, _from_utf8_basic_rslt},
  rng::Rng,
};
use alloc::vec::Vec;
use arrayvec::{ArrayString, ArrayVec};
use base64::prelude::{Engine as _, BASE64_STANDARD};
use core::borrow::BorrowMut;
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use sha2::Sha256;

impl<EB, S> Executor<EB, S>
where
  EB: BorrowMut<ExecutorBuffer>,
  S: Stream,
{
  pub(crate) async fn manage_authentication<RNG>(
    &mut self,
    config: &Config<'_>,
    rng: &mut RNG,
    tls_server_end_point: Option<&[u8]>,
  ) -> crate::Result<()>
  where
    RNG: Rng,
  {
    let ExecutorBufferPartsMut { nb, .. } = self.eb.borrow_mut().parts_mut();
    let msg0 = Self::fetch_msg_from_stream(nb, &mut self.stream).await?;
    match msg0.ty {
      MessageTy::Authentication(Authentication::Md5Password(salt)) => {
        let hashed = {
          let mut md5 = Md5::new();
          md5.update(config.password);
          md5.update(config.user);
          let output = md5.finalize_reset();
          md5.update(
            ArrayString::<{ 16 * 2 }>::try_from(format_args!("{output:x}"))
              .map_err(|err| crate::Error::from(err.simplify()))?
              .as_str(),
          );
          md5.update(salt);
          ArrayString::<{ 16 * 2 + 3 }>::try_from(format_args!("md5{:x}", md5.finalize()))
            .map_err(|err| crate::Error::from(err.simplify()))?
        };
        let mut fbw = FilledBufferWriter::from(&mut *nb);
        password(&mut fbw, &hashed)?;
        self.stream.write_all(fbw._curr_bytes()).await?;
      }
      MessageTy::Authentication(Authentication::Ok) => {
        return Ok(());
      }
      MessageTy::Authentication(Authentication::Sasl(data)) => {
        let mut has_sasl_plus = false;
        for elem in data.split(|byte| *byte == b'\0') {
          if elem == b"SCRAM-SHA-256-PLUS" {
            has_sasl_plus = true;
          }
        }
        if !has_sasl_plus {
          return Err(crate::Error::UnknownAuthenticationMethod);
        }
        Self::sasl_authenticate(config, nb, rng, &mut self.stream, tls_server_end_point).await?;
      }
      MessageTy::Authentication(_)
      | MessageTy::BackendKeyData(..)
      | MessageTy::BindComplete
      | MessageTy::CloseComplete
      | MessageTy::CommandComplete(_)
      | MessageTy::CopyData
      | MessageTy::CopyDone
      | MessageTy::CopyInResponse
      | MessageTy::CopyOutResponse
      | MessageTy::DataRow(..)
      | MessageTy::EmptyQueryResponse
      | MessageTy::NoData
      | MessageTy::NoticeResponse
      | MessageTy::NotificationResponse
      | MessageTy::ParameterDescription(_)
      | MessageTy::ParameterStatus(..)
      | MessageTy::ParseComplete
      | MessageTy::PortalSuspended
      | MessageTy::ReadyForQuery
      | MessageTy::RowDescription(_) => {
        return Err(crate::Error::UnexpectedDatabaseMessage { received: msg0.tag });
      }
    }
    let msg1 = Self::fetch_msg_from_stream(nb, &mut self.stream).await?;
    if let MessageTy::Authentication(Authentication::Ok) = msg1.ty {
      Ok(())
    } else {
      Err(crate::Error::UnexpectedDatabaseMessage { received: msg1.tag })
    }
  }

  pub(crate) async fn read_after_authentication_data(&mut self) -> crate::Result<()> {
    loop {
      let ExecutorBufferPartsMut { nb, params, .. } = self.eb.borrow_mut().parts_mut();
      let msg = Self::fetch_msg_from_stream(nb, &mut self.stream).await?;
      match msg.ty {
        MessageTy::BackendKeyData(process_id, secret_key) => {
          self.process_id = process_id;
          self.secret_key = secret_key;
        }
        MessageTy::ParameterStatus(name, value) => {
          params.insert(
            params.partition_point(|(local_name, _)| local_name.as_bytes() < name),
            (_from_utf8_basic_rslt(name)?.try_into()?, _from_utf8_basic_rslt(value)?.try_into()?),
          );
        }
        MessageTy::ReadyForQuery => return Ok(()),
        _ => {
          return Err(crate::Error::UnexpectedDatabaseMessage { received: msg.tag });
        }
      }
    }
  }

  async fn sasl_authenticate<RNG>(
    config: &Config<'_>,
    nb: &mut PartitionedFilledBuffer,
    rng: &mut RNG,
    stream: &mut S,
    tls_server_end_point: Option<&[u8]>,
  ) -> crate::Result<()>
  where
    RNG: Rng,
  {
    let tsep_data = tls_server_end_point.ok_or(crate::Error::StreamDoesNotSupportTlsChannels)?;
    let local_nonce = nonce(rng);

    {
      let mut fbw = FilledBufferWriter::from(&mut *nb);
      sasl_first(&mut fbw, &local_nonce)?;
      stream.write_all(fbw._curr_bytes()).await?;
    }

    let (mut auth_data, response_nonce, salted_password) = {
      let msg = Self::fetch_msg_from_stream(&mut *nb, stream).await?;
      let MessageTy::Authentication(Authentication::SaslContinue {
        iterations,
        nonce,
        payload,
        salt,
      }) = msg.ty
      else {
        return Err(crate::Error::UnexpectedDatabaseMessage { received: msg.tag });
      };
      let mut decoded_salt = [0; 128];
      let n = BASE64_STANDARD.decode_slice(salt, &mut decoded_salt)?;
      (
        {
          let mut vec = Vec::with_capacity(64);
          vec.extend(b"n=,r=");
          vec.extend(local_nonce);
          vec.push(b',');
          vec.extend(payload);
          vec
        },
        ArrayVec::<u8, 68>::try_from(nonce)?,
        salted_password(iterations, decoded_salt.get(..n).unwrap_or_default(), config.password)?,
      )
    };

    {
      let mut fbw = FilledBufferWriter::from(&mut *nb);
      sasl_second(&mut auth_data, &mut fbw, &response_nonce, &salted_password, tsep_data)?;
      stream.write_all(fbw._curr_bytes()).await?;
    }

    {
      let msg = Self::fetch_msg_from_stream(&mut *nb, stream).await?;
      let MessageTy::Authentication(Authentication::SaslFinal(verifier_slice)) = msg.ty else {
        return Err(crate::Error::UnexpectedDatabaseMessage { received: msg.tag });
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
  RNG: Rng,
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
