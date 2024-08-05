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

    #[cfg(feature = "http-server-framework")]
    mod http_server_framework {
      use crate::{
        http::{
          HttpError, Request, ReqResData, Response,
          server_framework::{ReqMiddlewares, ResMiddlewares, PathManagement, Path}
        },
        misc::FnFut
      };

      $(
        impl<$($T,)+ ERR, RRD> ReqMiddlewares<ERR, RRD> for ($($T,)+)
        where
          $($T: for<'any> FnFut<&'any mut Request<RRD>, Result<(), ERR>>,)+
          ERR: From<crate::Error>
        {
          #[inline]
          async fn apply_req_middlewares(&self, req: &mut Request<RRD>) -> Result<(), ERR> {
            $( ${ignore($T)} (self.${index()})(req).await?; )+
            Ok(())
          }
        }

        impl<$($T,)+ ERR, RRD> ResMiddlewares<ERR, RRD> for ($($T,)+)
        where
          $($T: for<'any> FnFut<&'any mut Response<RRD>, Result<(), ERR>>,)+
          ERR: From<crate::Error>
        {
          #[inline]
          async fn apply_res_middlewares(&self, res: &mut Response<RRD>) -> Result<(), ERR> {
            $( ${ignore($T)} (self.${index()})(res).await?; )+
            Ok(())
          }
        }

        impl<$($T,)+ ERR, RRD> PathManagement<ERR, RRD> for ($(Path<$T>,)+)
        where
          $($T: PathManagement<ERR, RRD>,)+
          ERR: From<crate::Error>,
          RRD: ReqResData
        {
          #[inline]
          async fn manage_path(
            &self,
            _: bool,
            _: &'static str,
            req: Request<RRD>,
            [begin, end]: [usize; 2],
          ) -> Result<Response<RRD>, ERR> {
            match req.rrd.uri().as_str().get(begin..end).unwrap_or_default() {
              $(
                ${ignore($T)}
                elem if self.${index()}.name.starts_with(elem) => {
                  return self.${index()}.value.manage_path(false, self.${index()}.name, req, [begin, end]).await;
                }
              )+
              _ => return Err(ERR::from(HttpError::UriMismatch.into()))
            }
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
        impl<'de, $($T,)+ ERR> Decode<'de, Postgres<ERR>> for ($( $T, )+)
        where
          $(for<'local_de> $T: Decode<'local_de, Postgres<ERR>>,)+
          ERR: From<crate::Error>,
        {
          #[inline]
          fn decode(dv: &DecodeValue<'de>) -> Result<Self, ERR> {
            let mut sd = StructDecoder::<ERR>::new(dv);
            Ok((
              $( sd.decode::<$T>()?, )+
            ))
          }
        }

        impl<$($T,)+ ERR> Encode<Postgres<ERR>> for ($( $T, )+)
        where
          $($T: Encode<Postgres<ERR>> + Typed<Postgres<ERR>>,)+
          ERR: From<crate::Error>,
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
