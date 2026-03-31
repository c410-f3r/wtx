use crate::{crypto::Hash, misc::unlikely_elem};

impl Hash for crate::crypto::Sha256DigestAwsLcRs {
  type Digest = [u8; 32];

  #[inline]
  fn digest(data: &[u8]) -> Self::Digest {
    let rlst = aws_lc_rs::digest::digest(&aws_lc_rs::digest::SHA256, data);
    if let Ok(elem) = rlst.as_ref().try_into() { elem } else { unlikely_elem([0; 32]) }
  }
}

impl Hash for crate::crypto::Sha384DigestAwsLcRs {
  type Digest = [u8; 48];

  #[inline]
  fn digest(data: &[u8]) -> Self::Digest {
    let rlst = aws_lc_rs::digest::digest(&aws_lc_rs::digest::SHA384, data);
    if let Ok(elem) = rlst.as_ref().try_into() { elem } else { unlikely_elem([0; 48]) }
  }
}
