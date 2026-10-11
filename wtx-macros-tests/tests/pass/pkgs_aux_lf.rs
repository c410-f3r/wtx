//! lifetime in responses

wtx::create_packages_aux_wrapper!();

type Api = ();

#[wtx::pkg(data_format(json), id(super::Api), transport(http))]
mod pkg {
  use wtx::client_api_framework::network::transport::TransportParams;

  #[pkg::aux]
  impl<A, DRSR, TP> super::PkgsAux<A, DRSR, TP>
  where
    TP: TransportParams
  {}

  #[pkg::before_sending]
  async fn before_sending<I>(
    params: &mut FooParams<I>,
  ) -> wtx::Result<()>
  where
    I: Iterator<Item = u64>,
  {
    Ok(())
  }

  #[derive(Debug)]
  #[pkg::params]
  pub struct FooParams<I> {
    pub iterator: I,
  }

  #[pkg::req_data]
  pub(crate) type FooReq = ();

  #[pkg::res_data]
  pub(crate) type FooRes<S> = (S, i32);
}

fn main() {
  use wtx::{
    client_api_framework::misc::Pair,
    client_api_framework::network::{HttpParams, transport::SendingReceivingTransport},
    codec::format::SerdeJson,
    collections::VectorUsize,
    executor::StdExecutor,
    http::http2_client_pool::Http2ClientPoolBuilder,
    rng::{ChaCha20, CryptoSeedableRng},
    tls::TlsConfig,
  };

  fn data() -> VectorUsize<u64> {
    VectorUsize::new()
  }

  async fn _work() {
    let mut pair = Pair::new(
      PkgsAux::from_minimum((), (), HttpParams::from_uri("".into())),
      Http2ClientPoolBuilder::new(StdExecutor::default(), 1, ChaCha20::from_std_random().unwrap(), TlsConfig::plaintext()).unwrap().build(),
    );
    
    let data = data();

    for chunk in data.chunks(8) {
      let _string: (String, i32) = ().send_req_recv_res_pkg_decoded(
          &mut pair.pkgs_aux.foo().params([].into_iter()).build(),
          &mut pair.pkgs_aux,
        )
        .await
        .unwrap()
        .data;
    }

    for chunk in data.chunks(8) {
      let _str: (&str, i32) = ().send_req_recv_res_pkg_decoded(
          &mut pair.pkgs_aux.foo().params(chunk.iter().copied()).build(),
          &mut pair.pkgs_aux,
        )
        .await
        .unwrap()
        .data;
    }
  }   

  _work();
}
