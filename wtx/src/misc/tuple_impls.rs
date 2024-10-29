// FIXME(STABLE): macro_metavar_expr

macro_rules! impl_0_16 {
  ($( [$($T:ident($N:tt))*] )+) => {
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
              encode(
                _aux,
                &self.$N,
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
          HttpError, Request, ReqResBuffer,  Response, StatusCode,
          server_framework::{ConnAux, StreamAux, ReqMiddleware, ResMiddleware, PathManagement, PathParams}
        },
        misc::{ArrayVector, Vector}
      };

      $(
        impl<$($T,)*> ConnAux for ($($T,)*)
        where
          $($T: ConnAux,)*
        {
          type Init = ($($T::Init,)*);

          #[inline]
          fn conn_aux(_init: Self::Init) -> crate::Result<Self> {
            Ok(($( $T::conn_aux(_init.$N)?, )*))
          }
        }

        impl<$($T,)*> StreamAux for ($($T,)*)
        where
          $($T: StreamAux,)*
        {
          type Init = ($($T::Init,)*);

          #[inline]
          fn req_aux(_init: Self::Init, _req: &mut Request<ReqResBuffer>) -> crate::Result<Self> {
            Ok(($( $T::req_aux(_init.$N, _req)?, )*))
          }
        }

        impl<$($T,)* CA, ERR, RA> ReqMiddleware<CA, ERR, RA> for ($($T,)*)
        where
          $($T: ReqMiddleware<CA, ERR, RA>,)*
          ERR: From<crate::Error>
        {
          #[inline]
          async fn apply_req_middleware(&self, _ca: &mut CA, _ra: &mut RA, _req: &mut Request<ReqResBuffer>) -> Result<(), ERR> {
            $( self.$N.apply_req_middleware(_ca, _ra, _req).await?; )*
            Ok(())
          }
        }

        impl<$($T,)* CA, ERR, RA> ResMiddleware<CA, ERR, RA> for ($($T,)*)
        where
          $($T: ResMiddleware<CA, ERR, RA>,)*
          ERR: From<crate::Error>
        {
          #[inline]
          async fn apply_res_middleware(&self, _ca: &mut CA, _ra: &mut RA, mut _res: Response<&mut ReqResBuffer>) -> Result<(), ERR> {
            $({
              let local_res = Response {
                rrd: &mut *_res.rrd,
                status_code: _res.status_code,
                version: _res.version,
              };
              self.$N.apply_res_middleware(_ca, _ra, local_res).await?;
            })*
            Ok(())
          }
        }

        impl<$($T,)* CA, ERR, RA> PathManagement<CA, ERR, RA> for ($(PathParams<$T>,)*)
        where
          $($T: PathManagement<CA, ERR, RA>,)*
          ERR: From<crate::Error>,
        {
          const IS_ROUTER: bool = false;

          #[inline]
          async fn manage_path(
            &self,
            _ca: &mut CA,
            _path_defs: (u8, &[(&'static str, u8)]),
            _ra: &mut RA,
            _req: &mut Request<ReqResBuffer>,
          ) -> Result<StatusCode, ERR> {
            #[cfg(feature = "matchit")]
            match _path_defs.1.get(usize::from(_path_defs.0)).map(|el| el.1) {
              $(
                Some($N) => {
                  return self
                    .$N
                    .value
                    .manage_path(_ca, (_path_defs.0.wrapping_add(1), _path_defs.1), _ra, _req)
                    .await;
                }
              )*
              _ => Err(ERR::from(HttpError::UriMismatch.into()))
            }
            #[cfg(not(feature = "matchit"))]
            match _req.rrd.uri.path() {
              $(
                elem if elem == self.$N.full_path => {
                  return self
                    .$N
                    .value
                    .manage_path(_ca, (_path_defs.0.wrapping_add(1), _path_defs.1), _ra, _req)
                    .await
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
              let mut local_prev = _prev.clone();
              local_prev.push((self.$N.full_path, _idx))?;
              if $T::IS_ROUTER {
                self.$N.value.paths_indices(local_prev, _vec)?;
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
              _ev = _ev.encode(&self.$N)?;
            )*
            Ok(())
          }
        }
      )+
    }
  }
}

impl_0_16! {
  []
  [A(0)]
  [A(0) B(1)]
  [A(0) B(1) C(2)]
  [A(0) B(1) C(2) D(3)]
  [A(0) B(1) C(2) D(3) E(4)]
  [A(0) B(1) C(2) D(3) E(4) F(5)]
  [A(0) B(1) C(2) D(3) E(4) F(5) G(6)]
  [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7)]
  [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8)]
  [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9)]
  [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10)]
  [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11)]
  [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12)]
  [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13)]
  [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14)]
  [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15)]
}
