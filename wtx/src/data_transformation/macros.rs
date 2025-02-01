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

macro_rules! generic_data_format_doc {
  ($ty:literal) => {
    concat!("Wrapper used in every generic ", $ty, " to manage different internal implementations.")
  };
}

//macro_rules! _impl_se {
//  (
//    ($drsr:ty, $bound:path)
//  ) => {
//    impl<D> crate::data_transformation::dnsn::Serialize<&mut SerdeJson> for VerbatimRequest<D>
//    where
//      D: serde::Serialize,
//    {
//      #[inline]
//      fn to_bytes(&mut self, bytes: &mut Vector<u8>, _: &mut &mut SerdeJson) -> crate::Result<()> {
//        if size_of::<D>() == 0 {
//          return Ok(());
//        }
//        serde_json::to_writer(bytes, &self.data)?;
//        Ok(())
//      }
//    }
//  }
//}

macro_rules! _impl_se_collections {
  (
    ($drsr:ty, $bound:path),
    $( array: |$array_self:ident, $array_bytes:ident, $array_drsr:ident| $array_block:block )?
    $( arrayvector: |$arrayvector_self:ident, $arrayvector_bytes:ident, $arrayvector_drsr:ident| $arrayvector_block:block )?
    slice_ref: |$slice_ref_self:ident, $slice_ref_bytes:ident, $slice_ref_drsr:ident| $slice_ref_block:block
    vec: |$vec_self:ident, $vec_bytes:ident, $vec_drsr:ident| $vec_block:block
  ) => {
    $(
      impl<T, const N: usize> crate::data_transformation::dnsn::Serialize<$drsr> for [T; N]
      where
        T: $bound,
      {
        #[inline]
        fn to_bytes(&mut self, bytes: &mut crate::misc::Vector<u8>, drsr: &mut $drsr) -> crate::Result<()>
        {
          let $array_self = self;
          let $array_bytes = bytes;
          let $array_drsr = drsr;
          $array_block;
          Ok(())
        }
      }
    )?

    $(
      impl<T, const N: usize> crate::data_transformation::dnsn::Serialize<$drsr> for crate::misc::ArrayVector<T, N>
      where
        T: $bound,
      {
        #[inline]
        fn to_bytes(&mut self, bytes: &mut crate::misc::Vector<u8>, drsr: &mut $drsr) -> crate::Result<()> {
          let $arrayvector_self = self;
          let $arrayvector_bytes = bytes;
          let $arrayvector_drsr = drsr;
          $arrayvector_block;
          Ok(())
        }
      }
    )?

    impl<T> crate::data_transformation::dnsn::Serialize<$drsr> for &[T]
    where
      T: $bound,
    {
      #[inline]
      fn to_bytes(&mut self, bytes: &mut crate::misc::Vector<u8>, drsr: &mut $drsr) -> crate::Result<()> {
        let $slice_ref_self = self;
        let $slice_ref_bytes = bytes;
        let $slice_ref_drsr = drsr;
        $slice_ref_block;
        Ok(())
      }
    }

    impl<T> crate::data_transformation::dnsn::Serialize<$drsr> for crate::misc::Vector<T>
    where
      T: $bound,
    {
      #[inline]
      fn to_bytes(&mut self, bytes: &mut crate::misc::Vector<u8>, drsr: &mut $drsr) -> crate::Result<()>  {
        let $vec_self = self;
        let $vec_bytes = bytes;
        let $vec_drsr = drsr;
        $vec_block;
        Ok(())
      }
    }
  };
}
