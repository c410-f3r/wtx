use crate::{
  http::{Method, RequestMut, Response},
  http2::ReqResBuffer,
  misc::{ByteVector, Uri},
};

pub struct ServerStream {}

impl ServerStream {
  pub(crate) fn new() -> Self {
    Self {}
  }

  #[inline]
  pub async fn recv_req<'rrb>(
    &self,
    rrb: &'rrb mut ReqResBuffer,
  ) -> crate::Result<RequestMut<'rrb, 'rrb, 'rrb, ByteVector>> {
    //Ok(RequestMut::http2(&mut rrb.data, &mut rrb.headers, Method::Get, Uri::new(&mut rrb.uri)))
    todo!()
  }

  #[inline]
  pub async fn send_res<D>(&self, res: Response<D>) -> crate::Result<()> {
    Ok(())
  }
}
