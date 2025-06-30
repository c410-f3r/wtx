macro_rules! generic_data_format_doc {
  ($ty:literal) => {
    concat!("Wrapper used in every generic ", $ty, " to manage different internal implementations.")
  };
}

macro_rules! _impl_dec {
  (
    $struct:ident<$(($($ty:tt)*) $(: $bound:path)?),*>,
    $drsr:ty,
    |$aux:ident, $dw:ident| $impl:block
  ) => {
    impl<'de, $($($ty)* $(: $bound)?,)*> crate::de::Decode<'de, crate::de::format::De<$drsr>> for $struct<$($($ty)*,)*> {
      #[inline]
      fn decode(
        $aux: &mut $drsr,
        $dw: &mut crate::de::format::DecodeWrapper<'de>
      ) -> crate::Result<Self> {
        $impl
      }
    }

    impl<'de, $($($ty)* $(: $bound)?,)*> crate::de::Decode<'de, crate::de::format::De<&mut $drsr>> for $struct<$($($ty)*,)*>
    where
      $struct<$($($ty)*,)*>: crate::de::Decode<'de, crate::de::format::De<$drsr>>,
    {
      #[inline]
      fn decode(
        aux: &mut &mut $drsr,
        dw: &mut crate::de::format::DecodeWrapper<'de>
      ) -> crate::Result<Self> {
        <$struct<$($($ty)*,)*>>::decode(*aux, dw)
      }
    }
  }
}

macro_rules! _impl_dec_seq {
  (
    $struct:ident<$($ty:ident $(: $bound:path)?),*>,
    $drsr:ty,
    |$aux:ident, $buffer:ident, $dw:ident| $impl:block
  ) => {
    impl<'de, $($ty: $($bound)?,)*> crate::de::DecodeSeq<'de, crate::de::format::De<$drsr>> for $struct<$($ty,)*> {
      #[inline]
      fn decode_seq(
        $aux: &mut $drsr,
        $buffer: &mut crate::collection::Vector<Self>,
        $dw: &mut crate::de::format::DecodeWrapper<'de>,
      ) -> crate::Result<()> {
        $impl
      }
    }

    impl<'de, $($ty: $($bound)?,)*> crate::de::DecodeSeq<'de, crate::de::format::De<&mut $drsr>> for $struct<$($ty,)*>
    where
      $struct<$($ty,)*>: crate::de::Decode<'de, crate::de::format::De<$drsr>>,
    {
      #[inline]
      fn decode_seq(
        aux: &mut &mut $drsr,
        buffer: &mut crate::collection::Vector<Self>,
        dw: &mut crate::de::format::DecodeWrapper<'de>,
      ) -> crate::Result<()> {
        <$struct<$($ty,)*>>::decode_seq(*aux, buffer, dw)
      }
    }
  }
}

macro_rules! _impl_enc {
  (
    $struct:ident<$($ty:ident $(: $bound:path)?),*>,
    $drsr:ty,
    |$this:ident, $aux:ident, $ew:ident| $impl:block
  ) => {
    impl<$($ty: $($bound)?,)*> crate::de::Encode<crate::de::format::De<$drsr>> for $struct<$($ty,)*> {
      #[inline]
      fn encode(
        &self,
        $aux: &mut $drsr,
        $ew: &mut crate::de::format::EncodeWrapper<'_>
      ) -> crate::Result<()> {
        if size_of::<Self>() == 0 {
          return Ok(());
        }
        let $this = self;
        $impl
        Ok(())
      }
    }

    impl<'de, $($ty: $($bound)?,)*> crate::de::Encode<crate::de::format::De<&mut $drsr>> for $struct<$($ty,)*>
    where
      $struct<$($ty,)*>: crate::de::Encode<crate::de::format::De<$drsr>>,
    {
      #[inline]
      fn encode(
        &self,
        aux: &mut &mut $drsr,
        ew: &mut crate::de::format::EncodeWrapper<'_>
      ) -> crate::Result<()> {
        <$struct<$($ty,)*>>::encode(self, *aux, ew)
      }
    }
  }
}

macro_rules! _impl_se_collections {
  (
    ($drsr:ty, $bound:path),
    $( array: |$array_self:ident, $array_bytes:ident, $array_drsr:ident| $array_block:block )?
    $( arrayvector: |$arrayvector_self:ident, $arrayvector_bytes:ident, $arrayvector_drsr:ident| $arrayvector_block:block )?
    slice_ref: |$slice_ref_self:ident, $slice_ref_bytes:ident, $slice_ref_drsr:ident| $slice_ref_block:block
    vec: |$vec_self:ident, $vec_bytes:ident, $vec_drsr:ident| $vec_block:block
  ) => {
    $(
      impl<T, const N: usize> crate::de::Encode<crate::de::format::De<$drsr>> for [T; N]
      where
        T: $bound,
      {
        #[inline]
        fn encode(&self, $array_drsr: &mut SerdeJson, ew: &mut crate::de::format::EncodeWrapper<'_>) -> crate::Result<()> {
          let $array_self = self;
          let $array_bytes = &mut *ew.vector;
          $array_block;
          Ok(())
        }
      }
    )?

    $(
      impl<L, T, const N: usize> crate::de::Encode<crate::de::format::De<$drsr>> for crate::collection::ArrayVector<L, T, N>
      where
        L: crate::collection::IndexedStorageLen,
        T: $bound,
      {
        #[inline]
        fn encode(&self, $arrayvector_drsr: &mut SerdeJson, ew: &mut crate::de::format::EncodeWrapper<'_>) -> crate::Result<()> {
          let $arrayvector_self = self;
          let $arrayvector_bytes = &mut *ew.vector;
          $arrayvector_block;
          Ok(())
        }
      }
    )?

    impl<T> crate::de::Encode<crate::de::format::De<$drsr>> for &'_ [T]
    where
      T: $bound,
    {
      #[inline]
      fn encode(&self, $slice_ref_drsr: &mut SerdeJson, ew: &mut crate::de::format::EncodeWrapper<'_>) -> crate::Result<()> {
        let $slice_ref_self = self;
        let $slice_ref_bytes = &mut *ew.vector;
        $slice_ref_block;
        Ok(())
      }
    }

    impl<T> crate::de::Encode<crate::de::format::De<$drsr>> for crate::collection::Vector<T>
    where
      T: $bound,
    {
      #[inline]
      fn encode(&self, $vec_drsr: &mut SerdeJson, ew: &mut crate::de::format::EncodeWrapper<'_>) -> crate::Result<()> {
        let $vec_self = self;
        let $vec_bytes = &mut *ew.vector;
        $vec_block;
        Ok(())
      }
    }
  };
}
