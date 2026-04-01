use crate::{crypto::Hash, misc::unlikely_elem};
use aws_lc_rs::digest::Context;

impl Hash for crate::crypto::Sha256DigestAwsLcRs {
  type Digest = [u8; 32];

  #[inline]
  fn digest<'data>(data: impl Iterator<Item = &'data [u8]>) -> Self::Digest {
    let mut context = Context::new(&aws_lc_rs::digest::SHA256);
    for elem in data {
      context.update(elem);
    }
    let rlst = context.finish();
    if let Ok(elem) = rlst.as_ref().try_into() { elem } else { unlikely_elem([0; 32]) }
  }
}

impl Hash for crate::crypto::Sha384DigestAwsLcRs {
  type Digest = [u8; 48];

  #[inline]
  fn digest<'data>(data: impl Iterator<Item = &'data [u8]>) -> Self::Digest {
    let mut context = Context::new(&aws_lc_rs::digest::SHA384);
    for elem in data {
      context.update(elem);
    }
    let rlst = context.finish();
    if let Ok(elem) = rlst.as_ref().try_into() { elem } else { unlikely_elem([0; 48]) }
  }
}
