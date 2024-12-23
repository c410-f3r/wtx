#![expect(clippy::indexing_slicing, reason = "intended for testing environments")]

use crate::{
  client_api_framework::{
    misc::{manage_after_sending_related, manage_before_sending_related, FromBytes},
    network::{
      transport::{RecievingTransport, SendingTransport, Transport, TransportParams},
      TransportGroup,
    },
    pkg::{Package, PkgsAux},
    Api, ClientApiFrameworkError,
  },
  misc::{Deque, Lease, Vector},
};
use alloc::borrow::{Cow, ToOwned};
use core::{fmt::Debug, marker::PhantomData, ops::Range};

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
///   network::transport::{MockStr, SendingRecievingTransport},
///   pkg::PkgsAux,
/// };
/// let _ = MockStr::default()
///   .send_recv_decode_contained(&mut (), &mut PkgsAux::from_minimum((), (), ()))
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

impl<DRSR, T, TP> RecievingTransport<DRSR> for Mock<T, TP>
where
  T: Debug + Lease<[u8]> + PartialEq + ToOwned + 'static + ?Sized,
  TP: TransportParams,
  <T as ToOwned>::Owned: Debug + FromBytes,
{
  #[inline]
  async fn recv<A>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
  {
    let response = self.pop_response()?;
    pkgs_aux.byte_buffer.clear();
    pkgs_aux
      .byte_buffer
      .extend_from_copyable_slice(response.as_ref().lease())
      .map_err(Into::into)?;
    Ok(0..pkgs_aux.byte_buffer.len())
  }
}

impl<DRSR, T, TP> SendingTransport<DRSR> for Mock<T, TP>
where
  T: Debug + Lease<[u8]> + PartialEq + ToOwned + 'static + ?Sized,
  TP: TransportParams,
  <T as ToOwned>::Owned: Debug + FromBytes,
{
  #[inline]
  async fn send<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, TP>,
  {
    manage_before_sending_related(pkg, pkgs_aux, &mut *self).await?;
    self
      .requests
      .push(Cow::Owned(FromBytes::from_bytes(&pkgs_aux.byte_buffer)?))
      .map_err(Into::into)?;
    pkgs_aux.byte_buffer.clear();
    manage_after_sending_related(pkg, pkgs_aux).await?;
    Ok(())
  }
}

impl<DRSR, T, TP> Transport<DRSR> for Mock<T, TP>
where
  T: Debug + Lease<[u8]> + PartialEq + ToOwned + 'static + ?Sized,
  TP: TransportParams,
  <T as ToOwned>::Owned: Debug + FromBytes,
{
  const GROUP: TransportGroup = TransportGroup::Stub;
  type Params = TP;
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
