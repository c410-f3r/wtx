use crate::{
  codec::{Base64Alphabet, U64String, encode_base64_into_buffer},
  collections::Vector,
  crypto::{Hash as _, Hmac as _, HmacSha256Global, Sha256HashGlobal},
  database::{
    RecordValues,
    client::postgres::{Config, Oid, Postgres, PostgresEncodeWrapper, PostgresError, Ty},
  },
  misc::counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, i16_write_iter, i32_write},
};

pub(crate) fn bind<E, RV>(
  buffer: &mut Vector<u8>,
  portal: &str,
  rv: RV,
  stmt_cmd_id_array: &U64String,
) -> Result<(), E>
where
  E: From<crate::Error>,
  RV: RecordValues<Postgres<E>>,
{
  i32_write(
    CounterWriterBytesTy::IncludesLen,
    Some(b'B'),
    &mut PostgresEncodeWrapper::new(buffer),
    |local_ew| {
      let _ = local_ew.buffer().extend_from_copyable_slices(&[
        portal.as_bytes(),
        &[0],
        stmt_cmd_id_array.as_bytes(),
        &[0],
      ])?;
      let rv_len = rv.len();

      i16_write_iter(
        CounterWriterIterTy::Elements,
        (0..rv_len).map(|_| 1i16),
        None,
        local_ew,
        |elem, local_local_ew| {
          local_local_ew.buffer().extend_from_copyable_slice(&elem.to_be_bytes())?;
          Ok(())
        },
      )?;

      {
        local_ew
          .buffer()
          .extend_from_copyable_slice(&i16::try_from(rv_len).map_err(Into::into)?.to_be_bytes())?;
        let mut aux = (0usize, 0);
        let _ = rv.encode_values(
          &mut aux,
          local_ew,
          |(counter, start), local_ev| {
            *counter = counter.wrapping_add(1);
            *start = local_ev.buffer().len();
            let _rslt = local_ev.buffer().extend_from_copyable_slice(&[0; 4]);
            4
          },
          |(_, start), local_ev, is_null, elem_len| {
            let written = if is_null { -1i32 } else { i32::try_from(elem_len).unwrap_or(i32::MAX) };
            let bytes_opt = local_ev.buffer().get_mut(*start..);
            if let Some([a0, b0, c0, d0, ..]) = bytes_opt {
              let [a1, b1, c1, d1] = written.to_be_bytes();
              *a0 = a1;
              *b0 = b1;
              *c0 = c1;
              *d0 = d1;
            }
            0
          },
        )?;
        if aux.0 != rv_len {
          return Err(E::from(PostgresError::InvalidRecordValuesIterator.into()));
        }
      }

      i16_write_iter(
        CounterWriterIterTy::Elements,
        &[1i16],
        None,
        local_ew,
        |elem, local_local_sw| {
          local_local_sw.buffer().extend_from_copyable_slice(&elem.to_be_bytes())?;
          Ok(())
        },
      )?;

      Ok::<_, E>(())
    },
  )
}

pub(crate) fn describe(buffer: &mut Vector<u8>, data: &[u8], variant: u8) -> crate::Result<()> {
  i32_write(
    CounterWriterBytesTy::IncludesLen,
    Some(b'D'),
    &mut PostgresEncodeWrapper::new(buffer),
    |local_ew| {
      let _ = local_ew.buffer().extend_from_copyable_slices([&[variant][..], data, &[0]])?;
      Ok(())
    },
  )
}

pub(crate) fn encrypted_conn(sw: &mut Vector<u8>) -> crate::Result<()> {
  i32_write(
    CounterWriterBytesTy::IncludesLen,
    None,
    &mut PostgresEncodeWrapper::new(sw),
    |local_ew| {
      local_ew
        .buffer()
        .extend_from_copyable_slice(&0b0000_0100_1101_0010_0001_0110_0010_1111i32.to_be_bytes())?;
      crate::Result::Ok(())
    },
  )
}

pub(crate) fn execute(sw: &mut Vector<u8>, max_rows: i32, portal: &str) -> crate::Result<()> {
  i32_write(
    CounterWriterBytesTy::IncludesLen,
    Some(b'E'),
    &mut PostgresEncodeWrapper::new(sw),
    |local_ew| {
      let _ = local_ew.buffer().extend_from_copyable_slices([
        portal.as_bytes(),
        &[0],
        &max_rows.to_be_bytes(),
      ])?;
      crate::Result::Ok(())
    },
  )
}

pub(crate) fn initial_conn_msg(config: &Config<'_>, sw: &mut Vector<u8>) -> crate::Result<()> {
  i32_write(
    CounterWriterBytesTy::IncludesLen,
    None,
    &mut PostgresEncodeWrapper::new(sw),
    |local_ew| {
      let application_name = if config.application_name.is_empty() {
        [&[][..], &[][..], &[][..]]
      } else {
        [b"application_name\0", config.application_name.as_bytes(), &[0]]
      };
      let _ = local_ew.buffer().extend_from_copyable_slices([
        196_608i32.to_be_bytes().as_slice(),
        //
        b"user\0",
        config.user.as_bytes(),
        &[0],
        //
        b"database\0",
        config.db.as_bytes(),
        &[0],
        //
        application_name[0],
        application_name[1],
        application_name[2],
        //
        b"client_encoding\0",
        b"UTF8\0",
        b"DateStyle\0",
        b"ISO\0",
        b"TimeZone\0",
        b"UTC\0\0",
      ])?;
      crate::Result::Ok(())
    },
  )
}

