use crate::client_api_framework::{
  dnsn::{Deserialize, Serialize},
  pkg::Package,
};
use alloc::string::String;
use core::marker::PhantomData;
#[cfg(feature = "rkyv")]
use rkyv::bytecheck;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct FooBar<EREQC, ERESC>(EREQC, (), PhantomData<ERESC>);

impl<EREQC, ERESC> FooBar<EREQC, ERESC> {
  pub(crate) fn _new(ereqc: EREQC) -> Self {
    Self(ereqc, (), PhantomData)
  }
}

impl<DRSR, EREQC, ERESC> Package<(), DRSR, ()> for FooBar<EREQC, ERESC>
where
  EREQC: Serialize<DRSR>,
  ERESC: Deserialize<DRSR>,
{
  type ExternalRequestContent = EREQC;
  type ExternalResponseContent = ERESC;
  type PackageParams = ();

  fn ext_req_content(&self) -> &Self::ExternalRequestContent {
    &self.0
  }

  fn ext_req_content_mut(&mut self) -> &mut Self::ExternalRequestContent {
    &mut self.0
  }

  fn pkg_params(&self) -> &Self::PackageParams {
    &self.1
  }

  fn pkg_params_mut(&mut self) -> &mut Self::PackageParams {
    &mut self.1
  }
}

#[allow(dead_code)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize))]
#[cfg_attr(feature = "miniserde", derive(miniserde::Serialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize))]
#[cfg_attr(feature = "rkyv", archive_attr(derive(Debug, rkyv::bytecheck::CheckBytes)))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, PartialEq)]
pub(crate) struct Foo {
  #[cfg_attr(feature = "rkyv", with(rkyv::with::RefAsBox))]
  pub(crate) foo: &'static str,
}

#[allow(dead_code)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshDeserialize, borsh::BorshSerialize))]
#[cfg_attr(feature = "miniserde", derive(miniserde::Deserialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive_attr(derive(Debug, rkyv::bytecheck::CheckBytes)))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, PartialEq)]
pub(crate) struct Bar {
  pub(crate) bar: String,
}

macro_rules! _create_dnsn_test {
  (
    $name:ident,
    ($req:ident, $res:ident),
    $drsr_ident:ident as $drsr_expr:expr,
    ($raw_ser:expr, $raw_der:expr),
    ($fmt_ser:expr, $fmt_der:expr),
  ) => {
    mod $name {
      use crate::client_api_framework::{
        data_format::{$req, $res},
        dnsn::{
          tests::{Bar, Foo, FooBar},
          $drsr_ident,
        },
        network::transport::{Mock, Transport},
        pkg::PkgsAux,
      };

      #[tokio::test]
      async fn der_and_ser_have_correct_outputs() {
        let pkgs_aux = &mut PkgsAux::from_minimum((), $drsr_expr, ());
        let mut trans = Mock::default();
        trans.push_response($raw_der);
        assert_eq!(
          trans
            .send_recv_decode_contained(&mut FooBar::<_, $res<Bar>>::_new($fmt_ser), pkgs_aux)
            .await
            .unwrap(),
          $fmt_der
        );
        trans.assert_request($raw_ser);
        trans.assert_does_not_have_non_asserted_requests();
      }
    }
  };
}
