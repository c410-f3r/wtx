/// Implements a bunch of auxiliary methods for enums.
#[macro_export]
macro_rules! create_enum {
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
      /// An array that contains all variants
      $v fn all() -> [Self; { Self::len() }] {
        [$( $enum_ident::$variant_ident_fixed, )*]
      }

      #[inline]
      /// The total number of variants
      $v const fn len() -> usize {
        const { 0 $( + { let _: $n = $variant_n_fixed; 1 })* }
      }

      /// See [`$crate::misc::EnumVarStrings`].
      #[inline]
      $v const fn strings(&self) -> $crate::misc::EnumVarStrings<{
        #[allow(unused_mut, reason = "macro stuff")]
        let mut n;
        $({
          #[allow(unused_mut, reason = "repetition can be empty")]
          let mut local_n = 0;
          let _: $n = $variant_n_fixed;
          $({ let _ = $variant_str_fixed; local_n += 1; })?
          $({ let _ = $variant_str_fixed_n; local_n += 1; })*
          #[allow(unused_assignments, reason = "repetition can be empty")]
          { n = local_n; }
        })*
        n
      }> {
        match self {
          $(
            $enum_ident::$variant_ident_fixed => $crate::misc::EnumVarStrings {
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
      type Err = $crate::Error;

      #[inline]
      fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
      }
   }

    impl TryFrom<$n> for $enum_ident {
      type Error = $crate::Error;

      #[inline]
      fn try_from(from: $n) -> $crate::Result<Self> {
        let rslt = match from {
          $($variant_n_fixed => Self::$variant_ident_fixed,)*
          _ => return Err($crate::Error::UnexpectedUint { received: from.into() }),
        };
        Ok(rslt)
      }
    }

    impl TryFrom<&str> for $enum_ident {
      type Error = $crate::Error;

      #[inline]
      fn try_from(from: &str) -> $crate::Result<Self> {
        from.as_bytes().try_into()
      }
    }

    impl TryFrom<&[u8]> for $enum_ident {
      type Error = $crate::Error;

      #[inline]
      fn try_from(from: &[u8]) -> $crate::Result<Self> {
        match from {
          $(
            from if from == const { stringify!($variant_ident_fixed).as_bytes() }
              || from == const { stringify!($variant_n_fixed).as_bytes() }
              $(|| from == const { $variant_str_fixed.as_bytes() })?
              $(|| from == const { $variant_str_fixed_n.as_bytes() })* =>
            {
              Ok(Self::$variant_ident_fixed)
            },
          )*
          _ => Err($crate::Error::UnexpectedBytes {
            length: from.len().try_into().unwrap_or(u16::MAX),
            ty: core::any::type_name::<Self>().split("::").last().and_then(|el| el.get(..8)).unwrap_or_default().try_into()?,
          }),
        }
      }
    }
  }
}

/// Creates a vector containing the arguments.
#[macro_export]
macro_rules! vector {
  ($($tt:tt)*) => {
    $crate::collection::Vector::from_vec(alloc::vec![$($tt)*])
  };
}

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

macro_rules! _debug {
  ($($tt:tt)+) => {
    #[cfg(feature = "tracing")]
    tracing::debug!($($tt)+);
  };
}

macro_rules! doc_epoch {
  () => {
    "`secs` is the amount of seconds passed since the UNIX epoch. This parameter is only relevant
    for few `no_std` devices that can't natively provide time measurements, as such, regular users
    should simply pass `zero`."
  };
}

macro_rules! doc_many_elems_cap_overflow {
  () => {
    "There is no capacity left to insert a set of new elements."
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
    let (chunks, rem) = $slice.as_chunks();
    let mut iter = chunks.iter();
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
    for elem in rem {
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
    8 => $_8:expr,
    16 => $_16:expr,
    32 => $_32:expr,
    64 => $_64:expr $(,)?
  ) => {{
    #[cfg(not(any(
      target_feature = "avx2",
      target_feature = "avx512f",
      target_feature = "neon",
      target_feature = "sse2"
    )))]
    let rslt = $_8;

    #[cfg(all(
      target_feature = "neon",
      not(any(target_feature = "avx2", target_feature = "avx512f"))
    ))]
    let rslt = $_16;

    #[cfg(all(
      target_feature = "sse2",
      not(any(target_feature = "avx2", target_feature = "avx512f", target_feature = "neon"))
    ))]
    let rslt = $_16;

    #[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
    let rslt = $_32;

    #[cfg(target_feature = "avx512f")]
    let rslt = $_64;

    rslt
  }};
}

macro_rules! _simd_bytes {
  (
    ($align:ident, $bytes:expr),
    |$bytes_ident:ident| $bytes_expr:expr,
    |$before_align_ident:ident| $before_align_expr:expr,
    |$_16_ident:ident| $_16_expr:expr,
    |$_32_ident:ident| $_32_expr:expr,
    |$_64_ident:ident| $_64_expr:expr,
    |$after_align_ident:ident| $after_align_expr:expr $(,)?
  ) => {{
    // SAFETY: changing a sequence of `u8` should be fine
    let (_prefix, _chunks, _suffix) = unsafe { $bytes.$align() };
    _simd! {
      8 => {
        let _: [[u8; 8]] = *_chunks;
        let $bytes_ident = _prefix; $bytes_expr
        let $before_align_ident = $bytes_ident; $before_align_expr;
        let $_16_ident = _chunks; $_16_expr;
        let $after_align_ident = _suffix; $after_align_expr;
        let $bytes_ident = $after_align_ident; $bytes_expr
      },
      16 => {
        let _: [[u8; 16]] = *_chunks;
        let $bytes_ident = _prefix; $bytes_expr
        let $before_align_ident = $bytes_ident; $before_align_expr;
        let $_16_ident = _chunks; $_16_expr;
        let $after_align_ident = _suffix; $after_align_expr;
        let $bytes_ident = $after_align_ident; $bytes_expr
      },
      32 => {
        let _: [[u8; 32]] = *_chunks;
        let $bytes_ident = _prefix; $bytes_expr
        let $before_align_ident = $bytes_ident; $before_align_expr;
        let $_32_ident = _chunks; $_32_expr;
        let $after_align_ident = _suffix; $after_align_expr;
        let $bytes_ident = $after_align_ident; $bytes_expr
      },
      64 => {
        let _: [[u8; 64]] = *_chunks;
        let $bytes_ident = _prefix; $bytes_expr
        let $before_align_ident = $bytes_ident; $before_align_expr;
        let $_64_ident = _chunks; $_64_expr;
        let $after_align_ident = _suffix; $after_align_expr;
        let $bytes_ident = $after_align_ident; $bytes_expr
      },
    }
  }};
}

macro_rules! _simd_lanes {
  () => {{
    _simd! {
      8 => 8,
      16 => 16,
      32 => 32,
      64 => 64,
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
    crate::misc::span::Span::new(
      #[cfg(feature = "tracing")]
      tracing::trace_span!($($tt)+),
      #[cfg(not(feature = "tracing"))]
      ()
    )
  };
}