pub(crate) fn parse<E, RV>(
  buffer: &mut Vector<u8>,
  rv: &RV,
  stmt_cmd: &str,
  stmt_cmd_id_array: &U64String,
) -> Result<(), E>
where
  E: From<crate::Error>,
  RV: RecordValues<Postgres<E>>,
{
  i32_write(
    CounterWriterBytesTy::IncludesLen,
    Some(b'P'),
    &mut PostgresEncodeWrapper::new(buffer),
    |local_ew| {
      let _ = local_ew.buffer().extend_from_copyable_slices(&[
        stmt_cmd_id_array.as_bytes(),
        &[0],
        stmt_cmd.as_bytes(),
        &[0],
      ])?;
      let idx = local_ew.buffer().len();
      local_ew.buffer().extend_from_copyable_slice(&[0, 0])?;
      let mut counter: i16 = 0;
      rv.walk(|_is_null, ty| {
        let oid: Oid = ty.unwrap_or(Ty::Custom(0)).into();
        local_ew.buffer().extend_from_copyable_slice(&oid.to_be_bytes())?;
        counter = counter.wrapping_add(1);
        Ok(())
      })?;
      if let Some([b0, b1, ..]) = local_ew.buffer().get_mut(idx..) {
        let [b2, b3] = counter.to_be_bytes();
        *b0 = b2;
        *b1 = b3;
      }
      Ok(())
    },
  )
}

pub(crate) fn query(cmd: &[u8], sw: &mut Vector<u8>) -> crate::Result<()> {
  i32_write(
    CounterWriterBytesTy::IncludesLen,
    Some(b'Q'),
    &mut PostgresEncodeWrapper::new(sw),
    |local_ew| {
      let _ = local_ew.buffer().extend_from_copyable_slices([cmd, &[0]])?;
      crate::Result::Ok(())
    },
  )
}

pub(crate) fn sasl_first(
  sw: &mut Vector<u8>,
  (method_bytes, method_header): (&[u8], &[u8]),
  nonce: &[u8],
) -> crate::Result<()> {
  i32_write(
    CounterWriterBytesTy::IncludesLen,
    Some(b'p'),
    &mut PostgresEncodeWrapper::new(sw),
    |local_ew| {
      let _ = local_ew.buffer().extend_from_copyable_slices([method_bytes, &[0]])?;
      i32_write(CounterWriterBytesTy::IgnoresLen, None, local_ew, |local_local_sw| {
        let _ =
          local_local_sw.buffer().extend_from_copyable_slices([method_header, b"n=,r=", nonce])?;
        crate::Result::Ok(())
      })
    },
  )
}

pub(crate) fn sasl_second(
  auth_data: &mut Vector<u8>,
  (idx, buffer): (usize, &mut Vector<u8>),
  method_header: &[u8],
  response_nonce: &[u8],
  salted_password: &[u8; 32],
  tls_server_end_point: &[u8],
) -> crate::Result<()> {
  const ALPHABET: Base64Alphabet = Base64Alphabet::Standard;
  i32_write(
    CounterWriterBytesTy::IncludesLen,
    Some(b'p'),
    &mut PostgresEncodeWrapper::new(buffer),
    |local_ew| {
      local_ew.buffer().extend_from_copyable_slice(b"c=")?;
      let _ = encode_base64_into_buffer(ALPHABET, local_ew.buffer(), method_header)?;
      let _ = encode_base64_into_buffer(ALPHABET, local_ew.buffer(), tls_server_end_point)?;
      let _ = local_ew.buffer().extend_from_copyable_slices([b",r=", response_nonce])?;

      let local_bytes = local_ew.buffer().get(idx.wrapping_add(5)..).unwrap_or_default();
      let _ = auth_data.extend_from_copyable_slices([b",", local_bytes])?;

      let client_key: [u8; 32] = {
        let mut mac = HmacSha256Global::from_key(salted_password)?;
        mac.update(b"Client Key");
        mac.finalize()
      };

      let client_signature = {
        let stored_client_key: [u8; 32] = Sha256HashGlobal::digest([client_key.as_slice()]);
        let mut hmac = HmacSha256Global::from_key(&stored_client_key)?;
        hmac.update(auth_data);
        hmac.finalize()
      };

      let mut client_proof = client_key;
      for (proof, signature) in client_proof.iter_mut().zip(client_signature) {
        *proof ^= signature;
      }
      local_ew.buffer().extend_from_copyable_slice(b",p=")?;
      let _ = encode_base64_into_buffer(ALPHABET, local_ew.buffer(), &client_proof)?;

      crate::Result::Ok(())
    },
  )?;
  Ok(())
}

pub(crate) fn sync(sw: &mut Vector<u8>) -> crate::Result<()> {
  i32_write(
    CounterWriterBytesTy::IncludesLen,
    Some(b'S'),
    &mut PostgresEncodeWrapper::new(sw),
    |_| crate::Result::Ok(()),
  )
}
