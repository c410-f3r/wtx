// FIXME(STABLE): macro_metavar_expr

#![allow(clippy::unused_unit, reason = "macro-generated code")]

macro_rules! impl_tuples {
  ($( [$($T:ident($T13:tt))*] )+) => {
    #[cfg(feature = "database")]
    mod database {
      use crate::database::{Database, RecordValues, Typed, record_values::encode};
      use crate::codec::Encode;

      $(
        impl<DB, $($T,)*> RecordValues<DB> for ($( $T, )*)
        where
          DB: Database,
          $($T: Encode<DB> + Typed<DB>,)*
        {
          #[inline]
          fn encode_values<'bytes, 'rem, AUX>(
            &self,
            _aux: &mut AUX,
            _ew: &mut DB::EncodeWrapper<'bytes, 'bytes, 'rem>,
            mut _prefix_cb: impl FnMut(&mut AUX, &mut DB::EncodeWrapper<'bytes, 'bytes, 'rem>) -> usize,
            mut _suffix_cb: impl FnMut(&mut AUX, &mut DB::EncodeWrapper<'bytes, 'bytes, 'rem>, bool, usize) -> usize,
          ) -> Result<usize, DB::Error> {
            let mut _n: usize = 0;
            $(
              encode(
                _aux,
                &self.$T13,
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
            $( _cb(self.$T13.is_null(), self.$T13.runtime_ty())?; )*
            Ok(())
          }
        }
      )+
    }

    #[cfg(feature = "http2-server-framework")]
    mod http_server_framework {
      use crate::{
        collections::{ArrayVectorCopy, ShortStrU8, Vector},
        http::{
          OperationMode, HttpError, StatusCode, AutoStream, ManualStream, Request,
          MsgBufferString, Response,
          http2_server_framework::{Endpoint, Middleware, RouteMatch, EndpointNode, PathParams}
        },
      };
      use core::ops::ControlFlow;

      $(
        impl<$($T,)* DATA, ERR> Middleware<DATA, ERR> for ($($T,)*)
        where
          $($T: Middleware<DATA, ERR>,)*
          ERR: From<crate::Error>
        {
          type Aux = ($($T::Aux,)*);

          #[inline]
          fn aux(&self) -> Self::Aux {
            ($(self.$T13.aux(),)*)
          }

          #[inline]
          async fn req(
            &self,
            _data: &mut DATA,
            _mw_aux: &mut Self::Aux,
            _req: &mut Request<MsgBufferString>,
          ) -> Result<ControlFlow<StatusCode, ()>, ERR> {
            $({
              let rslt = self.$T13.req(_data, &mut _mw_aux.$T13, _req).await?;
              if let ControlFlow::Break(status_code) = rslt {
                return Ok(ControlFlow::Break(status_code));
              }
            })*
            Ok(ControlFlow::Continue(()))
          }

          #[inline]
          async fn res(
            &self,
            _data: &mut DATA,
            _mw_aux: &mut Self::Aux,
            _res: Response<&mut MsgBufferString>,
          ) -> Result<ControlFlow<StatusCode, ()>, ERR> {
            $({
              let local_res = Response {
                msg_data: &mut *_res.msg_data,
                status_code: _res.status_code,
              };
              let rslt = self.$T13.res(_data, &mut _mw_aux.$T13, local_res).await?;
              if let ControlFlow::Break(status_code) = rslt {
                return Ok(ControlFlow::Break(status_code));
              }
            })*
            Ok(ControlFlow::Continue(()))
          }
        }

        impl<$($T,)* DATA, ERR, STREAM> Endpoint<DATA, ERR, STREAM> for ($(PathParams<$T>,)*)
        where
          $($T: Endpoint<DATA, ERR, STREAM>,)*
          ERR: From<crate::Error>,
        {
          const OM: OperationMode = OperationMode::Auto;

          #[inline]
          async fn auto(
            &self,
            _auto_stream: &mut AutoStream<DATA>,
            _path_defs: (u8, &[RouteMatch]),
          ) -> Result<StatusCode, ERR> {
            match _path_defs.1.get(usize::from(_path_defs.0)).map(|el| el.idx) {
              $(
                Some($T13) => {
                  return self
                    .$T13
                    .value
                    .auto(_auto_stream, (_path_defs.0.wrapping_add(1), _path_defs.1))
                    .await;
                }
              )*
              _ => Err(ERR::from(HttpError::UriMismatch.into()))
            }
          }

          #[inline]
          async fn manual(
            &self,
            _manual_stream: ManualStream<DATA, STREAM>,
            _path_defs: (u8, &[RouteMatch]),
          ) -> Result<(), ERR> {
            match _path_defs.1.get(usize::from(_path_defs.0)).map(|el| el.idx) {
              $(
                Some($T13) => {
                  return self
                    .$T13
                    .value
                    .manual(_manual_stream, (_path_defs.0.wrapping_add(1), _path_defs.1))
                    .await;
                }
              )*
              _ => Err(ERR::from(HttpError::UriMismatch.into()))
            }
          }
        }

        impl<$($T,)* DATA, ERR, STREAM> EndpointNode<DATA, ERR, STREAM> for ($(PathParams<$T>,)*)
        where
          $($T: EndpointNode<DATA, ERR, STREAM>,)*
          ERR: From<crate::Error>,
        {
          const IS_ROUTER: bool = false;

          #[inline]
          fn paths_indices(
            &self,
            _prev: ArrayVectorCopy<RouteMatch, 4>,
            _vec: &mut Vector<ArrayVectorCopy<RouteMatch, 4>>
          ) -> crate::Result<()> {
            $({
              let mut local_prev = _prev.clone();
              local_prev.push(RouteMatch::new($T13, $T::OM, ShortStrU8::new(self.$T13.full_path)?))?;
              if $T::IS_ROUTER {
                self.$T13.value.paths_indices(local_prev, _vec)?;
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
        codec::{Decode, Encode},
        database::{
          Typed, client::postgres::{PostgresDecodeWrapper, PostgresEncodeWrapper, Postgres, StructDecoder, StructEncoder},
        },
      };

      $(
        impl<'de, $($T,)* ERR> Decode<'de, Postgres<ERR>> for ($( $T, )*)
        where
          $($T: Decode<'de, Postgres<ERR>>,)*
          ERR: From<crate::Error>,
        {
          #[inline]
          fn decode(dw: &mut PostgresDecodeWrapper<'de, '_>) -> Result<Self, ERR> {
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
          fn encode(&self, _ew: &mut PostgresEncodeWrapper<'_>) -> Result<(), ERR> {
            let mut _ev = StructEncoder::<ERR>::new(_ew)?;
            $(
              _ev = _ev.encode(&self.$T13)?;
            )*
            Ok(())
          }
        }
      )+
    }

    #[cfg(feature = "web-socket-server-framework")]
    mod web_socket_server_framework {
      use alloc::string::String;
      use crate::{
        collections::Vector,
        executor::Executor,
        http::{Router, WebSocketRouter},
        futures::FnFut,
        web_socket::{WebSocket, WsCompression},
      };

      type LocalWs<T2, EX, TM> = WebSocket<
        <T2 as WsCompression<false>>::NegotiatedCompression,
        <EX as Executor>::TcpStream,
        TM,
        false,
      >;

      $(
        impl<$($T,)* CO, ER, EX, TM> WebSocketRouter<CO, ER, EX, TM> for (
          $(
            (&'static str, $T),
          )*
        )
        where
          $($T: FnFut<(Vector<u8>, LocalWs<CO, EX, TM>), Result = Result<(), ER>>,)*
          CO: WsCompression<false>,
          ER: From<crate::Error>,
          EX: Executor,
        {
          #[inline]
          async fn call(
            &self,
            matcher: &Router<u8>,
            path: String,
            _ws: LocalWs<CO, EX, TM>,
          ) -> Result<(), ER> {
            let rslt = matcher.find(&path)?;
            match rslt.data() {
              $(
                $T13 => (self.$T13.1).call((path.into_bytes().into(), _ws)).await?,
              )*
              _ => {}
            }
            Ok(())
          }

          #[inline]
          fn paths(&self) -> impl ExactSizeIterator<Item = &'static str> {
            [$(self.$T13.0,)*].into_iter()
          }
        }
      )+
    }
  }
}

mod _16_tuple_impls {
  impl_tuples! {
    []
    [T0(0)]
    [T0(0) T1(1)]
    [T0(0) T1(1) T2(2)]
    [T0(0) T1(1) T2(2) T3(3)]
    [T0(0) T1(1) T2(2) T3(3) T4(4)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15)]
  }
}

#[cfg(feature = "32-tuple-impls")]
mod _32_tuple_impls {
  impl_tuples! {
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17) T18(18)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17) T18(18) T19(19)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17) T18(18) T19(19) T20(20)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17) T18(18) T19(19) T20(20) T21(21)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17) T18(18) T19(19) T20(20) T21(21) T22(22)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17) T18(18) T19(19) T20(20) T21(21) T22(22) T23(23)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17) T18(18) T19(19) T20(20) T21(21) T22(22) T23(23) T24(24)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17) T18(18) T19(19) T20(20) T21(21) T22(22) T23(23) T24(24) T25(25)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17) T18(18) T19(19) T20(20) T21(21) T22(22) T23(23) T24(24) T25(25) T26(26)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17) T18(18) T19(19) T20(20) T21(21) T22(22) T23(23) T24(24) T25(25) T26(26) T27(27)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17) T18(18) T19(19) T20(20) T21(21) T22(22) T23(23) T24(24) T25(25) T26(26) T27(27) T28(28)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17) T18(18) T19(19) T20(20) T21(21) T22(22) T23(23) T24(24) T25(25) T26(26) T27(27) T28(28) T29(29)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17) T18(18) T19(19) T20(20) T21(21) T22(22) T23(23) T24(24) T25(25) T26(26) T27(27) T28(28) T29(29) T30(30)]
    [T0(0) T1(1) T2(2) T3(3) T4(4) T5(5) T6(6) T7(7) T8(8) T9(9) T10(10) T11(11) T12(12) T13(13) T14(14) T15(15) T16(16) T17(17) T18(18) T19(19) T20(20) T21(21) T22(22) T23(23) T24(24) T25(25) T26(26) T27(27) T28(28) T29(29) T30(30) T31(31)]
  }
}
