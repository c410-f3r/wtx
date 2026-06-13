use crate::{
  executor::StdRuntime,
  http::{
    AutoStream, ManualStream, Method, MsgBufferString, Request, Response, StatusCode,
    http2_server_framework::{HttpRouter, Middleware, StateClean, endpoint::Endpoint, get},
  },
};
use core::{
  net::{IpAddr, Ipv4Addr},
  ops::ControlFlow,
};

#[test]
fn compiles() {
  async fn one(_: StateClean<'_, ()>) -> crate::Result<StatusCode> {
    Ok(StatusCode::Ok)
  }

  async fn two(_: StateClean<'_, ()>) -> crate::Result<StatusCode> {
    Ok(StatusCode::Ok)
  }

  async fn three(_: ManualStream<(), ()>) -> crate::Result<()> {
    Ok(())
  }
  let _router = HttpRouter::paths(paths!(
    ("/aaa", HttpRouter::paths(paths!(("/bbb", get(one)), ("/ccc", get(two)))).unwrap()),
    ("/ddd", get(one)),
    ("/eee", get(two)),
    ("/fff", HttpRouter::paths(paths!(("/ggg", get(one)))).unwrap()),
    ("/hhh", get(three)),
  ))
  .unwrap();
}

// /aaa ->   /bbb ->  /ccc
//      \         \
//       \         -> /ddd
//        -> /eee
//
// /fff
#[test]
fn nested_middlewares() {
  struct Counter(u8);

  struct CounterMw;

  impl Middleware<Counter, crate::Error> for CounterMw {
    type Aux = ();

    fn aux(&self) -> Self::Aux {}

    async fn req(
      &self,
      data: &mut Counter,
      _: &mut Self::Aux,
      _: &mut Request<MsgBufferString>,
    ) -> crate::Result<ControlFlow<StatusCode, ()>> {
      data.0 += 3;
      Ok(ControlFlow::Continue(()))
    }

    async fn res(
      &self,
      data: &mut Counter,
      _: &mut Self::Aux,
      _: Response<&mut MsgBufferString>,
    ) -> crate::Result<ControlFlow<StatusCode, ()>> {
      data.0 += 7;
      Ok(ControlFlow::Continue(()))
    }
  }

  async fn add11(state: StateClean<'_, Counter>) -> crate::Result<StatusCode> {
    assert_eq!(state.data.0, 6);
    state.data.0 += 11;
    Ok(StatusCode::Ok)
  }

  async fn add12(mut state: ManualStream<Counter, ()>) -> crate::Result<()> {
    state.data.0 += 13;
    Ok(())
  }

  async fn add13(state: StateClean<'_, Counter>) -> crate::Result<StatusCode> {
    state.data.0 += 15;
    Ok(StatusCode::Ok)
  }

  async fn add14(state: StateClean<'_, Counter>) -> crate::Result<StatusCode> {
    assert_eq!(state.data.0, 3);
    state.data.0 += 17;
    Ok(StatusCode::Ok)
  }

  let http_router = HttpRouter::new(
    paths!(
      (
        "/aaa",
        HttpRouter::new(
          paths!(
            (
              "/bbb",
              HttpRouter::new(paths!(("/ccc", get(add11)), ("/ddd", get(add12))), CounterMw)
                .unwrap()
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

  StdRuntime::new().block_on(async {
    let mut auto_stream = AutoStream {
      data: Counter(0),
      peer: IpAddr::V4(Ipv4Addr::from_bits(0)),
      protocol: None,
      req: Request::http2(Method::Get, MsgBufferString::default()),
    };

    {
      auto_stream.req.msg_data.uri.reset().push_str("http://localhost/aaa/bbb/ccc");
      let path = auto_stream.req.msg_data.uri.path();
      let path_defs = http_router.router.find(path).unwrap().data().clone().0;
      let _ = http_router.auto(&mut auto_stream, (0, &path_defs)).await.unwrap();
      // 3 + 3 + 11 + 7 + 7
      assert_eq!(auto_stream.data.0, 31);
    }

    auto_stream.data = Counter(0);
    auto_stream.req.clear();

    {
      auto_stream.req.msg_data.uri.reset().push_str("http://localhost/fff");
      let path = auto_stream.req.msg_data.uri.path();
      let path_defs = http_router.router.find(path).unwrap().data().clone().0;
      let _ = http_router.auto(&mut auto_stream, (0, &path_defs)).await.unwrap();
      // 3 + 17 + 7
      assert_eq!(auto_stream.data.0, 27);
    }
  });
}
