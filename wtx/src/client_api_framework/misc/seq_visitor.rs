use core::marker::PhantomData;

pub(crate) struct _SeqVisitor<E, F, T>(F, PhantomData<(E, T)>);

impl<E, F, T> _SeqVisitor<E, F, T> {
  pub(crate) fn _new(cb: F) -> Self {
    Self(cb, PhantomData)
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::client_api_framework::misc::seq_visitor::_SeqVisitor;
  use core::{
    any::type_name,
    fmt::{Display, Formatter},
  };
  use serde::{
    de::{Error as _, SeqAccess, Visitor},
    Deserialize,
  };

  impl<'de, E, F, T> Visitor<'de> for _SeqVisitor<E, F, T>
  where
    E: Display,
    F: FnMut(T) -> Result<(), E>,
    T: Deserialize<'de>,
  {
    type Value = ();

    fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
      formatter.write_fmt(format_args!("generic sequence of {}", type_name::<T>()))
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
      A: SeqAccess<'de>,
    {
      while let Some(elem) = seq.next_element::<T>()? {
        (self.0)(elem).map_err(A::Error::custom)?;
      }
      Ok(())
    }
  }
}
