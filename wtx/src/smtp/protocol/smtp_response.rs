use crate::{
  codec::{Decode, Encode},
  smtp::{
    smtp_cc::_SmtpCc,
    smtp_cc_wrappers::{_SmtpDecodeWrapper, _SmtpEncodeWrapper},
  },
};

pub(crate) struct _SmtpResponse {
  pub _code: u16,
  pub _esc: [u8; 3],
  pub _message: (),
}

impl<'de> Decode<'de, _SmtpCc> for _SmtpResponse {
  #[inline]
  fn decode(_: &mut _SmtpDecodeWrapper<'de>) -> crate::Result<Self> {
    Ok(Self { _code: 0, _esc: [0; 3], _message: () })
  }
}

impl Encode<_SmtpCc> for _SmtpResponse {
  #[inline]
  fn encode(&self, _: &mut _SmtpEncodeWrapper<'_>) -> crate::Result<()> {
    Ok(())
  }
}
