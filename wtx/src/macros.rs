macro_rules! _conn_params_methods {
  () => {
    /// The initial amount of "credit" a counterpart can have for sending data.
    #[inline]
    #[must_use]
    pub fn initial_window_len(mut self, elem: u32) -> Self {
      self.cp._initial_window_len = elem;
      self
    }

    /// The maximum number of data bytes or the sum of all frames that composed the body data.
    #[inline]
    #[must_use]
    pub fn max_body_len(mut self, elem: u32) -> Self {
      self.cp._max_body_len = elem;
      self
    }

    /// Maximum number of active concurrent streams
    #[inline]
    #[must_use]
    pub fn max_concurrent_streams_num(mut self, elem: u32) -> Self {
      self.cp._max_concurrent_streams_num = elem;
      self
    }

    /// Maximum frame ***payload*** length
    #[inline]
    #[must_use]
    pub fn max_frame_len(mut self, elem: u32) -> Self {
      self.cp._max_frame_len = elem;
      self
    }

    /// Maximum HPACK length
    ///
    /// Indicates the maximum length of the HPACK structure that holds cached decoded headers
    /// received from a counterpart.
    ///
    /// - The first parameter indicates the local HPACK ***decoder*** length that is externally
    ///   advertised and can become the remote HPACK ***encoder*** length.
    /// - The second parameter indicates the maximum local HPACK ***encoder*** length. In other words,
    ///   it doesn't allow external actors to dictate very large lengths.
    #[inline]
    #[must_use]
    pub fn max_hpack_len(mut self, elem: (u32, u32)) -> Self {
      self.cp._max_hpack_len = elem;
      self
    }

    /// The maximum number of bytes of the entire set of headers in a request/response.
    #[inline]
    #[must_use]
    pub fn max_headers_len(mut self, elem: u32) -> Self {
      self.cp._max_headers_len = elem;
      self
    }

    /// Maximum number of receiving streams
    ///
    /// Servers only. Prevents clients from opening more than the specified number of streams.
    #[inline]
    #[must_use]
    pub fn max_recv_streams_num(mut self, elem: u32) -> Self {
      self.cp._max_recv_streams_num = elem;
      self
    }
  };
}

macro_rules! _create_enum {
  (
    $(#[$container_mac:meta])*
    $v:vis enum $enum_ident:ident<$n:ty> {
      $(
        $(#[$variant_mac_fixed:meta])*
        $variant_ident_fixed:ident = ($variant_n_fixed:literal $(, $variant_str_fixed:literal)? $(| $variant_str_fixed_n:literal)*)
      ),* $(,)?
    }
  ) => {
    $(#[$container_mac])*
    $v enum $enum_ident {
      $($(#[$variant_mac_fixed])* $variant_ident_fixed,)*
    }

    #[allow(dead_code, reason = "outside may or may not use methods")]
    impl $enum_ident {
      #[inline]
      /// The total number of variants
      $v const fn len() -> usize {
        const { 0 $( + { let _ = $variant_n_fixed; 1 })* }
      }

      /// See [`crate::misc::EnumVarStrings`].
      #[inline]
      $v const fn strings(&self) -> crate::misc::EnumVarStrings<{
        let mut n;
        $({
          #[allow(unused_mut, reason = "repetition can be empty")]
          let mut local_n = 0;
          let _ = $variant_n_fixed;
          $({ let _ = $variant_str_fixed; local_n += 1; })?
          $({ let _ = $variant_str_fixed_n; local_n += 1; })*
          #[allow(unused_assignments, reason = "repetition can be empty")]
          { n = local_n; }
        })*
        n
      }> {
        match self {
          $(
            $enum_ident::$variant_ident_fixed => crate::misc::EnumVarStrings {
              custom: [$($variant_str_fixed,)? $($variant_str_fixed_n,)*],
              ident: stringify!($variant_ident_fixed),
              number: stringify!($variant_n_fixed),
            },
          )*
        }
      }
    }

    #[allow(
      unused_qualifications,
      reason = "macro shouldn't control what the outside uses"
    )]
    impl core::fmt::Display for $enum_ident {
      #[inline]
      fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.strings().ident)
      }
    }

    impl From<$enum_ident> for $n {
      #[inline]
      fn from(from: $enum_ident) -> Self {
        match from {
          $($enum_ident::$variant_ident_fixed => $variant_n_fixed,)*
        }
      }
    }

    impl core::str::FromStr for $enum_ident {
      type Err = crate::Error;

      #[inline]
      fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
      }
   }

    impl TryFrom<$n> for $enum_ident {
      type Error = crate::Error;

      #[inline]
      fn try_from(from: $n) -> crate::Result<Self> {
        let rslt = match from {
          $($variant_n_fixed => Self::$variant_ident_fixed,)*
          _ => return Err(crate::Error::UnexpectedUint { received: from.into() }),
        };
        Ok(rslt)
      }
    }

    impl TryFrom<&str> for $enum_ident {
      type Error = crate::Error;

      #[inline]
      fn try_from(from: &str) -> crate::Result<Self> {
        match from {
          $(
            stringify!($variant_ident_fixed)
              | stringify!($variant_n_fixed)
              $(| $variant_str_fixed)?
              $(| $variant_str_fixed_n)* =>
            {
              Ok(Self::$variant_ident_fixed)
            },
          )*
          _ => Err(crate::Error::UnexpectedString { length: from.len() }),
        }
      }
    }

    impl TryFrom<&[u8]> for $enum_ident {
      type Error = crate::Error;

      #[inline]
      fn try_from(from: &[u8]) -> crate::Result<Self> {
        match from {
          $(
            from if from == stringify!($variant_ident_fixed).as_bytes()
              || from == stringify!($variant_n_fixed).as_bytes()
              $(|| from == $variant_str_fixed.as_bytes())?
              $(|| from == $variant_str_fixed_n.as_bytes())* =>
            {
              Ok(Self::$variant_ident_fixed)
            },
          )*
          _ => Err(crate::Error::UnexpectedString { length: from.len() }),
        }
      }
    }
  }
}

