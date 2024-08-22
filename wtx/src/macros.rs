macro_rules! _conn_params_methods {
  ($($field:tt)+) => {
    /// The initial amount of "credit" a counterpart can have for sending data.
    #[inline]
    #[must_use]
    pub fn initial_window_len(mut self, elem: u32) -> Self {
      self.$($field)+.initial_window_len = elem;
      self
    }

    /// The maximum number of data bytes or the sum of all frames that composed the body data.
    #[inline]
    #[must_use]
    pub fn max_body_len(mut self, elem: u32) -> Self {
      self.$($field)+.max_body_len = elem;
      self
    }

    /// Maximum number of active concurrent streams
    #[inline]
    #[must_use]
    pub fn max_concurrent_streams_num(mut self, elem: u32) -> Self {
      self.$($field)+.max_concurrent_streams_num = elem;
      self
    }

    /// Maximum frame ***payload*** length
    #[inline]
    #[must_use]
    pub fn max_frame_len(mut self, elem: u32) -> Self {
      self.$($field)+.max_frame_len = elem;
      self
    }

    /// The maximum number of bytes of the entire set of headers in a request/response.
    #[inline]
    #[must_use]
    pub fn max_headers_len(mut self, elem: u32) -> Self {
      self.$($field)+.max_headers_len = elem;
      self
    }

    /// Maximum number of receiving streams
    ///
    /// Servers only. Prevents clients from opening more than the specified number of streams.
    #[inline]
    #[must_use]
    pub fn max_recv_streams_num(mut self, elem: u32) -> Self {
      self.$($field)+.max_recv_streams_num = elem;
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
        let rslt = match from {
          $(
            stringify!($variant_ident_fixed)
              | stringify!($variant_n_fixed)
              $(| $variant_str_fixed)?
              $(| $variant_str_fixed_n)* =>
            {
              Self::$variant_ident_fixed
            },
          )*
          _ => return Err(crate::Error::UnexpectedString { length: from.len() }),
        };
        Ok(rslt)
      }
    }

    impl TryFrom<&[u8]> for $enum_ident {
      type Error = crate::Error;

      #[inline]
      fn try_from(from: &[u8]) -> crate::Result<Self> {
        $(
          if from == stringify!($variant_ident_fixed).as_bytes()
            || from == stringify!($variant_n_fixed).as_bytes()
            $(|| from == $variant_str_fixed.as_bytes())?
            $(|| from == $variant_str_fixed_n.as_bytes())*
          {
            return Ok(Self::$variant_ident_fixed);
          }
        )*
        Err(crate::Error::UnexpectedString { length: from.len() })
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
    2_147_483_647
  };
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
