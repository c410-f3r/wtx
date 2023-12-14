macro_rules! create_enum {
  (
    $(#[$mac:meta])*
    $v:vis enum $enum_ident:ident<$n:ty> {
      $($(#[$doc:meta])* $variant_ident:ident = ($variant_n:literal $(, $variant_str:literal)?)),* $(,)?
    }
  ) => {
    $(#[$mac])*
    $v enum $enum_ident {
      $($(#[$doc])* $variant_ident,)*
    }

    impl $enum_ident {
      #[inline]
      /// The total number of variants
      pub const fn len() -> usize {
        let mut len: usize = 0;
        $({
          let _ = $variant_n;
          len = len.wrapping_add(1);
        })*
        len
      }

      /// See [crate::EnumVarStrings].
      #[inline]
      pub const fn strings(&self) -> crate::misc::EnumVarStrings {
        match self {
          $(
            $enum_ident::$variant_ident => crate::misc::EnumVarStrings {
              custom: {
                #[allow(unused_assignments, unused_mut)]
                let mut rslt = "";
                $(rslt = $variant_str;)?
                rslt
              },
              ident: stringify!($variant_ident),
              number: stringify!($variant_n),
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
          $($enum_ident::$variant_ident => $variant_n,)*
        }
      }
    }

    impl TryFrom<$n> for $enum_ident {
      type Error = crate::Error;

      #[inline]
      fn try_from(from: $n) -> crate::Result<Self> {
        let rslt = match from {
          $($variant_n => Self::$variant_ident,)*
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
            stringify!($variant_ident) | stringify!($variant_n) $(| $variant_str)?  => {
              Self::$variant_ident
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
