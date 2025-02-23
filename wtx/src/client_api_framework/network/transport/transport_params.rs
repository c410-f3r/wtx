/// Additional information or metadata received or transmitted by a transport.
pub trait TransportParams {
  /// For example, HTTP has request headers.
  type ExternalRequestParams;
  /// For example, HTTP has response headers.
  type ExternalResponseParams;

  /// External Request and Response Parameters.
  fn ext_params(&self) -> (&Self::ExternalRequestParams, &Self::ExternalResponseParams);

  /// Mutable version of [TransportParams::ext_params].
  fn ext_params_mut(
    &mut self,
  ) -> (&mut Self::ExternalRequestParams, &mut Self::ExternalResponseParams);

  /// External Request Parameters.
  fn ext_req_params(&self) -> &Self::ExternalRequestParams {
    self.ext_params().0
  }

  /// Mutable version of [`Self::ext_req_params`].
  fn ext_req_params_mut(&mut self) -> &mut Self::ExternalRequestParams {
    self.ext_params_mut().0
  }

  /// External Response Parameters.
  fn ext_res_params(&self) -> &Self::ExternalResponseParams {
    self.ext_params().1
  }

  /// Mutable version of [`Self::ext_res_params`].
  fn ext_res_params_mut(&mut self) -> &mut Self::ExternalResponseParams {
    self.ext_params_mut().1
  }

  /// Sets the inner parameters with their default values.
  fn reset(&mut self);
}

impl TransportParams for () {
  type ExternalRequestParams = ();
  type ExternalResponseParams = ();

  #[inline]
  fn ext_params(&self) -> (&Self::ExternalRequestParams, &Self::ExternalResponseParams) {
    (&(), &())
  }

  #[inline]
  fn ext_params_mut(
    &mut self,
  ) -> (&mut Self::ExternalRequestParams, &mut Self::ExternalResponseParams) {
    (self, alloc::boxed::Box::leak(alloc::boxed::Box::new(())))
  }

  #[inline]
  fn reset(&mut self) {}
}

impl<TP> TransportParams for &mut TP
where
  TP: TransportParams,
{
  type ExternalRequestParams = TP::ExternalRequestParams;
  type ExternalResponseParams = TP::ExternalResponseParams;

  #[inline]
  fn ext_params(&self) -> (&Self::ExternalRequestParams, &Self::ExternalResponseParams) {
    (**self).ext_params()
  }

  #[inline]
  fn ext_params_mut(
    &mut self,
  ) -> (&mut Self::ExternalRequestParams, &mut Self::ExternalResponseParams) {
    (**self).ext_params_mut()
  }

  #[inline]
  fn reset(&mut self) {
    (**self).reset()
  }
}
