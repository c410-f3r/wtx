use crate::{
  codec::{Decode, Encode},
  smtp::{
    smtp_cc::_SmtpCc,
    smtp_cc_wrappers::{_SmtpDecodeWrapper, _SmtpEncodeWrapper},
  },
};

pub(crate) struct _Ehlo {
  pub _auth_mechanisms: u64,
  pub _capabilities: u32,
  pub _deliver_by: u64,
  pub _future_release_datetime: u64,
  pub _future_release_interval: u64,
  pub _hostname: (),
  pub _size: usize,
}

impl<'de> Decode<'de, _SmtpCc> for _Ehlo {
  #[inline]
  fn decode(_: &mut _SmtpDecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self {
      _auth_mechanisms: 0,
      _capabilities: 0,
      _deliver_by: 0,
      _future_release_datetime: 0,
      _future_release_interval: 0,
      _hostname: (),
      _size: 0,
    })
  }
}

impl Encode<_SmtpCc> for _Ehlo {
  #[inline]
  fn encode(&self, _: &mut _SmtpEncodeWrapper<'_>) -> crate::Result<()> {
    Ok(())
  }
}
