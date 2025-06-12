use crate::{
  client_api_framework::pkg::Package,
  data_transformation::dnsn::De,
  misc::{DecodeSeq, Encode},
};
use alloc::string::String;
use core::marker::PhantomData;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct _FooBar<EREQC, ERESC>(EREQC, (), PhantomData<ERESC>);

impl<EREQC, ERESC> _FooBar<EREQC, ERESC> {
  pub(crate) fn _new(ereqc: EREQC) -> Self {
    Self(ereqc, (), PhantomData)
  }
}

impl<DRSR, EREQC, ERESC, T> Package<(), DRSR, T, ()> for _FooBar<EREQC, ERESC>
where
  EREQC: Encode<De<DRSR>>,
  ERESC: for<'de> DecodeSeq<'de, De<DRSR>>,
{
  type ExternalRequestContent = EREQC;
  type ExternalResponseContent<'de> = ERESC;
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

#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, PartialEq)]
pub(crate) struct _Foo {
  pub(crate) foo: &'static str,
}

#[cfg_attr(feature = "borsh", derive(borsh::BorshDeserialize, borsh::BorshSerialize))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, PartialEq)]
pub(crate) struct _Bar {
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
      use crate::{
        client_api_framework::{
          network::transport::{Mock, SendingReceivingTransport},
          pkg::PkgsAux,
        },
        data_transformation::{
          dnsn::{
            tests::{_Bar, _Foo, _FooBar},
            $drsr_ident,
          },
          format::{$req, $res},
        },
      };

      #[test]
      fn der_and_ser_have_correct_outputs() {
        crate::executor::Runtime::new()
          .block_on(async {
            let pkgs_aux = &mut PkgsAux::from_minimum((), $drsr_expr, ());
            let mut trans = Mock::default();
            trans.push_response($raw_der);
            assert_eq!(
              trans
                .send_pkg_recv_decode_contained(
                  &mut _FooBar::<_, $res<_Bar>>::_new($fmt_ser),
                  pkgs_aux
                )
                .await
                .unwrap(),
              $fmt_der
            );
            trans.assert_request($raw_ser);
            trans.assert_does_not_have_non_asserted_requests();
          })
          .unwrap();
      }
    }
  };
}
