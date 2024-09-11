macro_rules! impl_0_16 {
  ($( ($($T:ident)*) )+) => {
    #[cfg(feature = "database")]
    mod database {
      use crate::database::{Database, Encode, RecordValues, encode};

      $(
        impl<DB, $($T,)*> RecordValues<DB> for ($( $T, )*)
        where
          DB: Database,
          $($T: Encode<DB>,)*
        {
          #[inline]
          fn encode_values<'buffer, 'tmp, AUX>(
            &mut self,
            _aux: &mut AUX,
            _ev: &mut DB::EncodeValue<'buffer, 'tmp>,
            mut _prefix_cb: impl FnMut(&mut AUX, &mut DB::EncodeValue<'buffer, 'tmp>) -> usize,
            mut _suffix_cb: impl FnMut(&mut AUX, &mut DB::EncodeValue<'buffer, 'tmp>, bool, usize) -> usize,
          ) -> Result<usize, DB::Error> {
            let mut _n: usize = 0;
            $(
              ${ignore($T)}
              encode(
                _aux,
                &self.${index()},
                _ev,
                &mut _n,
                &mut _prefix_cb,
                &mut _suffix_cb
              )?;
            )*
            Ok(_n)
          }

          #[inline]
          fn len(&self) -> usize {
            const { 0 $(+ { const $T: usize = 1; $T })* }
          }
        }
      )+
    }

    #[cfg(feature = "http-server-framework")]
    mod http_server_framework {
      use crate::{
        http::{
          HttpError, Request, ReqResData, ReqResDataMut, Response, StatusCode,
          server_framework::{ConnAux, ReqAux, ReqMiddleware, ResMiddleware, PathManagement, PathParams}
        },
        misc::{ArrayVector, Vector}
      };

      $(
        impl<$($T,)*> ConnAux for ($($T,)*)
        where
          $($T: ConnAux,)*
        {
          type Init = ($(${ignore($T)} $T::Init,)*);

          #[inline]
          fn conn_aux(_init: Self::Init) -> crate::Result<Self> {
            Ok(($( ${ignore($T)} $T::conn_aux(_init.${index()})?, )*))
          }
        }

        impl<$($T,)*> ReqAux for ($($T,)*)
        where
          $($T: ReqAux,)*
        {
          type Init = ($(${ignore($T)} $T::Init,)*);

          #[inline]
          fn req_aux<RRD>(_init: Self::Init, _req: &mut Request<RRD>) -> crate::Result<Self>
          where
            RRD: ReqResDataMut
          {
            Ok(($( ${ignore($T)} $T::req_aux(_init.${index()}, _req)?, )*))
          }
        }

        impl<$($T,)* CA, ERR, RA, RRD> ReqMiddleware<CA, ERR, RA, RRD> for ($($T,)*)
        where
          $($T: ReqMiddleware<CA, ERR, RA, RRD>,)*
          ERR: From<crate::Error>
        {
          #[inline]
          async fn apply_req_middleware(&self, _ca: &mut CA, _ra: &mut RA, _req: &mut Request<RRD>) -> Result<(), ERR> {
            $( ${ignore($T)} self.${index()}.apply_req_middleware(_ca, _ra, _req).await?; )*
            Ok(())
          }
        }

        impl<$($T,)* CA, ERR, RA, RRD> ResMiddleware<CA, ERR, RA, RRD> for ($($T,)*)
        where
          $($T: ResMiddleware<CA, ERR, RA, RRD>,)*
          ERR: From<crate::Error>
        {
          #[inline]
          async fn apply_res_middleware(&self, _ca: &mut CA, _ra: &mut RA, mut _res: Response<&mut RRD>) -> Result<(), ERR> {
            $({
              let local_res = Response {
                rrd: &mut *_res.rrd,
                status_code: _res.status_code,
                version: _res.version,
              };
              ${ignore($T)} self.${index()}.apply_res_middleware(_ca, _ra, local_res).await?;
            })*
            Ok(())
          }
        }

        impl<$($T,)* CA, ERR, RA, RRD> PathManagement<CA, ERR, RA, RRD> for ($(PathParams<$T>,)*)
        where
          $($T: PathManagement<CA, ERR, RA, RRD>,)*
          ERR: From<crate::Error>,
          RRD: ReqResData
        {
          const IS_ROUTER: bool = false;

          #[inline]
          async fn manage_path(
            &self,
            _ca: &mut CA,
            path_defs: (u8, &[(&'static str, u8)]),
            _ra: &mut RA,
            _req: &mut Request<RRD>,
          ) -> Result<StatusCode, ERR> {
            match path_defs.1.get(usize::from(path_defs.0)).map(|el| el.1) {
              $(
                ${ignore($T)}
                Some(${index()}) => {
                  return self
                    .${index()}
                    .value
                    .manage_path(_ca, (path_defs.0.wrapping_add(1), path_defs.1), _ra, _req)
                    .await;
                }
              )*
              _ => Err(ERR::from(HttpError::UriMismatch.into()))
            }
          }

          #[inline]
          fn paths_indices(
            &self,
            _prev: ArrayVector<(&'static str, u8), 8>,
            _vec: &mut Vector<ArrayVector<(&'static str, u8), 8>>
          ) -> crate::Result<()> {
            let mut _idx: u8 = 0;
            $({
              ${ignore($T)}
              let mut local_prev = _prev.clone();
              local_prev.push((self.${index()}.full_path, _idx))?;
              if $T::IS_ROUTER {
                self.${index()}.value.paths_indices(local_prev, _vec)?;
              } else {
                _vec.push(local_prev)?;
              }
              _idx = _idx.wrapping_add(1);
            })*
            Ok(())
          }
        }
      )+
    }

    mod lease {
      use crate::misc::{Lease, LeaseMut};

      $(
        impl<$($T,)*> Lease<Self> for ($( $T, )*) {
          #[inline]
          fn lease(&self) -> &Self {
            self
          }
        }

        impl<$($T,)*> LeaseMut<Self> for ($( $T, )*) {
          #[inline]
          fn lease_mut(&mut self) -> &mut Self {
            self
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
        impl<'de, $($T,)* ERR> Decode<'de, Postgres<ERR>> for ($( $T, )*)
        where
          $($T: Decode<'de, Postgres<ERR>>,)*
          ERR: From<crate::Error>,
        {
          #[inline]
          fn decode(dv: &DecodeValue<'de>) -> Result<Self, ERR> {
            let mut _sd = StructDecoder::<ERR>::new(dv);
            Ok((
              $( _sd.decode::<$T>()?, )*
            ))
          }
        }

        impl<$($T,)* ERR> Encode<Postgres<ERR>> for ($( $T, )*)
        where
          $($T: Encode<Postgres<ERR>> + Typed<Postgres<ERR>>,)*
          ERR: From<crate::Error>,
        {
          #[inline]
          fn encode(&self, _ev: &mut EncodeValue<'_, '_>) -> Result<(), ERR> {
            let mut _ev = StructEncoder::<ERR>::new(_ev)?;
            $(
              ${ignore($T)}
              _ev = _ev.encode(&self.${index()})?;
            )*
            Ok(())
          }
        }
      )+
    }
  }
}

impl_0_16! {
  ()
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
