/// Useful to automatically create a local `PkgsAux` wrapper that implements
/// `core::ops::DerefMut` in case you want to use a fluent-like interface for your APIs.
#[macro_export]
macro_rules! create_packages_aux_wrapper {
  () => {
    $crate::create_packages_aux_wrapper!(@PkgsAux<A with A>);
  };
  ($name:ident) => {
    $crate::create_packages_aux_wrapper!(@$name<A with A>);
  };
  ($name:ident, $api_ty:ty) => {
    $crate::create_packages_aux_wrapper!(@$name<with $api_ty>);
  };
  (
    @$name:ident<
      $($api_param:ident)? with $api_ty:ty
    >
  ) => {
    /// Just a wrapper that implements [core::ops::Deref] and [`core::ops::DerefMut`] to easily call
    /// methods from `PkgsAux`.
    #[derive(Debug)]
    pub struct $name<$($api_param,)? DRSR, TP>($crate::client_api_framework::pkg::PkgsAux<$api_ty, DRSR, TP>)
    where
      TP: $crate::client_api_framework::network::transport::TransportParams;

    impl<$($api_param,)? DRSR, TP> $name<$api_ty, DRSR, TP>
    where
      TP: $crate::client_api_framework::network::transport::TransportParams
    {
      /// Proxy of [`$crate::client_api_framework::pkg::PkgsAux::from_minimum`].
      #[inline]
      pub fn from_minimum(api: $api_ty, drsr: DRSR, tp: TP) -> Self {
        Self($crate::client_api_framework::pkg::PkgsAux::from_minimum(api, drsr, tp))
      }
    }

    impl<$($api_param,)? DRSR, TP> core::ops::Deref for $name<$api_ty, DRSR, TP>
    where
      TP: $crate::client_api_framework::network::transport::TransportParams
    {
      type Target = $crate::client_api_framework::pkg::PkgsAux<$api_ty, DRSR, TP>;

      #[inline]
      fn deref(&self) -> &Self::Target {
        &self.0
      }
    }

    impl<$($api_param,)? DRSR, TP> core::ops::DerefMut for $name<$api_ty, DRSR, TP>
    where
      TP: $crate::client_api_framework::network::transport::TransportParams
    {
      #[inline]
      fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
      }
    }

    impl<$($api_param,)? DRSR, TP> From<$crate::client_api_framework::pkg::PkgsAux<$api_ty, DRSR, TP>> for $name<$api_ty, DRSR, TP>
    where
      TP: $crate::client_api_framework::network::transport::TransportParams
    {
      #[inline]
      fn from(from: $crate::client_api_framework::pkg::PkgsAux<$api_ty, DRSR, TP>) -> Self {
        Self(from)
      }
    }
  };
}

macro_rules! _create_set_of_request_throttling {
  (
    $name:ident {
      $( $method:ident ),+ $(,)?
    }
  ) => {
    /// A set of [`$crate::utils::RequestThrottling`] for specified API usage
    #[derive(Debug)]
    pub struct $name {
      $(
        pub(crate) $method: $crate::utils::RequestThrottling,
      )+
    }

    impl $name {
            pub fn new(
        $( $method: $crate::utils::RequestLimit, )+
      ) -> Self {
        Self {
          $(
            $method: $crate::utils::RequestThrottling::from_rl($method),
          )+
        }
      }
    }
  };
}

macro_rules! generic_trans_params_doc {
  () => {
    "Grouping of request and response parameters"
  };
}

macro_rules! generic_trans_req_params_doc {
  ($ty:literal) => {
    concat!("All possible ", $ty, " parameters that a request can manipulate for sending.")
  };
}

macro_rules! generic_trans_res_params_doc {
  ($ty:literal) => {
    concat!("All possible response parameters returned by a ", $ty, " request.")
  };
}
