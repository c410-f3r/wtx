use crate::{
  client_api_framework::{
    misc::{manage_after_sending_related, manage_before_sending_related},
    network::{
      transport::{RecievingTransport, SendingTransport, Transport, TransportParams},
      TcpParams, TransportGroup,
    },
    pkg::{Package, PkgsAux},
    Api, ClientApiFrameworkError,
  },
  misc::Lease,
};
use core::ops::Range;
use std::{
  io::{Read, Write},
  net::TcpStream,
};

impl RecievingTransport for TcpStream {
  #[inline]
  async fn recv<A, DRSR>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
  {
    let len = self.read(pkgs_aux.byte_buffer.as_mut()).map_err(Into::into)?;
    Ok(0..len)
  }
}

impl SendingTransport for TcpStream {
  #[inline]
  async fn send<A, DRSR, P>(
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
}

impl Transport for TcpStream {
  const GROUP: TransportGroup = TransportGroup::TCP;
  type Params = TcpParams;
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
  T: Transport,
{
  pkgs_aux.byte_buffer.clear();
  manage_before_sending_related(pkg, pkgs_aux, &mut *trans).await?;
  let mut slice = pkgs_aux.byte_buffer.lease();
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
  pkgs_aux.byte_buffer.extend_from_iter((0..pkgs_aux.byte_buffer.capacity()).map(|_| 0))?;
  manage_after_sending_related(pkg, pkgs_aux).await?;
  if everything_was_sent {
    Ok(())
  } else {
    Err(A::Error::from(ClientApiFrameworkError::CouldNotSendTheFullRequestData.into()))
  }
}

#[cfg(all(feature = "_async-tests", test))]
mod tests {
  use crate::{
    client_api_framework::{
      network::{
        transport::{
          tests::{_Ping, _PingPong, _Pong},
          SendingRecievingTransport,
        },
        TcpParams,
      },
      pkg::PkgsAux,
    },
    misc::sleep,
    tests::_uri,
  };
  use core::time::Duration;
  use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
  };

  #[tokio::test(flavor = "multi_thread")]
  async fn tcp() {
    let uri_client = _uri();
    let uri_server = uri_client.to_string();
    let _server = tokio::spawn(async move {
      let tcp_listener = TcpListener::bind(uri_server.hostname_with_implied_port()).unwrap();
      let mut buffer = [0; 8];
      let (mut stream, _) = tcp_listener.accept().unwrap();
      let idx = stream.read(&mut buffer).unwrap();
      stream.write_all(&buffer[..idx]).unwrap();
    });
    sleep(Duration::from_millis(100)).await.unwrap();
    let mut pa = PkgsAux::from_minimum((), (), TcpParams::from_uri(uri_client.as_str()));
    let mut trans = TcpStream::connect(uri_client.hostname_with_implied_port()).unwrap();
    let res = trans.send_recv_decode_contained(&mut _PingPong(_Ping, ()), &mut pa).await.unwrap();
    assert_eq!(res, _Pong("pong"));
  }
}
