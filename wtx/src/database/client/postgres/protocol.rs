use crate::{
  database::{
    client::postgres::{statements::Statement, Config, Oid, Postgres},
    RecordValues,
  },
  misc::FilledBufferWriter,
};
use alloc::vec::Vec;
use base64::{engine::general_purpose::STANDARD, Engine};
use hmac::{digest::FixedOutput, Hmac, Mac};
use sha2::{Digest, Sha256};

#[inline]
pub(crate) fn bind<E, RV>(
  fbw: &mut FilledBufferWriter<'_>,
  portal: &str,
  rv: RV,
  stmt: &Statement<'_>,
  stmt_id_str: &str,
) -> Result<(), E>
where
  E: From<crate::Error>,
  RV: RecordValues<Postgres<E>>,
{
  write(fbw, true, Some(b'B'), |local_fbw| {
    local_fbw._extend_from_slices_each_c(&[portal.as_bytes(), stmt_id_str.as_bytes()]);
    let rv_len = rv.len();

    write_iter(local_fbw, (0..rv_len).map(|_| 1i16), None, |elem, local_local_fbw| {
      local_local_fbw._extend_from_slice(&elem.to_be_bytes());
      Ok(())
    })?;

    {
      local_fbw._extend_from_slice(&i16::try_from(rv_len).map_err(Into::into)?.to_be_bytes());
      let mut aux = (0usize, 0, 0);
      let _ = rv.encode_values(
        &mut aux,
        local_fbw,
        stmt.params.iter(),
        |(counter, len_before, start), local_local_fbw| {
          *counter = counter.wrapping_add(1);
          *start = local_local_fbw._len();
          local_local_fbw._extend_from_slice(&[0; 4]);
          *len_before = local_local_fbw._len();
          4
        },
        |(_, len_before, start), local_local_fbw, is_null| {
          let written = if is_null {
            -1i32
          } else {
            i32::try_from(local_local_fbw._len().wrapping_sub(*len_before)).unwrap_or(i32::MAX)
          };
          let bytes_opt = local_local_fbw._curr_bytes_mut().get_mut(*start..);
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
        return Err(crate::Error::PG_InvalidRecordValuesIterator.into());
      }
    }

    write_iter(local_fbw, &[1i16], None, |elem, local_local_fbw| {
      local_local_fbw._extend_from_slice(&elem.to_be_bytes());
      Ok(())
    })?;

    Ok::<_, E>(())
  })
}

#[inline]
pub(crate) fn describe(
  data: &str,
  fbw: &mut FilledBufferWriter<'_>,
  variant: u8,
) -> crate::Result<()> {
  write(fbw, true, Some(b'D'), |local_fbw| {
    local_fbw._extend_from_byte(variant);
    local_fbw._extend_from_slice_c(data.as_bytes());
    Ok(())
  })
}

#[inline]
pub(crate) fn encrypted_conn(fbw: &mut FilledBufferWriter<'_>) -> crate::Result<()> {
  write(fbw, true, None, |local_fbw| {
    local_fbw._extend_from_slice(&0b0000_0100_1101_0010_0001_0110_0010_1111i32.to_be_bytes());
    Ok::<_, crate::Error>(())
  })
}

#[inline]
pub(crate) fn execute(
  fbw: &mut FilledBufferWriter<'_>,
  max_rows: i32,
  portal: &str,
) -> crate::Result<()> {
  write(fbw, true, Some(b'E'), |local_fbw| {
    local_fbw._extend_from_slice_c(portal.as_bytes());
    local_fbw._extend_from_slice(&max_rows.to_be_bytes());
    Ok::<_, crate::Error>(())
  })
}

#[inline]
pub(crate) fn initial_conn_msg(
  config: &Config<'_>,
  fbw: &mut FilledBufferWriter<'_>,
) -> crate::Result<()> {
  write(fbw, true, None, |local_fbw| {
    local_fbw._extend_from_slice(&0b11_0000_0000_0000_0000i32.to_be_bytes());
    local_fbw._extend_from_slices_each_c(&[b"user", config.user.as_bytes()]);
    local_fbw._extend_from_slices_each_c(&[b"database", config.db.as_bytes()]);
    if !config.app_name.is_empty() {
      local_fbw._extend_from_slices_each_c(&[b"application_name", config.app_name.as_bytes()]);
    }
    local_fbw._extend_from_slices_each_c(&[
      b"client_encoding",
      b"UTF8",
      b"DateStyle",
      b"ISO",
      b"TimeZone",
      b"UTC",
    ]);
    local_fbw._extend_from_slice_c(b"");
    Ok::<_, crate::Error>(())
  })
}

#[inline]
pub(crate) fn parse(
  cmd: &str,
  fbw: &mut FilledBufferWriter<'_>,
  iter: impl IntoIterator<Item = Oid>,
  name: &str,
) -> crate::Result<()> {
  write(fbw, true, Some(b'P'), |local_fbw| {
    local_fbw._extend_from_slices_each_c(&[name.as_bytes(), cmd.as_bytes()]);
    write_iter(local_fbw, iter, None, |ty, local_local_fbw| {
      local_local_fbw._extend_from_slice(&ty.to_be_bytes());
      Ok(())
    })
  })
}

#[inline]
pub(crate) fn query(cmd: &[u8], fbw: &mut FilledBufferWriter<'_>) -> crate::Result<()> {
  write(fbw, true, Some(b'Q'), |local_fbw| {
    local_fbw._extend_from_slice_c(cmd);
    Ok::<_, crate::Error>(())
  })
}

#[inline]
pub(crate) fn sasl_first(
  fbw: &mut FilledBufferWriter<'_>,
  (method_bytes, method_header): (&[u8], &[u8]),
  nonce: &[u8],
) -> crate::Result<()> {
  write(fbw, true, Some(b'p'), |local_fbw| {
    local_fbw._extend_from_slice_c(method_bytes);
    write(local_fbw, false, None, |local_local_fbw| {
      local_local_fbw._extend_from_slice(method_header);
      local_local_fbw._extend_from_slice(b"n=,r=");
      local_local_fbw._extend_from_slice(nonce);
      Ok::<_, crate::Error>(())
    })
  })?;
  Ok(())
}

#[inline]
pub(crate) fn sasl_second(
  auth_data: &mut Vec<u8>,
  fbw: &mut FilledBufferWriter<'_>,
  method_header: &[u8],
  response_nonce: &[u8],
  salted_password: &[u8; 32],
  tls_server_end_point: &[u8],
) -> crate::Result<()> {
  write(fbw, true, Some(b'p'), |local_fbw| {
    local_fbw._extend_from_slice(b"c=");
    {
      let n = STANDARD.encode_slice(method_header, local_fbw._available_bytes_mut())?;
      local_fbw._shift_idx(n);
    }
    {
      let n = STANDARD.encode_slice(tls_server_end_point, local_fbw._available_bytes_mut())?;
      local_fbw._shift_idx(n);
    }
    local_fbw._extend_from_slice(b",r=");
    local_fbw._extend_from_slice(response_nonce);

    auth_data.push(b',');
    auth_data.extend(local_fbw._curr_bytes().get(5..).unwrap_or_default());

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

    {
      local_fbw._extend_from_slice(b",p=");
      let n = STANDARD.encode_slice(client_proof, local_fbw._available_bytes_mut())?;
      local_fbw._shift_idx(n);
    }

    Ok::<_, crate::Error>(())
  })?;
  Ok(())
}

#[inline]
pub(crate) fn sync(fbw: &mut FilledBufferWriter<'_>) -> crate::Result<()> {
  write(fbw, true, Some(b'S'), |_| Ok::<_, crate::Error>(()))
}

#[inline]
pub(crate) fn write<E>(
  fbw: &mut FilledBufferWriter<'_>,
  include_len: bool,
  prefix: Option<u8>,
  cb: impl FnOnce(&mut FilledBufferWriter<'_>) -> Result<(), E>,
) -> Result<(), E>
where
  E: From<crate::Error>,
{
  if let Some(elem) = prefix {
    fbw._extend_from_byte(elem);
  }
  let (len_before, start) = if include_len {
    let len = fbw._len();
    fbw._extend_from_slice(&[0; 4]);
    (len, len)
  } else {
    let start = fbw._len();
    fbw._extend_from_slice(&[0; 4]);
    (fbw._len(), start)
  };
  cb(fbw)?;
  let written = fbw._len().wrapping_sub(len_before);
  let [a1, b1, c1, d1] = i32::try_from(written).map_err(Into::into)?.to_be_bytes();
  if let Some([a0, b0, c0, d0, ..]) = fbw._curr_bytes_mut().get_mut(start..) {
    *a0 = a1;
    *b0 = b1;
    *c0 = c1;
    *d0 = d1;
  }
  Ok(())
}

#[inline]
pub(crate) fn write_iter<E, T>(
  fbw: &mut FilledBufferWriter<'_>,
  iter: impl IntoIterator<Item = T>,
  prefix: Option<u8>,
  mut cb: impl FnMut(T, &mut FilledBufferWriter<'_>) -> Result<(), E>,
) -> Result<(), E>
where
  E: From<crate::Error>,
{
  if let Some(elem) = prefix {
    fbw._extend_from_byte(elem);
  }
  let len_before = fbw._len();
  fbw._extend_from_slice(&[0; 2]);
  let mut counter: usize = 0;
  for elem in iter.into_iter().take(u16::MAX.into()) {
    cb(elem, fbw)?;
    counter = counter.wrapping_add(1);
  }
  let bytes = fbw._curr_bytes_mut();
  if let Some([a0, b0, ..]) = bytes.get_mut(len_before..) {
    let [a1, b1] = i16::try_from(counter).map_err(Into::into)?.to_be_bytes();
    *a0 = a1;
    *b0 = b1;
  }
  Ok(())
}
