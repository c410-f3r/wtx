/// Maps data of arbitrary size into a fixed-size value.
pub trait Hash {
  /// Array
  type Array: AsRef<[u8]>;

  /// Computes the hash digest of the given `data` and writes the resulting
  /// fixed-size output into `buffer`.
  fn digest(data: &[u8]) -> Self::Array;
}

impl Hash for () {
  type Array = [u8; 0];

  #[inline]
  fn digest(_: &[u8]) -> Self::Array {
    [0; 0]
  }
}

#[cfg(feature = "aws-lc-rs")]
mod aws_lc_rs {
  use crate::{crypto::Hash, misc::unlikely_elem};

  impl Hash for crate::crypto::Sha256DigestAwsLcRs {
    type Array = [u8; 32];

    #[inline]
    fn digest(data: &[u8]) -> Self::Array {
      let rlst = aws_lc_rs::digest::digest(&aws_lc_rs::digest::SHA256, data);
      if let Ok(elem) = rlst.as_ref().try_into() { elem } else { unlikely_elem([0; 32]) }
    }
  }

  impl Hash for crate::crypto::Sha384DigestAwsLcRs {
    type Array = [u8; 48];

    #[inline]
    fn digest(data: &[u8]) -> Self::Array {
      let rlst = aws_lc_rs::digest::digest(&aws_lc_rs::digest::SHA384, data);
      if let Ok(elem) = rlst.as_ref().try_into() { elem } else { unlikely_elem([0; 48]) }
    }
  }
}

#[cfg(feature = "sha2")]
mod rust_crypto {
  use crate::crypto::Hash;

  impl Hash for sha2::Sha256 {
    type Array = [u8; 32];

    #[inline]
    fn digest(data: &[u8]) -> Self::Array {
      <sha2::Sha256 as sha2::Digest>::digest(data).into()
    }
  }

  impl Hash for sha2::Sha384 {
    type Array = [u8; 48];

    #[inline]
    fn digest(data: &[u8]) -> Self::Array {
      <sha2::Sha384 as sha2::Digest>::digest(data).into()
    }
  }
}
