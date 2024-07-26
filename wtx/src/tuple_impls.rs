macro_rules! tuple_impls {
  ($( ($($T:ident)+) )+) => {
    #[cfg(feature = "database")]
    mod database {
      use crate::database::{Database, Encode, RecordValues, encode};
      $(
        impl<DB, $($T),+> RecordValues<DB> for ($( $T, )+)
        where
          DB: Database,
          $($T: Encode<DB>,)+
        {
          #[inline]
          fn encode_values<'buffer, 'tmp, AUX>(
            &mut self,
            aux: &mut AUX,
            ev: &mut DB::EncodeValue<'buffer, 'tmp>,
            mut prefix_cb: impl FnMut(&mut AUX, &mut DB::EncodeValue<'buffer, 'tmp>) -> usize,
            mut suffix_cb: impl FnMut(&mut AUX, &mut DB::EncodeValue<'buffer, 'tmp>, bool, usize) -> usize,
          ) -> Result<usize, DB::Error> {
            let mut n: usize = 0;
            $(
              ${ignore($T)}
              encode(
                aux,
                &self.${index()},
                ev,
                &mut n,
                &mut prefix_cb,
                &mut suffix_cb
              )?;
            )+
            Ok(n)
          }
          #[inline]
          fn len(&self) -> usize {
            const { 0 $(+ { const $T: usize = 1; $T })+ }
          }
        }
      )+
    }

    #[cfg(feature = "postgres")]
    mod postgres {
      use crate::database::{
        Decode, Encode, Typed,
        client::postgres::{DecodeValue, EncodeValue, Postgres, StructDecoder, StructEncoder}
      };
      $(
        impl<'de, ERR, $($T),+> Decode<'de, Postgres<ERR>> for ($( $T, )+)
        where
          ERR: From<crate::Error>,
          $(for<'local_de> $T: Decode<'local_de, Postgres<ERR>>,)+
        {
          #[inline]
          fn decode(dv: &DecodeValue<'de>) -> Result<Self, ERR> {
            let mut sd = StructDecoder::<ERR>::new(dv);
            Ok((
              $( sd.decode::<$T>()?, )+
            ))
          }
        }
        impl<ERR, $($T),+> Encode<Postgres<ERR>> for ($( $T, )+)
        where
          ERR: From<crate::Error>,
          $($T: Encode<Postgres<ERR>> + Typed<Postgres<ERR>>,)+
        {
          #[inline]
          fn encode(&self, ev: &mut EncodeValue<'_, '_>) -> Result<(), ERR> {
            let mut _ev = StructEncoder::<ERR>::new(ev)?;
            $(
              ${ignore($T)}
              _ev = _ev.encode(&self.${index()})?;
            )+
            Ok(())
          }
        }
      )+
    }
  }
}

tuple_impls! {
  (A)
  (A B)
  (A B C)
  (A B C D)
  (A B C D E)
  (A B C D E F)
  (A B C D E F G)
  (A B C D E F G H)
  (A B C D E F G H I)
  (A B C D E F G H I J)
  (A B C D E F G H I J K)
  (A B C D E F G H I J K L)
  (A B C D E F G H I J K L M)
  (A B C D E F G H I J K L M N)
  (A B C D E F G H I J K L M N O)
  (A B C D E F G H I J K L M N O P)
}
