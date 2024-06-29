use crate::client_api_framework::{
  misc::{manage_after_sending_related, manage_before_sending_related},
  network::{
    transport::{Transport, TransportParams},
    TcpParams, TransportGroup, UdpParams,
  },
  pkg::{Package, PkgsAux},
  Api, ClientApiFrameworkError,
};
use core::ops::Range;
use std::{
  io::{Read, Write},
  net::{TcpStream, UdpSocket},
};

impl<DRSR> Transport<DRSR> for TcpStream {
  const GROUP: TransportGroup = TransportGroup::TCP;
  type Params = TcpParams;

  #[inline]
  async fn send<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TcpParams>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, TcpParams>,
  {
    send(pkg, pkgs_aux, self, |bytes, _, trans| Ok(trans.write(bytes)?)).await
  }

  #[inline]
  async fn send_recv<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TcpParams>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
    P: Package<A, DRSR, TcpParams>,
  {
    send_recv(pkg, pkgs_aux, self, |bytes, _, trans| Ok(trans.read(bytes)?)).await
  }
}

impl<DRSR> Transport<DRSR> for UdpSocket {
  const GROUP: TransportGroup = TransportGroup::UDP;
  type Params = UdpParams;

  #[inline]
  async fn send<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, UdpParams>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, UdpParams>,
  {
    send(pkg, pkgs_aux, self, |bytes, ext_req_params, trans| {
      Ok(trans.send_to(bytes, ext_req_params.url.uri())?)
    })
    .await
  }

  #[inline]
  async fn send_recv<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, UdpParams>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
    P: Package<A, DRSR, UdpParams>,
  {
    send_recv(pkg, pkgs_aux, self, |bytes, _, trans| Ok(trans.recv(bytes)?)).await
  }
}

async fn send<A, DRSR, P, T>(
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, T::Params>,
  trans: &mut T,
  cb: impl Fn(
    &[u8],
    &<T::Params as TransportParams>::ExternalRequestParams,
    &mut T,
  ) -> crate::Result<usize>,
) -> Result<(), A::Error>
where
  A: Api,
  P: Package<A, DRSR, T::Params>,
  T: Transport<DRSR>,
{
  pkgs_aux.byte_buffer.clear();
  manage_before_sending_related(pkg, pkgs_aux, &mut *trans).await?;
  let mut slice = pkgs_aux.byte_buffer.as_slice();
  let mut everything_was_sent = false;
  for _ in 0..16 {
    let sent = cb(slice, pkgs_aux.tp.ext_req_params(), trans)?;
    if sent == slice.len() {
      everything_was_sent = true;
      break;
    }
    slice = slice.get(sent..).unwrap_or_default();
  }
  pkgs_aux.byte_buffer.clear();
  pkgs_aux.byte_buffer.extend((0..pkgs_aux.byte_buffer.capacity()).map(|_| 0));
  manage_after_sending_related(pkg, pkgs_aux).await?;
  if everything_was_sent {
    Ok(())
  } else {
    Err(A::Error::from(ClientApiFrameworkError::CouldNotSendTheFullRequestData.into()))
  }
}

async fn send_recv<A, DRSR, P, T>(
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, T::Params>,
  trans: &mut T,
  cb: impl Fn(
    &mut [u8],
    &<T::Params as TransportParams>::ExternalRequestParams,
    &mut T,
  ) -> crate::Result<usize>,
) -> Result<Range<usize>, A::Error>
where
  A: Api,
  P: Package<A, DRSR, T::Params>,
  T: Transport<DRSR>,
{
  trans.send(pkg, pkgs_aux).await?;
  let slice = pkgs_aux.byte_buffer.as_mut();
  let len = cb(slice, pkgs_aux.tp.ext_req_params(), trans)?;
  Ok(0..len)
}

#[cfg(test)]
mod tests {
  use crate::{
    client_api_framework::{
      network::{
        transport::{
          tests::{Ping, PingPong, Pong},
          Transport,
        },
        TcpParams, UdpParams,
      },
      pkg::PkgsAux,
    },
    misc::{_uri, sleep},
  };
  use core::time::Duration;
  use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream, UdpSocket},
  };

  #[tokio::test(flavor = "multi_thread")]
  async fn tcp() {
    let uri_client = _uri();
    let uri_server = uri_client.to_string();
    let _server = tokio::spawn(async move {
      let tcp_listener = TcpListener::bind(uri_server.host()).unwrap();
      let mut buffer = [0; 8];
      let (mut stream, _) = tcp_listener.accept().unwrap();
      let idx = stream.read(&mut buffer).unwrap();
      stream.write_all(&buffer[..idx]).unwrap();
    });
    sleep(Duration::from_millis(100)).await.unwrap();
    let mut pa = PkgsAux::from_minimum((), (), TcpParams::from_uri(uri_client.uri()));
    let mut trans = TcpStream::connect(uri_client.host()).unwrap();
    let res = trans.send_recv_decode_contained(&mut PingPong(Ping, ()), &mut pa).await.unwrap();
    assert_eq!(res, Pong("pong"));
  }

  #[tokio::test]
  async fn udp() {
    let addr = "127.0.0.1:12346";
    let mut pa = PkgsAux::from_minimum((), (), UdpParams::from_uri(addr));
    let mut trans = UdpSocket::bind(addr).unwrap();
    let res = trans.send_recv_decode_contained(&mut PingPong(Ping, ()), &mut pa).await.unwrap();
    assert_eq!(res, Pong("pong"));
  }
}
