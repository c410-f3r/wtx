//! Postgres, is a free and open-source relational database management system (RDBMS) emphasizing
//! extensibility and SQL compliance.

#[macro_use]
mod macros;

mod authentication;
mod batch;
mod client_buffer;
mod config;
#[cfg(feature = "database-tests")]
mod database_test;
mod db_error;
#[cfg(all(feature = "_integration-tests", test))]
mod integration_tests;
mod message;
mod misc;
mod msg_field;
mod postgres_client;
mod postgres_column_info;
mod postgres_decode_wrapper;
mod postgres_encode_wrapper;
mod postgres_error;
mod postgres_record;
mod postgres_records;
mod protocol;
mod sql_state;
mod struct_decoder;
mod struct_encoder;
mod ty;
mod tys;

use crate::{
  codec::{CodecController, U64String},
  database::{
    Database, DatabaseTy,
    client::rdbms::{
      common_client_buffer::CommonClientBuffer,
      common_record::CommonRecord,
      common_records::CommonRecords,
      statement::{Statement, StatementMut},
      statements::Statements,
    },
  },
};
pub use batch::Batch;
pub use client_buffer::ClientBuffer;
pub use config::Config;
use core::{
  fmt::{Debug, Formatter},
  marker::PhantomData,
};
#[cfg(feature = "database-tests")]
pub use database_test::*;
pub use db_error::{DbError, ErrorPosition, Severity};
pub use postgres_client::PostgresClient;
pub use postgres_decode_wrapper::PostgresDecodeWrapper;
pub use postgres_encode_wrapper::PostgresEncodeWrapper;
pub use postgres_error::PostgresError;
pub use postgres_record::PostgresRecord;
pub use postgres_records::PostgresRecords;
pub use sql_state::SqlState;
pub use struct_decoder::StructDecoder;
pub use struct_encoder::StructEncoder;
pub use ty::Ty;
pub use tys::pg_range::PgRange;

pub(crate) type Oid = u32;
pub(crate) type PostgresCommonRecord<'exec, E> =
  CommonRecord<'exec, U64String, postgres_column_info::PostgresColumnInfo, Postgres<E>, Ty>;
pub(crate) type PostgresCommonRecords<'exec, E> =
  CommonRecords<'exec, U64String, postgres_column_info::PostgresColumnInfo, Postgres<E>, Ty>;
pub(crate) type PostgresStatements =
  Statements<U64String, postgres_column_info::PostgresColumnInfo, Ty>;
pub(crate) type PostgresStatement<'stmts> =
  Statement<'stmts, U64String, postgres_column_info::PostgresColumnInfo, Ty>;
pub(crate) type PostgresStatementMut<'stmts> =
  StatementMut<'stmts, U64String, postgres_column_info::PostgresColumnInfo, Ty>;
pub(crate) type PostgresCommonExecutorBuffer =
  CommonClientBuffer<U64String, postgres_column_info::PostgresColumnInfo, Ty>;

/// Postgres
pub struct Postgres<E>(PhantomData<fn() -> E>);

impl<E> Database for Postgres<E>
where
  E: From<crate::Error>,
{
  const TY: DatabaseTy = DatabaseTy::Postgres;

  type Record<'exec> = PostgresRecord<'exec, E>;
  type Records<'exec> = PostgresRecords<'exec, E>;
  type Ty = Ty;
}

impl<E> CodecController for Postgres<E>
where
  E: From<crate::Error>,
{
  type DecodeWrapper<'inner, 'outer, 'rem>
    = PostgresDecodeWrapper<'inner, 'rem>
  where
    'inner: 'outer;
  type Error = E;
  type EncodeWrapper<'inner, 'outer, 'rem>
    = PostgresEncodeWrapper<'inner, 'outer>
  where
    'inner: 'outer;
}

impl<E> Debug for Postgres<E> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Postgres").finish()
  }
}

impl<E> Default for Postgres<E> {
  #[inline]
  fn default() -> Self {
    Self(PhantomData)
  }
}

mod array {
  use crate::{
    collections::{ArrayString, LinearStorageLen},
    database::{
      FromRecords, FromRecordsParams, Record as _,
      client::postgres::{Postgres, PostgresRecord, PostgresRecords},
    },
    misc::{from_utf8_basic, into_rslt},
  };

  impl<'exec, E, L, const N: usize> FromRecords<'exec, Postgres<E>> for ArrayString<L, N>
  where
    E: From<crate::Error>,
    L: LinearStorageLen,
  {
    const FIELDS: u16 = 1;
    const ID_IDX: Option<usize> = None;
    type IdTy = ();

    #[inline]
    fn from_records(
      curr_params: &mut FromRecordsParams<PostgresRecord<'exec, E>>,
      _: &PostgresRecords<'_, E>,
    ) -> Result<Self, E> {
      let rslt = from_utf8_basic(into_rslt(curr_params.curr_record.value(0))?.bytes())
        .map_err(From::from)?
        .try_into()?;
      curr_params.inc_consumed_records(1);
      Ok(rslt)
    }
  }
}

#[cfg(feature = "crypto")]
mod crypto {
  use crate::{
    codec::{Decode, Encode},
    crypto::SignatureTy,
    database::{
      Typed,
      client::postgres::{Postgres, PostgresDecodeWrapper, PostgresEncodeWrapper, Ty},
    },
  };

  impl<'de, E> Decode<'de, Postgres<E>> for SignatureTy
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn decode(dw: &mut PostgresDecodeWrapper<'de, '_>) -> Result<Self, E> {
      let string = <&str as Decode<'de, Postgres<E>>>::decode(dw)?;
      Ok(Self::try_from(string.as_bytes())?)
    }
  }

  impl<E> Encode<Postgres<E>> for SignatureTy
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn encode(&self, ew: &mut PostgresEncodeWrapper<'_, '_>) -> Result<(), E> {
      <&str as Encode<Postgres<E>>>::encode(&(*self).into(), ew)
    }
  }

  impl<E> Typed<Postgres<E>> for SignatureTy
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn runtime_ty(&self) -> Option<Ty> {
      None
    }

    #[inline]
    fn static_ty() -> Option<Ty> {
      None
    }
  }
}
