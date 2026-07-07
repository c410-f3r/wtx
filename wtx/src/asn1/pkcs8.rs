// <https://www.rfc-editor.org/info/rfc5208/>

use crate::{
  asn1::{
    Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Asn1Error, INTEGER_TAG, Len, Octetstring, Oid,
    SEQUENCE_TAG, asn1_writer, decode_asn1_tlv, parse_der_from_pem_range,
  },
  codec::{Decode, DecodeWrapper, Encode, EncodeWrapper, GenericCodec},
  collections::TryExtend,
  misc::{Lease, LeaseMut, Pem},
};
use core::fmt::{Debug, Formatter};

/// A standard syntax for storing private key information.
#[derive(Clone, Default, PartialEq)]
pub struct Pkcs8<B> {
  /// Version
  pub version: u8,
  /// Identifies the private-key algorithm
  pub private_key_algorithm: Oid,
  /// Bytes whose contents are the value of the private key
  pub private_key: Octetstring<B>,
}

impl<'this, B> Pkcs8<B>
where
  B: Lease<[u8]>,
{
  /// From PEM data
  #[inline]
  pub fn from_pem<BUF>(
    buffer: &'this mut BUF,
    bytes: &[u8],
  ) -> crate::Result<(Self, &'this [u8], Asn1DecodeWrapperAux)>
  where
    B: TryFrom<&'this [u8]>,
    BUF: LeaseMut<[u8]> + TryExtend<(u8, usize)>,
    <B as TryFrom<&'this [u8]>>::Error: Into<crate::Error>,
  {
    let pem = Pem::decode(&mut DecodeWrapper::new(bytes, &mut *buffer))?;
    let slice: &'this [u8] = <BUF as Lease<[u8]>>::lease(buffer);
    parse_der_from_pem_range(slice, &pem)
  }
}

impl<'de, B> Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>> for Pkcs8<B>
where
  B: Lease<[u8]> + TryFrom<&'de [u8]>,
  B::Error: Into<crate::Error>,
{
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de, Asn1DecodeWrapperAux>) -> crate::Result<Self> {
    let (SEQUENCE_TAG, _, bytes, rest) = decode_asn1_tlv(dw.bytes)? else {
      return Err(Asn1Error::InvalidPkcs8.into());
    };
    let version = {
      let Some(([INTEGER_TAG, 1, 0], local_rest)) = bytes.split_at_checked(3) else {
        return Err(Asn1Error::InvalidPkcs8.into());
      };
      dw.bytes = local_rest;
      0
    };
    let private_key_algorithm = {
      let (SEQUENCE_TAG, _, local_bytes, local_rest) = decode_asn1_tlv(dw.bytes)? else {
        return Err(Asn1Error::InvalidPkcs8.into());
      };
      dw.bytes = local_bytes;
      let private_key_algorithm = Oid::decode(dw)?;
      dw.bytes = local_rest;
      private_key_algorithm
    };
    let private_key = Octetstring::decode(dw)?;
    dw.bytes = rest;
    Ok(Self { version, private_key_algorithm, private_key })
  }
}

impl<B> Encode<GenericCodec<(), Asn1EncodeWrapperAux>> for Pkcs8<B>
where
  B: Lease<[u8]>,
{
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_, Asn1EncodeWrapperAux>) -> crate::Result<()> {
    asn1_writer(ew, Len::MAX_TWO_BYTES, SEQUENCE_TAG, |local_ew| {
      local_ew.buffer.extend_from_copyable_slice(&[INTEGER_TAG, 1, 0])?;
      asn1_writer(local_ew, Len::MAX_ONE_BYTE, SEQUENCE_TAG, |local_local_ew| {
        self.private_key_algorithm.encode(local_local_ew)
      })?;
      self.private_key.encode(local_ew)?;
      Ok(())
    })
  }
}

