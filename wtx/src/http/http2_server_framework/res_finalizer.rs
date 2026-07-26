use crate::{
  collections::{ArrayString, ArrayVector, LinearStorageLen, Vector},
  http::{MsgBufferString, Request, StatusCode},
};
use alloc::string::String;

/// Modifies responses
pub trait ResFinalizer<E> {
  /// Finalize response
  fn finalize_response(self, req: &mut Request<MsgBufferString>) -> Result<StatusCode, E>;
}

impl<E> ResFinalizer<E> for ()
where
  E: From<crate::Error>,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<MsgBufferString>) -> Result<StatusCode, E> {
    req.clear();
    Ok(StatusCode::Ok)
  }
}

impl<E> ResFinalizer<E> for &'static str
where
  E: From<crate::Error>,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<MsgBufferString>) -> Result<StatusCode, E> {
    req.clear();
    req.msg_data.body.extend_from_copyable_slice(self.as_bytes())?;
    Ok(StatusCode::Ok)
  }
}

impl<E, L, const N: usize> ResFinalizer<E> for ArrayString<L, N>
where
  E: From<crate::Error>,
  L: LinearStorageLen,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<MsgBufferString>) -> Result<StatusCode, E> {
    req.clear();
    req.msg_data.body.extend_from_copyable_slice(self.as_bytes())?;
    Ok(StatusCode::Ok)
  }
}

impl<E, L, const N: usize> ResFinalizer<E> for ArrayVector<L, u8, N>
where
  E: From<crate::Error>,
  L: LinearStorageLen,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<MsgBufferString>) -> Result<StatusCode, E> {
    req.clear();
    req.msg_data.body.extend_from_copyable_slice(&self)?;
    Ok(StatusCode::Ok)
  }
}

impl<E, T> ResFinalizer<E> for Result<T, E>
where
  E: From<crate::Error>,
  T: ResFinalizer<E>,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<MsgBufferString>) -> Result<StatusCode, E> {
    self?.finalize_response(req)
  }
}

impl<E> ResFinalizer<E> for String
where
  E: From<crate::Error>,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<MsgBufferString>) -> Result<StatusCode, E> {
    req.clear();
    req.msg_data.body.extend_from_copyable_slice(self.as_bytes())?;
    Ok(StatusCode::Ok)
  }
}

impl<E> ResFinalizer<E> for Vector<u8>
where
  E: From<crate::Error>,
{
  #[inline]
  fn finalize_response(self, req: &mut Request<MsgBufferString>) -> Result<StatusCode, E> {
    req.clear();
    req.msg_data.body.extend_from_copyable_slice(&self)?;
    Ok(StatusCode::Ok)
  }
}
