use crate::{
  http::{Headers, ReqResBuffer, Request},
  misc::Vector,
};

/// Used as an auxiliary tool for tests.
#[inline]
pub fn state_tuple_wo_uri<CA, SA>(
  (conn_aux, stream_aux, req): &mut (CA, SA, Request<ReqResBuffer>),
) -> (&mut CA, &mut SA, Request<(&mut Vector<u8>, &mut Headers)>) {
  let (body, headers, _) = req.rrd.parts_mut();
  (conn_aux, stream_aux, Request { method: req.method, rrd: (body, headers), version: req.version })
}
