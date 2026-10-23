//! X.509

#![no_main]

use wtx::{
  asn1::Asn1DecodeWrapperAux,
  codec::{Decode, DecodeWrapper, GenericCodec},
  collections::VectorUsize,
  x509::{AlgorithmIdentifier, Certificate, Crl, Pkcs8, TbsCertificate},
};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
  {
    let mut buffer = VectorUsize::new();
    let _rslt0 = Certificate::<&[u8]>::from_der(data);
    let _rslt1 = Certificate::<&[u8]>::from_pem(&mut buffer, data);
  }
  {
    let mut buffer = VectorUsize::new();
    let _rslt0 =
      Crl::<&[u8]>::decode(&mut DecodeWrapper::new(data, Asn1DecodeWrapperAux::default()));
    let _rslt1 = Crl::<&[u8]>::from_pem(&mut buffer, data);
  }
  {
    fn decode<'de, T>(data: &'de [u8])
    where
      T: Decode<'de, GenericCodec<Asn1DecodeWrapperAux, ()>>,
    {
      let _rslt = T::decode(&mut DecodeWrapper::new(data, Asn1DecodeWrapperAux::default()));
    }

    decode::<AlgorithmIdentifier<&[u8]>>(data);
    decode::<TbsCertificate<&[u8]>>(data);
  }
  {
    let mut buffer = VectorUsize::new();
    let _rslt0 =
      Pkcs8::<&[u8]>::decode(&mut DecodeWrapper::new(data, Asn1DecodeWrapperAux::default()));
    let _rslt1 = Pkcs8::<&[u8]>::from_pem(&mut buffer, data);
  }
});
