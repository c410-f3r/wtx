use crate::misc::{Lease, ToOwned};
use core::ops::Deref;

/// A clone-on-write smart pointer.
#[derive(Debug)]
pub enum Cow<'any, B, O>
where
  B: ToOwned<O> + ?Sized,
  O: Lease<B>,
{
  /// Borrowed data.
  Borrowed(&'any B),
  /// Owned data.
  Owned(O),
}

impl<B, O> AsRef<B> for Cow<'_, B, O>
where
  B: ToOwned<O> + ?Sized,
  O: Lease<B>,
{
  #[inline]
  fn as_ref(&self) -> &B {
    self
  }
}

impl<B, O> Deref for Cow<'_, B, O>
where
  B: ToOwned<O> + ?Sized,
  O: Lease<B>,
{
  type Target = B;

  #[inline]
  fn deref(&self) -> &B {
    match self {
      Self::Borrowed(el) => el,
      Self::Owned(el) => el.lease(),
    }
  }
}

impl<B, O> Lease<B> for Cow<'_, B, O>
where
  B: ToOwned<O> + ?Sized,
  O: Lease<B>,
{
  #[inline]
  fn lease(&self) -> &B {
    self
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::misc::{Cow, Lease, ToOwned};
  use alloc::string::String;
  use core::{any::type_name, fmt::Formatter, marker::PhantomData};
  use serde::{Deserialize, Deserializer, Serialize, Serializer};

  impl<B, O> Serialize for Cow<'_, B, O>
  where
    B: Serialize + ToOwned<O> + ?Sized,
    O: Lease<B>,
  {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      self.as_ref().serialize(serializer)
    }
  }

  impl<'any, 'de, O> Deserialize<'de> for Cow<'any, str, O>
  where
    'de: 'any,
    str: ToOwned<O>,
    O: Lease<str>,
  {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      pub(crate) struct LocalVisitor<'any, B, O>
      where
        B: ?Sized,
      {
        borrowed: PhantomData<&'any B>,
        owned: PhantomData<O>,
      }

      impl<B, O> LocalVisitor<'_, B, O>
      where
        B: ?Sized,
      {
        #[inline]
        fn do_expecting(formatter: &mut Formatter<'_>) -> core::fmt::Result {
          formatter.write_fmt(format_args!(
            "a borrowed `{}` or an owned `{}`",
            type_name::<B>(),
            type_name::<O>()
          ))
        }
      }

      impl<'any, 'de, O> serde::de::Visitor<'de> for LocalVisitor<'any, str, O>
      where
        'de: 'any,
        str: ToOwned<O>,
        O: Lease<str>,
      {
        type Value = Cow<'any, str, O>;

        #[inline]
        fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
          Self::do_expecting(formatter)
        }

        #[inline]
        fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where
          E: serde::de::Error,
        {
          Ok(Cow::Borrowed(v))
        }

        #[inline]
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
          E: serde::de::Error,
        {
          self.visit_string(v.into())
        }

        #[inline]
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
          E: serde::de::Error,
        {
          Ok(Cow::Owned(v.as_str().to_owned().map_err(serde::de::Error::custom)?))
        }
      }

      deserializer.deserialize_str(LocalVisitor { borrowed: PhantomData, owned: PhantomData })
    }
  }
}