macro_rules! _debug {
  ($($tt:tt)+) => {
    #[cfg(feature = "tracing")]
    tracing::debug!($($tt)+);
  };
}

macro_rules! doc_bad_format {
  () => {
    "Couldn't create a new instance using `Arguments`."
  };
}

macro_rules! doc_many_elems_cap_overflow {
  () => {
    "There is no capacity left to insert a set of new elements."
  };
}

macro_rules! doc_out_of_bounds_params {
  () => {
    "Received parameters lead to outcomes that can't accurately represent the underlying data."
  };
}

macro_rules! doc_reserve_overflow {
  () => {
    "It was not possible to reserve more memory"
  };
}

macro_rules! doc_single_elem_cap_overflow {
  () => {
    "There is no capacity left to insert a new element."
  };
}

macro_rules! _internal_buffer_doc {
  () => {
    "Buffer used for internal operations."
  };
}

macro_rules! _internal_doc {
  () => {
    "Internal element not meant for public usage."
  };
}

macro_rules! _iter4 {
  ($slice:expr, $init:block, |$elem:ident| $block:block) => {{
    let mut iter = crate::misc::ArrayChunks::new($slice);
    for [a, b, c, d] in iter.by_ref() {
      $init
      let $elem = a;
      $block
      let $elem = b;
      $block
      let $elem = c;
      $block
      let $elem = d;
      $block
    }
    for elem in iter.into_remainder() {
      let $elem = elem;
      $block
    }
  }};
}

macro_rules! _iter4_mut {
  ($slice:expr, $init:block, |$elem:ident| $block:block) => {{
    let mut iter = crate::misc::ArrayChunksMut::new($slice);
    for [a, b, c, d] in iter.by_ref() {
      $init
      let $elem = a;
      $block
      let $elem = b;
      $block
      let $elem = c;
      $block
      let $elem = d;
      $block
    }
    for elem in iter.into_remainder() {
      let $elem = elem;
      $block
    }
  }};
}

macro_rules! _max_continuation_frames {
  () => {
    16
  };
}

macro_rules! _max_frames_mismatches {
  () => {
    32
  };
}

macro_rules! _simd {
  (
    fallback => $fallback:expr,
    128 => $_128:expr,
    256 => $_256:expr,
    512 => $_512:expr $(,)?
  ) => {{
    #[cfg(target_feature = "avx512f")]
    let rslt = $_512;

    #[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
    let rslt = $_256;

    #[cfg(all(
      target_feature = "neon",
      not(any(target_feature = "avx2", target_feature = "avx512f"))
    ))]
    let rslt = $_128;

    #[cfg(all(
      target_feature = "sse2",
      not(any(target_feature = "avx2", target_feature = "avx512f", target_feature = "neon"))
    ))]
    let rslt = $_128;

    #[cfg(not(any(
      target_feature = "avx2",
      target_feature = "avx512f",
      target_feature = "neon",
      target_feature = "sse2"
    )))]
    let rslt = $fallback;

    rslt
  }};
}

macro_rules! _simd_bytes {
  (
    ($align:ident, $bytes:expr),
    (|$bytes_ident_a:ident| $bytes_expr_a:expr, |$bytes_ident_b:ident| $bytes_expr_b:expr),
    |$_16:ident| $_128:expr,
    |$_32:ident| $_256:expr,
    |$_64:ident| $_512:expr  $(,)?
  ) => {{
    // SAFETY: Changing a sequence of `u8` should be fine
    let (_prefix, _chunks, _suffix) = unsafe { $bytes.$align() };
    _simd! {
      fallback => {
        let $bytes_ident_a = _prefix;
        $bytes_expr_a;
      }
      128 => {
        let $bytes_ident_a = _prefix;
        $bytes_expr_a;
        let _: [[u8; 64]] = *_chunks;
        let $_16 = _chunks;
        $_128
        let $bytes_ident_b = _suffix;
        $bytes_expr_b;
      },
      256 => {
        let $bytes_ident_a = _prefix;
        $bytes_expr_a;
        let _: [[u8; 32]] = *_chunks;
        let $_16 = _chunks;
        $_128
        let $bytes_ident_b = _suffix;
        $bytes_expr_b;
      },
      512 => {
        let $bytes_ident_a = _prefix;
        $bytes_expr_a;
        let _: [[u8; 16]] = *_chunks;
        let $_16 = _chunks;
        $_128
        let $bytes_ident_b = _suffix;
        $bytes_expr_b;
      },
    }
  }};
}

macro_rules! _trace {
  ($($tt:tt)+) => {
    #[cfg(feature = "tracing")]
    tracing::trace!($($tt)+)
  };
}

macro_rules! _trace_span {
  ($($tt:tt)+) => {
    crate::misc::_Span::_new(
      #[cfg(feature = "tracing")]
      tracing::trace_span!($($tt)+),
      #[cfg(not(feature = "tracing"))]
      ()
    )
  };
}

macro_rules! _vector {
  ($($tt:tt)+) => {
    crate::misc::Vector::from_vec(alloc::vec![$($tt)+])
  };
}
