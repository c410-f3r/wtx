use crate::http::{
  AutoStream, ManualStream, Method, ReqResBuffer, Request, Response, StatusCode,
  server_framework::{
    ConnAux, Middleware, Router, ServerFramework, ServerFrameworkBuilder, StateClean, StreamAux,
    endpoint::Endpoint, get,
  },
};
use core::{
  net::{IpAddr, Ipv4Addr},
  ops::ControlFlow,
};

#[test]
fn compiles() {
  async fn one(_: StateClean<'_, (), (), ReqResBuffer>) -> crate::Result<StatusCode> {
    Ok(StatusCode::Ok)
  }

  async fn two(_: StateClean<'_, (), (), ReqResBuffer>) -> crate::Result<StatusCode> {
    Ok(StatusCode::Ok)
  }

  async fn three(_: ManualStream<(), (), ()>) -> crate::Result<()> {
    Ok(())
  }

  let router = Router::paths(paths!(
    ("/aaa", Router::paths(paths!(("/bbb", get(one)), ("/ccc", get(two)))).unwrap()),
    ("/ddd", get(one)),
    ("/eee", get(two)),
    ("/fff", Router::paths(paths!(("/ggg", get(one)))).unwrap()),
    ("/hhh", get(three)),
  ))
  .unwrap();

  let _sf = ServerFrameworkBuilder::new((), router).without_aux();
}

// /aaa ->   /bbb ->  /ccc
//      \         \
//       \         -> /ddd
//        -> /eee
//
// /fff
#[tokio::test]
async fn nested_middlewares() {
  struct Counter(u8);

  impl ConnAux for Counter {
    type Init = ();

    fn conn_aux(_: Self::Init) -> crate::Result<Self> {
      Ok(Self(0))
    }
  }

  impl StreamAux for Counter {
    type Init = ();

    fn stream_aux(_: Self::Init) -> crate::Result<Self> {
      Ok(Self(0))
    }
  }

  struct CounterMw;

  impl Middleware<Counter, crate::Error, Counter> for CounterMw {
    type Aux = ();

    fn aux(&self) -> Self::Aux {}

    async fn req(
      &self,
      conn_aux: &mut Counter,
      _: &mut Self::Aux,
      _: &mut Request<ReqResBuffer>,
      stream_aux: &mut Counter,
    ) -> crate::Result<ControlFlow<StatusCode, ()>> {
      conn_aux.0 += 3;
      stream_aux.0 += 3;
      Ok(ControlFlow::Continue(()))
    }

    async fn res(
      &self,
      conn_aux: &mut Counter,
      _: &mut Self::Aux,
      _: Response<&mut ReqResBuffer>,
      stream_aux: &mut Counter,
    ) -> crate::Result<ControlFlow<StatusCode, ()>> {
      conn_aux.0 += 7;
      stream_aux.0 += 7;
      Ok(ControlFlow::Continue(()))
    }
  }

  async fn add11(
    state: StateClean<'_, Counter, Counter, ReqResBuffer>,
  ) -> crate::Result<StatusCode> {
    assert_eq!(state.conn_aux.0, 6);
    assert_eq!(state.stream_aux.0, 6);
    state.conn_aux.0 += 11;
    state.stream_aux.0 += 11;
    Ok(StatusCode::Ok)
  }

  async fn add12(mut state: ManualStream<Counter, (), Counter>) -> crate::Result<()> {
    state.conn_aux.0 += 13;
    state.stream_aux.0 += 13;
    Ok(())
  }

  async fn add13(
    state: StateClean<'_, Counter, Counter, ReqResBuffer>,
  ) -> crate::Result<StatusCode> {
    state.conn_aux.0 += 15;
    state.stream_aux.0 += 15;
    Ok(StatusCode::Ok)
  }

  async fn add14(
    state: StateClean<'_, Counter, Counter, ReqResBuffer>,
  ) -> crate::Result<StatusCode> {
    assert_eq!(state.conn_aux.0, 3);
    assert_eq!(state.stream_aux.0, 3);
    state.conn_aux.0 += 17;
    state.stream_aux.0 += 17;
    Ok(StatusCode::Ok)
  }

  let router = Router::new(
    paths!(
      (
        "/aaa",
        Router::new(
          paths!(
            (
              "/bbb",
              Router::new(paths!(("/ccc", get(add11)), ("/ddd", get(add12))), CounterMw).unwrap()
            ),
            ("/eee", get(add13))
          ),
          (),
        )
        .unwrap()
      ),
      ("/fff", get(add14)),
    ),
    CounterMw,
  )
  .unwrap();

  let sf = ServerFrameworkBuilder::new((), router).with_dflt_aux();
  let mut auto_stream = AutoStream {
    conn_aux: Counter(0),
    peer: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
    protocol: None,
    req: Request::http2(Method::Get, ReqResBuffer::default()),
    stream_aux: Counter(0),
  };

  {
    auto_stream.req.rrd.uri.reset().push_str("http://localhost/aaa/bbb/ccc");
    let el = ServerFramework::<_, (), (), _, _, _, _, _, ()>::route_params(
      auto_stream.req.rrd.uri.path(),
      &sf._router,
    )
    .unwrap();
    let _ = sf._router.auto(&mut auto_stream, (0, &el.0)).await.unwrap();
    // 3 + 3 + 11 + 7 + 7
    assert_eq!(auto_stream.conn_aux.0, 31);
    // 3 + 3 + 11 + 7 + 7
    assert_eq!(auto_stream.stream_aux.0, 31);
  }

  auto_stream.conn_aux = Counter(0);
  auto_stream.req.rrd.clear();
  auto_stream.stream_aux = Counter(0);

  {
    auto_stream.req.rrd.uri.reset().push_str("http://localhost/fff");
    let el = ServerFramework::<_, (), (), _, _, _, _, _, ()>::route_params(
      auto_stream.req.rrd.uri.path(),
      &sf._router,
    )
    .unwrap();
    let _ = sf._router.auto(&mut auto_stream, (0, &el.0)).await.unwrap();
    // 3 + 17 + 7
    assert_eq!(auto_stream.conn_aux.0, 27);
    // 3 + 17 + 7
    assert_eq!(auto_stream.stream_aux.0, 27);
  }
}
