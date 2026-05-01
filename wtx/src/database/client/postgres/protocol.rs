use crate::{
  codec::{Base64Alphabet, U64String, base64_encode},
  collection::Vector,
  crypto::{Hash, Hmac, HmacSha256Global, Sha256DigestGlobal},
  database::{
    RecordValues,
    client::postgres::{Config, EncodeWrapper, Oid, Postgres, PostgresError, Ty},
  },
  misc::{
    SuffixWriterFbvm,
    counter_writer::{CounterWriterBytesTy, CounterWriterIterTy, i16_write_iter, i32_write},
  },
};

pub(crate) fn bind<E, RV>(
  sw: &mut SuffixWriterFbvm<'_>,
  portal: &str,
  rv: RV,
  stmt_cmd_id_array: &U64String,
) -> Result<(), E>
where
  E: From<crate::Error>,
  RV: RecordValues<Postgres<E>>,
{
  i32_write(CounterWriterBytesTy::IncludesLen, Some(b'B'), sw, |local_sw| {
    local_sw.extend_from_slices_each_c(&[portal.as_bytes(), stmt_cmd_id_array.as_bytes()])?;
    let rv_len = rv.len();

    i16_write_iter(
      CounterWriterIterTy::Elements,
      (0..rv_len).map(|_| 1i16),
      None,
      local_sw,
      |elem, local_local_sw| {
        local_local_sw.extend_from_slice(&elem.to_be_bytes())?;
        Ok(())
      },
    )?;

    {
      local_sw.extend_from_slice(&i16::try_from(rv_len).map_err(Into::into)?.to_be_bytes())?;
      let mut aux = (0usize, 0);
      let _ = rv.encode_values(
        &mut aux,
        &mut EncodeWrapper::new(local_sw),
        |(counter, start), local_ev| {
          *counter = counter.wrapping_add(1);
          *start = local_ev.buffer().len();
          let _rslt = local_ev.buffer().extend_from_slice(&[0; 4]);
          4
        },
        |(_, start), local_ev, is_null, elem_len| {
          let written = if is_null { -1i32 } else { i32::try_from(elem_len).unwrap_or(i32::MAX) };
          let bytes_opt = local_ev.buffer().curr_bytes_mut().get_mut(*start..);
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
      local_sw,
      |elem, local_local_sw| {
        local_local_sw.extend_from_slice(&elem.to_be_bytes())?;
        Ok(())
      },
    )?;

    Ok::<_, E>(())
  })
}

pub(crate) fn describe(
  data: &[u8],
  sw: &mut SuffixWriterFbvm<'_>,
  variant: u8,
) -> crate::Result<()> {
  i32_write(CounterWriterBytesTy::IncludesLen, Some(b'D'), sw, |local_sw| {
    local_sw.extend_from_byte(variant)?;
    local_sw.extend_from_slice_c(data)?;
    Ok(())
  })
}

pub(crate) fn encrypted_conn(sw: &mut SuffixWriterFbvm<'_>) -> crate::Result<()> {
  i32_write(CounterWriterBytesTy::IncludesLen, None, sw, |local_sw| {
    local_sw.extend_from_slice(&0b0000_0100_1101_0010_0001_0110_0010_1111i32.to_be_bytes())?;
    Ok::<_, crate::Error>(())
  })
}

pub(crate) fn execute(
  sw: &mut SuffixWriterFbvm<'_>,
  max_rows: i32,
  portal: &str,
) -> crate::Result<()> {
  i32_write(CounterWriterBytesTy::IncludesLen, Some(b'E'), sw, |local_sw| {
    local_sw.extend_from_slice_c(portal.as_bytes())?;
    local_sw.extend_from_slice(&max_rows.to_be_bytes())?;
    Ok::<_, crate::Error>(())
  })
}

pub(crate) fn initial_conn_msg(
  config: &Config<'_>,
  sw: &mut SuffixWriterFbvm<'_>,
) -> crate::Result<()> {
  i32_write(CounterWriterBytesTy::IncludesLen, None, sw, |local_sw| {
    local_sw.extend_from_slice(&196_608i32.to_be_bytes())?;
    local_sw.extend_from_slices_each_c(&[b"user", config.user.as_bytes()])?;
    local_sw.extend_from_slices_each_c(&[b"database", config.db.as_bytes()])?;
    if !config.application_name.is_empty() {
      local_sw
        .extend_from_slices_each_c(&[b"application_name", config.application_name.as_bytes()])?;
    }
    local_sw.extend_from_slices_each_c(&[
      b"client_encoding",
      b"UTF8",
      b"DateStyle",
      b"ISO",
      b"TimeZone",
      b"UTC",
    ])?;
    local_sw.extend_from_slice_c(b"")?;
    Ok::<_, crate::Error>(())
  })
}

pub(crate) fn parse<E, RV>(
  rv: &RV,
  stmt_cmd: &str,
  stmt_cmd_id_array: &U64String,
  sw: &mut SuffixWriterFbvm<'_>,
) -> Result<(), E>
where
  E: From<crate::Error>,
  RV: RecordValues<Postgres<E>>,
{
  i32_write(CounterWriterBytesTy::IncludesLen, Some(b'P'), sw, |local_sw| {
    local_sw.extend_from_slices_each_c(&[stmt_cmd_id_array.as_bytes(), stmt_cmd.as_bytes()])?;
    let idx = local_sw.len();
    local_sw.extend_from_slice(&[0, 0])?;
    let mut counter: i16 = 0;
    rv.walk(|_is_null, ty| {
      let oid: Oid = ty.unwrap_or(Ty::Custom(0)).into();
      local_sw.extend_from_slice(&oid.to_be_bytes())?;
      counter = counter.wrapping_add(1);
      Ok(())
    })?;
    if let Some([a, b, ..]) = local_sw.curr_bytes_mut().get_mut(idx..) {
      let [c, d] = counter.to_be_bytes();
      *a = c;
      *b = d;
    }
    Ok(())
  })
}

pub(crate) fn query(cmd: &[u8], sw: &mut SuffixWriterFbvm<'_>) -> crate::Result<()> {
  i32_write(CounterWriterBytesTy::IncludesLen, Some(b'Q'), sw, |local_sw| {
    local_sw.extend_from_slice_c(cmd)?;
    Ok::<_, crate::Error>(())
  })
}

pub(crate) fn sasl_first(
  sw: &mut SuffixWriterFbvm<'_>,
  (method_bytes, method_header): (&[u8], &[u8]),
  nonce: &[u8],
) -> crate::Result<()> {
  i32_write(CounterWriterBytesTy::IncludesLen, Some(b'p'), sw, |local_sw| {
    local_sw.extend_from_slice_c(method_bytes)?;
    i32_write(CounterWriterBytesTy::IgnoresLen, None, local_sw, |local_local_sw| {
      local_local_sw.extend_from_slice(method_header)?;
      local_local_sw.extend_from_slice(b"n=,r=")?;
      local_local_sw.extend_from_slice(nonce)?;
      Ok::<_, crate::Error>(())
    })
  })
}

pub(crate) fn sasl_second(
  auth_data: &mut Vector<u8>,
  sw: &mut SuffixWriterFbvm<'_>,
  method_header: &[u8],
  response_nonce: &[u8],
  salted_password: &[u8; 32],
  tls_server_end_point: &[u8],
) -> crate::Result<()> {
  i32_write(CounterWriterBytesTy::IncludesLen, Some(b'p'), sw, |local_sw| {
    local_sw.extend_from_slice(b"c=")?;
    local_sw.create_buffer(method_header.len().wrapping_mul(2), |slice| {
      Ok(base64_encode(Base64Alphabet::Standard, method_header, slice)?.len())
    })?;
    local_sw.create_buffer(tls_server_end_point.len().wrapping_mul(2), |slice| {
      Ok(base64_encode(Base64Alphabet::Standard, tls_server_end_point, slice)?.len())
    })?;
    local_sw.extend_from_slices([b",r=", response_nonce])?;

    let local_bytes = local_sw.curr_bytes().get(5..).unwrap_or_default();
    let _ = auth_data.extend_from_copyable_slices([&b","[..], local_bytes])?;

    let client_key: [u8; 32] = {
      let mut mac = HmacSha256Global::from_key(salted_password)?;
      mac.update(b"Client Key");
      mac.digest()
    };

    let client_signature = {
      let stored_client_key: [u8; 32] = Sha256DigestGlobal::digest([client_key.as_slice()]);
      let mut hmac = HmacSha256Global::from_key(&stored_client_key)?;
      hmac.update(auth_data);
      hmac.digest()
    };

    let mut client_proof = client_key;
    for (proof, signature) in client_proof.iter_mut().zip(client_signature) {
      *proof ^= signature;
    }
    local_sw.extend_from_slice(b",p=")?;
    local_sw.create_buffer(client_proof.len().wrapping_mul(2), |slice| {
      Ok(base64_encode(Base64Alphabet::Standard, &client_proof, slice)?.len())
    })?;

    Ok::<_, crate::Error>(())
  })?;
  Ok(())
}

pub(crate) fn sync(sw: &mut SuffixWriterFbvm<'_>) -> crate::Result<()> {
  i32_write(CounterWriterBytesTy::IncludesLen, Some(b'S'), sw, |_| Ok::<_, crate::Error>(()))
}
