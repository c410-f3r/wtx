use core::marker::PhantomData;

pub(crate) struct _SeqVisitor<A, E, F, R, T>(F, R, PhantomData<(A, E, T)>);

impl<A, E, F, R, T> _SeqVisitor<A, E, F, R, T> {
  pub(crate) fn _new(first: F, rest: R) -> Self {
    Self(first, rest, PhantomData)
  }
}

#[cfg(feature = "serde")]
mod serde {
  use crate::data_transformation::seq_visitor::_SeqVisitor;
  use core::{
    any::type_name,
    fmt::Formatter, marker::PhantomData,
  };
  use serde::{
    de::{Error as _, SeqAccess, Visitor},
    Deserialize,
  };

  impl<'de, A, E, F, R, T> Visitor<'de> for _SeqVisitor<A, E, F, R, T>
  where
    E: From<SA::Error>,
    F: FnOnce(T) -> Result<A, E>,
    R: FnMut(A, T) -> Result<A, E>,
    T: Deserialize<'de>,
  {
    type Value = impl Iterator<Item = Result<T, E>>;

    #[inline]
    fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
      formatter.write_fmt(format_args!("generic sequence of {}", type_name::<T>()))
    }

    #[inline]
    fn visit_seq<SA>(mut self, mut seq: SA) -> Result<Self::Value, SA::Error>
    where
      SA: SeqAccess<'de>,
    {
      struct Iter<'de, E, SA, T> {
        phantom: PhantomData<(&'de (), E, T)>,
        sa: SA
      }

      impl<'de, E, SA, T> Iterator for Iter<'de, E, SA, T>
      where
        E: From<SA::Error>,
        SA: SeqAccess<'de>,
        T: Deserialize<'de>,
      {
        type Item = Result<T, E>;
      
        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
          self.sa.next_element::<T>().map_err(E::from).transpose()
        }
      }
      
      Ok(Iter {
        phantom: PhantomData,
        sa: seq
      })
    }
  }
}
