use crate::{
  collection::{IndexedStorageMut, Vector},
  database::{
    RecordValues,
    client::postgres::{Config, EncodeWrapper, Oid, Postgres, PostgresError, PostgresStatement},
  },
  misc::{
    SuffixWriterFbvm,
    counter_writer::{CounterWriter, I16Counter, I32Counter},
  },
};
use base64::{Engine, engine::general_purpose::STANDARD};
use hmac::{Hmac, KeyInit, Mac, digest::FixedOutput};
use sha2::{Digest, Sha256};

pub(crate) fn bind<E, RV>(
  sw: &mut SuffixWriterFbvm<'_>,
  portal: &str,
  rv: RV,
  _: &PostgresStatement<'_>,
  stmt_cmd_id_array: &[u8],
) -> Result<(), E>
where
  E: From<crate::Error>,
  RV: RecordValues<Postgres<E>>,
{
  I32Counter::default().write(sw, true, Some(b'B'), |local_sw| {
    local_sw.extend_from_slices_each_c(&[portal.as_bytes(), stmt_cmd_id_array])?;
    let rv_len = rv.len();

    I16Counter::default().write_iter(
      local_sw,
      (0..rv_len).map(|_| 1i16),
      None,
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

    I16Counter::default().write_iter(local_sw, &[1i16], None, |elem, local_local_sw| {
      local_local_sw.extend_from_slice(&elem.to_be_bytes())?;
      Ok(())
    })?;

    Ok::<_, E>(())
  })
}

pub(crate) fn describe(
  data: &[u8],
  sw: &mut SuffixWriterFbvm<'_>,
  variant: u8,
) -> crate::Result<()> {
  I32Counter::default().write(sw, true, Some(b'D'), |local_sw| {
    local_sw.extend_from_byte(variant)?;
    local_sw.extend_from_slice_c(data)?;
    Ok(())
  })
}

pub(crate) fn encrypted_conn(sw: &mut SuffixWriterFbvm<'_>) -> crate::Result<()> {
  I32Counter::default().write(sw, true, None, |local_sw| {
    local_sw.extend_from_slice(&0b0000_0100_1101_0010_0001_0110_0010_1111i32.to_be_bytes())?;
    Ok::<_, crate::Error>(())
  })
}

pub(crate) fn execute(
  sw: &mut SuffixWriterFbvm<'_>,
  max_rows: i32,
  portal: &str,
) -> crate::Result<()> {
  I32Counter::default().write(sw, true, Some(b'E'), |local_sw| {
    local_sw.extend_from_slice_c(portal.as_bytes())?;
    local_sw.extend_from_slice(&max_rows.to_be_bytes())?;
    Ok::<_, crate::Error>(())
  })
}

pub(crate) fn initial_conn_msg(
  config: &Config<'_>,
  sw: &mut SuffixWriterFbvm<'_>,
) -> crate::Result<()> {
  I32Counter::default().write(sw, true, None, |local_sw| {
    local_sw.extend_from_slice(&0b11_0000_0000_0000_0000i32.to_be_bytes())?;
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

pub(crate) fn parse(
  cmd: &str,
  sw: &mut SuffixWriterFbvm<'_>,
  iter: impl IntoIterator<Item = Oid>,
  name: &[u8],
) -> crate::Result<()> {
  I32Counter::default().write(sw, true, Some(b'P'), |local_sw| {
    local_sw.extend_from_slices_each_c(&[name, cmd.as_bytes()])?;
    I16Counter::default().write_iter(local_sw, iter, None, |ty, local_local_sw| {
      local_local_sw.extend_from_slice(&ty.to_be_bytes())?;
      Ok(())
    })
  })
}

pub(crate) fn query(cmd: &[u8], sw: &mut SuffixWriterFbvm<'_>) -> crate::Result<()> {
  I32Counter::default().write(sw, true, Some(b'Q'), |local_sw| {
    local_sw.extend_from_slice_c(cmd)?;
    Ok::<_, crate::Error>(())
  })
}

pub(crate) fn sasl_first(
  sw: &mut SuffixWriterFbvm<'_>,
  (method_bytes, method_header): (&[u8], &[u8]),
  nonce: &[u8],
) -> crate::Result<()> {
  I32Counter::default().write(sw, true, Some(b'p'), |local_sw| {
    local_sw.extend_from_slice_c(method_bytes)?;
    I32Counter::default().write(local_sw, false, None, |local_local_sw| {
      local_local_sw.extend_from_slice(method_header)?;
      local_local_sw.extend_from_slice(b"n=,r=")?;
      local_local_sw.extend_from_slice(nonce)?;
      Ok::<_, crate::Error>(())
    })
  })?;
  Ok(())
}

pub(crate) fn sasl_second(
  auth_data: &mut Vector<u8>,
  sw: &mut SuffixWriterFbvm<'_>,
  method_header: &[u8],
  response_nonce: &[u8],
  salted_password: &[u8; 32],
  tls_server_end_point: &[u8],
) -> crate::Result<()> {
  I32Counter::default().write(sw, true, Some(b'p'), |local_sw| {
    local_sw.extend_from_slice(b"c=")?;
    local_sw.create_buffer(method_header.len().wrapping_mul(2), |slice| {
      Ok(STANDARD.encode_slice(method_header, slice)?)
    })?;
    local_sw.create_buffer(tls_server_end_point.len().wrapping_mul(2), |slice| {
      Ok(STANDARD.encode_slice(tls_server_end_point, slice)?)
    })?;
    local_sw.extend_from_slices([b",r=", response_nonce])?;

    let local_bytes = local_sw.curr_bytes().get(5..).unwrap_or_default();
    let _ = auth_data.extend_from_copyable_slices([&b","[..], local_bytes])?;

    let client_key: [u8; 32] = {
      let mut mac = Hmac::<Sha256>::new_from_slice(salted_password)?;
      mac.update(b"Client Key");
      mac.finalize().into_bytes().into()
    };

    let client_signature = {
      let stored_client_key: [u8; 32] = {
        let mut hash = Sha256::default();
        hash.update(client_key.as_slice());
        hash.finalize_fixed().into()
      };
      let mut hmac = Hmac::<Sha256>::new_from_slice(&stored_client_key)?;
      hmac.update(auth_data);
      hmac.finalize().into_bytes()
    };

    let mut client_proof = client_key;
    for (proof, signature) in client_proof.iter_mut().zip(client_signature) {
      *proof ^= signature;
    }
    local_sw.extend_from_slice(b",p=")?;
    local_sw.create_buffer(client_proof.len().wrapping_mul(2), |slice| {
      Ok(STANDARD.encode_slice(client_proof, slice)?)
    })?;

    Ok::<_, crate::Error>(())
  })?;
  Ok(())
}

pub(crate) fn sync(sw: &mut SuffixWriterFbvm<'_>) -> crate::Result<()> {
  I32Counter::default().write(sw, true, Some(b'S'), |_| Ok::<_, crate::Error>(()))
}
