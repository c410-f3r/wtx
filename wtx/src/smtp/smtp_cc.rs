use crate::{
  codec::CodecController,
  smtp::smtp_cc_wrappers::{_SmtpDecodeWrapper, _SmtpEncodeWrapper},
};

pub(crate) struct _SmtpCc;

impl CodecController for _SmtpCc {
  type DecodeWrapper<'inner, 'outer, 'misc>
    = _SmtpDecodeWrapper<'inner>
  where
    'inner: 'outer;
  type Error = crate::Error;
  type EncodeWrapper<'inner, 'outer, 'misc>
    = _SmtpEncodeWrapper<'inner>
  where
    'inner: 'outer;
}