impl<B> Debug for Pkcs8<B> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Pkcs8").finish()
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    asn1::{Asn1DecodeWrapperAux, Asn1EncodeWrapperAux, Pkcs8},
    codec::{Decode, DecodeWrapper, Encode, EncodeWrapper},
    collections::Vector,
  };

  const ED25519: &str = "-----BEGIN PRIVATE KEY-----\n\
  MC4CAQAwBQYDK2VwBCIEIKCH5Uj5jiM8UTLbtRVxc4Loehk1PJxdqLzJNFc/ReB2\n\
  -----END PRIVATE KEY-----";

  const RSA: &str = "-----BEGIN PRIVATE KEY-----\n\
  MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCycMGOpSukVVQ9\n\
  6R40WJJfw2HAAtwLZcG89jliPMaOHru3cl/euS6KNbFs87wJjH+zcJUFJRKzFerN\n\
  dCzIXSpZ/6uCNeN1jiTh1bYi2UrGmXzYPGiNWmRv72VHhQkspWNNNdQhX1wnSf8E\n\
  +upStE+f3ZHxK1OizpQ1WxBJF48w+rZd/nNj90JCbaaClBONoI/+ZxgCLw6rJWzN\n\
  J7FmXOThEdSmxwoJF/1fBabJy9HgLwPghlUjTALv/dqxHbcmIlJQcJHEhbWbVfcy\n\
  opzeB0fRZ7TyW2H97+2f5H+/ReD70Oa+zlkABWm6P04ErApqJPI7/HaRTLXFWeyw\n\
  FcXyvsSZAgMBAAECggEAJ4C/0ODu9uvvAN+1UJhVGz8pSjU32owV5kvKK13SBQ93\n\
  tiZrY+ayD2XQmBKjS6ffc22Wh/OLnrrY5s/zxA2f/RmVMffVGaa0sow5zKA3Jh0/\n\
  nq1M5hIfTwp77OfePpSElci3ZAX05DvE6ajUrCd/wx/tmariUpYSCHfW9J9zEz/m\n\
  8pW/QVOQsz1wsKKqDeRnAqKm0fbn0l2tsJ/3OVf679oVckgpR7ebQZM+R/Q9wSr6\n\
  5vMhf7JFsihOnLzdo6ism4DgUJY+hXk4vUtafstW2WJ2Rew002q2BiVPJQi2H276\n\
  0TzUA57kWzcQNNPc8y9/VCyUbn/IyC8XYA3u5d/dwwKBgQDWV8wDR4gVHpEy5e6T\n\
  +Rk2HJgqKQfVLZdn1ItTJp5ElDcgsMWXmz0t79VWhvxr24H63NxkJ+3NanG3MWL8\n\
  LGofXE4PIQ+YNc9VDS3fJRkhj5PcCjcM4C2G0k/TNfPLt30vYe9+oHaG6D/ltWEh\n\
  HElq8/RdeCFSZ7jWCDVVBKRJmwKBgQDVHrPHAHGlaSHb2O6/XYELd8BPpsg2ADS1\n\
  iZ9BeUEdEdvD5YhFIM+CXgm3zmFe2OUb91IY81N02abEzqMxwE55sEptFgT3KAnI\n\
  nJwjC5KfiPCEcWtHKDIV527xtznoKgaNDjLPvkAzBeI7rddZt+4rgfGODkCZ4UeR\n\
  AJ/w+SK32wKBgQCUGdgCUBOsHBHRrGQ75DtSU1Gkl/MsjjL2cDrQeneTBSJOOTZe\n\
  Ocp9CiFLhzu0vthB4Qd7QMekTq9CGCLAAWRWRO4+r+ZZkpyutMuEStrhgJZ2zKwa\n\
  /m8WoAy98KKCmUcrTS0xPmiHcMRt0PTK7wOfne60AsRrbvWdFdDb7LgjjwKBgC9Z\n\
  AtfTYWw+TydoqqIZQ/IoSLFpfFGC+jLawGbraWvr68c512yEPZXZDo+najqINV5h\n\
  M/wXExOCx2ox/k+vSb//SomxuqiuXH4VTRr8FzcaVVUXXZ4RcA8tu5g3/MV3kL0F\n\
  yoQc4GZ1iC16Eb38/wzrcZ79y5xkUGIGoYIH147BAoGAdkCsrcx5yGicIR/WBcN0\n\
  UIMwi+MtTfyyZfYLCqHAAHDPl8Qjm4gGtiMhGQG+dOhX9d5rEIwLeVoMe/suV2Oh\n\
  6XQvFiWn5A1EwOyyDpTf/6+cK44bKLg9UgPdEmn47R0AE7BOTy9EPaUuWHSOzTI3\n\
  GrDDewJI1SYDD5Sj2qQcvUw=\n\
  -----END PRIVATE KEY-----";

  #[test]
  fn ed25519() {
    check_codec(ED25519);
  }

  #[test]
  fn rsa() {
    check_codec(RSA);
  }

  fn check_codec(pem: &str) {
    let mut codec_buffer = Vector::new();
    let mut pem_buffer = Vector::new();
    let mut ew = EncodeWrapper::new(&mut codec_buffer, Asn1EncodeWrapperAux::default());
    let pkcs = Pkcs8::<&[u8]>::from_pem(&mut pem_buffer, pem.as_bytes()).unwrap().0;
    pkcs.encode(&mut ew).unwrap();
    let mut dw = DecodeWrapper::new(&codec_buffer, Asn1DecodeWrapperAux::default());
    assert_eq!(Pkcs8::decode(&mut dw).unwrap(), pkcs);
  }
}
