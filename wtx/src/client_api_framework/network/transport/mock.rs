#![expect(clippy::indexing_slicing, reason = "intended for testing environments")]

use crate::{
  client_api_framework::{
    Api, ClientApiFrameworkError, SendBytesSource,
    misc::{
      FromBytes, log_req, manage_after_sending_bytes, manage_after_sending_pkg,
      manage_before_sending_bytes, manage_before_sending_pkg,
    },
    network::{
      TransportGroup,
      transport::{ReceivingTransport, SendingTransport, Transport, TransportParams},
    },
    pkg::{Package, PkgsAux},
  },
  collection::{Deque, Vector},
  misc::Lease,
};
use alloc::borrow::{Cow, ToOwned};
use core::{fmt::Debug, marker::PhantomData};

/// For API's that send and received raw bytes.
pub type MockBytes<TP> = Mock<[u8], TP>;
/// For API's that send and received strings.
pub type MockStr<TP> = Mock<str, TP>;

/// Used to assert issued requests as well as returned responses in a local environment.
///
/// Almost all methods panic at runtime.
///
/// ```rust,no_run
/// # async fn fun() -> wtx::Result<()> {
/// use wtx::client_api_framework::{
///   network::transport::{MockStr, SendingReceivingTransport},
///   pkg::PkgsAux,
/// };
/// let _ = MockStr::default()
///   .send_pkg_recv_decode_contained(&mut (), &mut PkgsAux::from_minimum((), (), ()))
///   .await?;
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct Mock<T, TP>
where
  T: ToOwned + 'static + ?Sized,
  <T as ToOwned>::Owned: Debug + FromBytes,
{
  asserted: usize,
  phantom: PhantomData<TP>,
  requests: Vector<Cow<'static, T>>,
  responses: Deque<Cow<'static, T>>,
}

impl<T, TP> Mock<T, TP>
where
  T: Debug + Lease<[u8]> + PartialEq + ToOwned + 'static + ?Sized,
  <T as ToOwned>::Owned: Debug + FromBytes,
{
  /// Ensures that no included request hasn't processed.
  ///
  /// # Panics
  ///
  /// If the number of asserted requests differs from the number of stored requests.
  #[inline]
  pub fn assert_does_not_have_non_asserted_requests(&self) {
    assert_eq!(self.asserted, self.requests.len());
  }

  /// Verifies if `req` is present in the inner request storage.
  ///
  /// # Panics
  ///
  /// If the stored request differs from the passed request.
  #[inline]
  #[track_caller]
  pub fn assert_request(&mut self, req: &T) {
    let stored = &self.requests[self.asserted];
    self.asserted = self.asserted.wrapping_add(1);
    assert_eq!(req, stored.as_ref());
  }

  /// Stores `res` into the inner response storage
  #[inline]
  pub fn push_response(&mut self, res: Cow<'static, T>) {
    self.responses.push_back(res).unwrap();
  }

  fn pop_response(&mut self) -> crate::Result<Cow<'static, T>> {
    Ok(self.responses.pop_front().ok_or(ClientApiFrameworkError::TestTransportNoResponse)?)
  }
}

impl<T, TP> ReceivingTransport<TP> for Mock<T, TP>
where
  T: Debug + Lease<[u8]> + PartialEq + ToOwned + 'static + ?Sized,
  TP: TransportParams,
  <T as ToOwned>::Owned: Debug + FromBytes,
{
  #[inline]
  async fn recv<A, DRSR>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    _: Self::ReqId,
  ) -> Result<(), A::Error>
  where
    A: Api,
  {
    let response = self.pop_response()?;
    pkgs_aux.byte_buffer.clear();
    pkgs_aux.byte_buffer.extend_from_copyable_slice(response.as_ref().lease())?;
    Ok(())
  }
}

impl<T, TP> SendingTransport<TP> for Mock<T, TP>
where
  T: Debug + Lease<[u8]> + PartialEq + ToOwned + 'static + ?Sized,
  TP: TransportParams,
  <T as ToOwned>::Owned: Debug + FromBytes,
{
  #[inline]
  async fn send_bytes<A, DRSR>(
    &mut self,
    bytes: SendBytesSource<'_>,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<(), A::Error>
  where
    A: Api,
  {
    manage_before_sending_bytes(pkgs_aux).await?;
    log_req(bytes.bytes(&pkgs_aux.byte_buffer), pkgs_aux.log_body.1, &mut *self);
    self.requests.push(Cow::Owned(FromBytes::from_bytes(bytes.bytes(&pkgs_aux.byte_buffer))?))?;
    pkgs_aux.byte_buffer.clear();
    manage_after_sending_bytes(pkgs_aux).await?;
    Ok(())
  }

  #[inline]
  async fn send_pkg<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>,
  {
    manage_before_sending_pkg(pkg, pkgs_aux, &mut *self).await?;
    log_req(&pkgs_aux.byte_buffer, pkgs_aux.log_body.1, &mut *self);
    self.requests.push(Cow::Owned(FromBytes::from_bytes(&pkgs_aux.byte_buffer)?))?;
    pkgs_aux.byte_buffer.clear();
    manage_after_sending_pkg(pkg, pkgs_aux, &mut *self).await?;
    Ok(())
  }
}

impl<T, TP> Transport<TP> for Mock<T, TP>
where
  T: Debug + Lease<[u8]> + PartialEq + ToOwned + 'static + ?Sized,
  TP: TransportParams,
  <T as ToOwned>::Owned: Debug + FromBytes,
{
  const GROUP: TransportGroup = TransportGroup::Stub;
  type Inner = Self;
  type ReqId = ();
}

impl<T, TP> Default for Mock<T, TP>
where
  T: ToOwned + 'static + ?Sized,
  <T as ToOwned>::Owned: Debug + FromBytes,
{
  #[inline]
  fn default() -> Self {
    Self { asserted: 0, phantom: PhantomData, requests: Vector::new(), responses: Deque::new() }
  }
}
