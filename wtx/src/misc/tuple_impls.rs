// FIXME(STABLE): macro_metavar_expr

macro_rules! impl_tuples {
  ($( [$($T:ident($N:tt))*] )+) => {
    #[cfg(feature = "database")]
    mod database {
      use crate::database::{Database, RecordValues, Typed, record_values::encode};
      use crate::misc::Encode;

      $(
        impl<DB, $($T,)*> RecordValues<DB> for ($( $T, )*)
        where
          DB: Database<Aux = ()>,
          $($T: Encode<DB> + Typed<DB>,)*
        {
          #[inline]
          fn encode_values<'buffer, 'tmp, AUX>(
            &self,
            _aux: &mut AUX,
            _ew: &mut DB::EncodeWrapper<'buffer, 'tmp>,
            mut _prefix_cb: impl FnMut(&mut AUX, &mut DB::EncodeWrapper<'buffer, 'tmp>) -> usize,
            mut _suffix_cb: impl FnMut(&mut AUX, &mut DB::EncodeWrapper<'buffer, 'tmp>, bool, usize) -> usize,
          ) -> Result<usize, DB::Error> {
            let mut _n: usize = 0;
            $(
              encode(
                _aux,
                &self.$N,
                _ew,
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

          #[allow(unused_mut, reason = "0-arity tuple")]
          #[inline]
          fn walk(&self, mut _cb: impl FnMut(bool, Option<DB::Ty>) -> Result<(), DB::Error>) -> Result<(), DB::Error> {
            $( _cb(self.$N.is_null(), self.$N.runtime_ty())?; )*
            Ok(())
          }
        }
      )+
    }

    #[cfg(feature = "http-server-framework")]
    mod http_server_framework {
      use crate::{
        http::{
          OperationMode, HttpError, StatusCode, AutoStream, ManualStream, Request,
          ReqResBuffer, Response,
          server_framework::{ConnAux, Endpoint, Middleware, StreamAux, RouteMatch, EndpointNode, PathParams}
        },
        misc::{ArrayVector, Vector}
      };
      use core::ops::ControlFlow;

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
          fn stream_aux(_init: Self::Init) -> crate::Result<Self> {
            Ok(($( $T::stream_aux(_init.$N)?, )*))
          }
        }

        impl<$($T,)* CA, ERR, SA> Middleware<CA, ERR, SA> for ($($T,)*)
        where
          $($T: Middleware<CA, ERR, SA>,)*
          ERR: From<crate::Error>
        {
          type Aux = ($($T::Aux,)*);

          #[inline]
          fn aux(&self) -> Self::Aux {
            ($(self.$N.aux(),)*)
          }

          #[inline]
          async fn req(
            &self,
            _conn_aux: &mut CA,
            _mw_aux: &mut Self::Aux,
            _req: &mut Request<ReqResBuffer>,
            _stream_aux: &mut SA,
          ) -> Result<ControlFlow<StatusCode, ()>, ERR> {
            $({
              let rslt = self.$N.req(_conn_aux, &mut _mw_aux.$N, _req, _stream_aux).await?;
              if let ControlFlow::Break(status_code) = rslt {
                return Ok(ControlFlow::Break(status_code));
              }
            })*
            Ok(ControlFlow::Continue(()))
          }

          #[inline]
          async fn res(
            &self,
            _conn_aux: &mut CA,
            _mw_aux: &mut Self::Aux,
            _res: Response<&mut ReqResBuffer>,
            _stream_aux: &mut SA,
          ) -> Result<ControlFlow<StatusCode, ()>, ERR> {
            $({
              let local_res = Response {
                rrd: &mut *_res.rrd,
                status_code: _res.status_code,
                version: _res.version,
              };
              let rslt = self.$N.res(_conn_aux, &mut _mw_aux.$N, local_res, _stream_aux).await?;
              if let ControlFlow::Break(status_code) = rslt {
                return Ok(ControlFlow::Break(status_code));
              }
            })*
            Ok(ControlFlow::Continue(()))
          }
        }

        impl<$($T,)* CA, ERR, STREAM, SA> Endpoint<CA, ERR, STREAM, SA> for ($(PathParams<$T>,)*)
        where
          $($T: Endpoint<CA, ERR, STREAM, SA>,)*
          ERR: From<crate::Error>,
        {
          const OM: OperationMode = OperationMode::Auto;

          #[inline]
          async fn auto(
            &self,
            _auto_stream: &mut AutoStream<CA, SA>,
            _path_defs: (u8, &[RouteMatch]),
          ) -> Result<StatusCode, ERR> {
            #[cfg(feature = "matchit")]
            match _path_defs.1.get(usize::from(_path_defs.0)).map(|el| el.idx) {
              $(
                Some($N) => {
                  return self
                    .$N
                    .value
                    .auto(_auto_stream, (_path_defs.0.wrapping_add(1), _path_defs.1))
                    .await;
                }
              )*
              _ => Err(ERR::from(HttpError::UriMismatch.into()))
            }
            #[cfg(not(feature = "matchit"))]
            match _auto_stream.req.rrd.uri.path() {
              $(
                elem if elem == self.$N.full_path => {
                  return self
                    .$N
                    .value
                    .auto(_auto_stream, (_path_defs.0.wrapping_add(1), _path_defs.1))
                    .await
                }
              )*
              _ => Err(ERR::from(HttpError::UriMismatch.into()))
            }
          }

          #[inline]
          async fn manual(
            &self,
            _manual_stream: ManualStream<CA, STREAM, SA>,
            _path_defs: (u8, &[RouteMatch]),
          ) -> Result<(), ERR> {
            #[cfg(feature = "matchit")]
            match _path_defs.1.get(usize::from(_path_defs.0)).map(|el| el.idx) {
              $(
                Some($N) => {
                  return self
                    .$N
                    .value
                    .manual(_manual_stream, (_path_defs.0.wrapping_add(1), _path_defs.1))
                    .await;
                }
              )*
              _ => Err(ERR::from(HttpError::UriMismatch.into()))
            }
            #[cfg(not(feature = "matchit"))]
            match _manual_stream.req.rrd.uri.path() {
              $(
                elem if elem == self.$N.full_path => {
                  return self
                    .$N
                    .value
                    .manual(_manual_stream, (_path_defs.0.wrapping_add(1), _path_defs.1))
                    .await
                }
              )*
              _ => Err(ERR::from(HttpError::UriMismatch.into()))
            }
          }
        }

        impl<$($T,)* CA, ERR, STREAM, SA> EndpointNode<CA, ERR, STREAM, SA> for ($(PathParams<$T>,)*)
        where
          $($T: EndpointNode<CA, ERR, STREAM, SA>,)*
          ERR: From<crate::Error>,
        {
          const IS_ROUTER: bool = false;

          #[inline]
          fn paths_indices(
            &self,
            _prev: ArrayVector<RouteMatch, 4>,
            _vec: &mut Vector<ArrayVector<RouteMatch, 4>>
          ) -> crate::Result<()> {
            $({
              let mut local_prev = _prev.clone();
              local_prev.push(RouteMatch::new($N, $T::OM, self.$N.full_path))?;
              if $T::IS_ROUTER {
                self.$N.value.paths_indices(local_prev, _vec)?;
              } else {
                _vec.push(local_prev)?;
              }
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
      use crate::{
        database::{
          Typed, client::postgres::{DecodeWrapper, EncodeWrapper, Postgres, StructDecoder, StructEncoder},
        },
        misc::{Decode, Encode}
      };

      $(
        impl<'de, $($T,)* ERR> Decode<'de, Postgres<ERR>> for ($( $T, )*)
        where
          $($T: Decode<'de, Postgres<ERR>>,)*
          ERR: From<crate::Error>,
        {
          #[inline]
          fn decode(_: &mut (), dw: &mut DecodeWrapper<'de>) -> Result<Self, ERR> {
            let mut _sd = StructDecoder::<ERR>::new(dw);
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
          fn encode(&self, _: &mut (), _ew: &mut EncodeWrapper<'_, '_>) -> Result<(), ERR> {
            let mut _ev = StructEncoder::<ERR>::new(_ew)?;
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

mod _16_tuple_impls {
  impl_tuples! {
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
}

#[cfg(feature = "32-tuple-impls")]
mod _32_tuple_impls {
  impl_tuples! {
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17) S(18)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17) S(18) T(19)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17) S(18) T(19) U(20)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17) S(18) T(19) U(20) V(21)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17) S(18) T(19) U(20) V(21) W(22)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17) S(18) T(19) U(20) V(21) W(22) X(23)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17) S(18) T(19) U(20) V(21) W(22) X(23) Y(24)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17) S(18) T(19) U(20) V(21) W(22) X(23) Y(24) Z(25)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17) S(18) T(19) U(20) V(21) W(22) X(23) Y(24) Z(25) AA(26)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17) S(18) T(19) U(20) V(21) W(22) X(23) Y(24) Z(25) AA(26) AB(27)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17) S(18) T(19) U(20) V(21) W(22) X(23) Y(24) Z(25) AA(26) AB(27) AC(28)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17) S(18) T(19) U(20) V(21) W(22) X(23) Y(24) Z(25) AA(26) AB(27) AC(28) AD(29)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17) S(18) T(19) U(20) V(21) W(22) X(23) Y(24) Z(25) AA(26) AB(27) AC(28) AD(29) AE(30)]
    [A(0) B(1) C(2) D(3) E(4) F(5) G(6) H(7) I(8) J(9) K(10) L(11) M(12) N(13) O(14) P(15) Q(16) R(17) S(18) T(19) U(20) V(21) W(22) X(23) Y(24) Z(25) AA(26) AB(27) AC(28) AD(29) AE(30) AF(31)]
  }
}
