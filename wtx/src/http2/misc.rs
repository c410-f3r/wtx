use crate::{http2::PAD_MASK, misc::_unlikely_elem};

pub(crate) fn trim_frame_pad(data: &mut &[u8], flags: u8) -> crate::Result<Option<u8>> {
  let mut pad_len = None;
  if flags & PAD_MASK == PAD_MASK {
    let [local_pad_len, rest @ ..] = data else {
      return _unlikely_elem(Err(crate::http2::ErrorCode::ProtocolError.into()));
    };
    let diff_opt = rest.len().checked_sub(usize::from(*local_pad_len));
    let Some(local_data) = diff_opt.and_then(|idx| data.get(..idx)) else {
      return _unlikely_elem(Err(crate::http2::ErrorCode::ProtocolError.into()));
    };
    *data = local_data;
    pad_len = Some(*local_pad_len);
  }
  Ok(pad_len)
}
