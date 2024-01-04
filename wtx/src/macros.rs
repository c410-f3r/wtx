macro_rules! create_enum {
  (
    $(#[$container_mac:meta])*
    $v:vis enum $enum_ident:ident<$n:ty> {
      $(
        $(#[$variant_mac_fixed:meta])*
        $variant_ident_fixed:ident = ($variant_n_fixed:literal $(, $variant_str_fixed:literal)?)
      ),* $(,)?

      $(
        @
        $(#[$variant_mac_range:meta])*
        $variant_ident_range:ident($variant_ident_value:pat) = ($variant_n_range:literal $(, $variant_str_range:literal)?)
      ),*
    }
  ) => {
    $(#[$container_mac])*
    $v enum $enum_ident {
      $($(#[$variant_mac_fixed])* $variant_ident_fixed,)*
      $($(#[$variant_mac_range])* $variant_ident_range($n),)*
    }

    impl $enum_ident {
      #[inline]
      /// The total number of variants
      pub const fn len() -> usize {
        let mut len: usize = 0;
        $({
          let _ = $variant_n_fixed;
          len = len.wrapping_add(1);
        })*
        $({
          let _ = $variant_n_range;
          len = len.wrapping_add(1);
        })*
        len
      }

      /// See [crate::EnumVarStrings].
      #[inline]
      pub const fn strings(&self) -> crate::misc::EnumVarStrings {
        match self {
          $(
            $enum_ident::$variant_ident_fixed => crate::misc::EnumVarStrings {
              custom: {
                #[allow(unused_assignments, unused_mut)]
                let mut rslt = "";
                $(rslt = $variant_str_fixed;)?
                rslt
              },
              ident: stringify!($variant_ident_fixed),
              number: stringify!($variant_n_fixed),
            },
          )*
          $(
            $enum_ident::$variant_ident_range(_) => crate::misc::EnumVarStrings {
              custom: {
                #[allow(unused_assignments, unused_mut)]
                let mut rslt = "";
                $(rslt = $variant_str_range;)?
                rslt
              },
              ident: stringify!($variant_ident_range),
              number: stringify!($variant_n_range),
            },
          )*
        }
      }
    }

    #[allow(
      // Macro shouldn't control what the outside uses.
      unused_qualifications
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
          $($enum_ident::$variant_ident_range(elem) => elem,)*
        }
      }
    }

    impl TryFrom<$n> for $enum_ident {
      type Error = crate::Error;

      #[inline]
      fn try_from(from: $n) -> crate::Result<Self> {
        let rslt = match from {
          $($variant_n_fixed => Self::$variant_ident_fixed,)*
          $($variant_ident_value => Self::$variant_ident_range(from),)*
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
            stringify!($variant_ident_fixed) | stringify!($variant_n_fixed) $(| $variant_str_fixed)?  => {
              Self::$variant_ident_fixed
            },
          )*
          _ => return Err(crate::Error::UnexpectedString { length: from.len() }),
        };
        Ok(rslt)
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

macro_rules! _iter4_mut {
  ($slice:expr, |$elem:ident| $block:block) => {{
    let mut iter = crate::misc::ArrayChunksMut::_new($slice);
    for [a, b, c, d] in iter.by_ref() {
      let $elem = a;
      $block;
      let $elem = b;
      $block;
      let $elem = c;
      $block;
      let $elem = d;
      $block;
    }
    for elem in iter._into_remainder() {
      let $elem = elem;
      $block;
    }
  }};
}
