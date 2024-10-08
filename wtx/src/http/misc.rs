use crate::{
  http::{Headers, ReqResBuffer, Request},
  misc::Vector,
};

/// Used as an auxiliary tool for tests.
#[inline]
pub fn state_tuple_wo_uri<CA, RA>(
  (ca, ra, req): &mut (CA, RA, Request<ReqResBuffer>),
) -> (&mut CA, &mut RA, Request<(&mut Vector<u8>, &mut Headers)>) {
  let (body, headers, _) = req.rrd.parts_mut();
  (ca, ra, Request { method: req.method, rrd: (body, headers), version: req.version })
}
