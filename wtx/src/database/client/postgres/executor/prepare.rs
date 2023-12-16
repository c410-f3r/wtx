#![allow(
  // Borrow checker limitation
  clippy::unreachable
)]

use crate::{
  database::{
    client::postgres::{
      bind, describe, execute,
      executor_buffer::ExecutorBuffer,
      parse,
      statements::{Column, PushRslt, Statement},
      sync,
      ty::Ty,
      Executor, MessageTy, MsgField, Postgres, Statements,
    },
    RecordValues, StmtId,
  },
  misc::{FilledBufferWriter, PartitionedFilledBuffer, Stream},
};
use arrayvec::ArrayString;
use core::borrow::BorrowMut;

impl<EB, S> Executor<EB, S>
where
  EB: BorrowMut<ExecutorBuffer>,
  S: Stream,
{
  pub(crate) async fn do_prepare_send_and_await<'stmts, E, SI, RV>(
    nb: &mut PartitionedFilledBuffer,
    rv: RV,
    stmt_id: SI,
    stmts: &'stmts mut Statements,
    stream: &mut S,
    tys: &[Ty],
  ) -> Result<Statement<'stmts>, E>
  where
    E: From<crate::Error>,
    RV: RecordValues<Postgres, E>,
    SI: StmtId,
  {
    let (_, id_str, stmt) = Self::do_prepare(nb, stmt_id, stmts, stream, tys).await?;
    let mut fbw = FilledBufferWriter::from(&mut *nb);
    bind(&mut fbw, "", rv, &id_str)?;
    execute(&mut fbw, 0, "")?;
    sync(&mut fbw)?;
    stream.write_all(fbw._curr_bytes()).await?;

    let bind_msg = Self::fetch_msg_from_stream(nb, stream).await?;
    let MessageTy::BindComplete = bind_msg.ty else {
      return Err(crate::Error::UnexpectedDatabaseMessage { received: bind_msg.tag }.into());
    };
    Ok(stmt)
  }

  pub(crate) async fn do_prepare<'stmts, SI>(
    nb: &mut PartitionedFilledBuffer,
    stmt_id: SI,
    stmts: &'stmts mut Statements,
    stream: &mut S,
    tys: &[Ty],
  ) -> crate::Result<(u64, ArrayString<22>, Statement<'stmts>)>
  where
    SI: StmtId,
  {
    let stmt_hash = stmt_id.hash(stmts.hasher_mut());
    let (id_str, mut builder) = match stmts.push(stmt_hash) {
      PushRslt::Builder(builder) => (Self::stmt_str(stmt_hash)?, builder),
      PushRslt::Stmt(stmt) => return Ok((stmt_hash, Self::stmt_str(stmt_hash)?, stmt)),
    };

    let stmt_str = stmt_id.cmd().ok_or(crate::Error::UnknownStatementId)?;

    let mut fbw = FilledBufferWriter::from(&mut *nb);
    parse(stmt_str, &mut fbw, tys.iter().copied().map(Into::into), &id_str)?;
    describe(&id_str, &mut fbw, b'S')?;
    sync(&mut fbw)?;
    stream.write_all(fbw._curr_bytes()).await?;

    let msg0 = Self::fetch_msg_from_stream(nb, stream).await?;
    let MessageTy::ParseComplete = msg0.ty else {
      return Err(crate::Error::UnexpectedDatabaseMessage { received: msg0.tag });
    };

    let msg1 = Self::fetch_msg_from_stream(nb, stream).await?;
    let MessageTy::ParameterDescription(mut pd) = msg1.ty else {
      return Err(crate::Error::UnexpectedDatabaseMessage { received: msg1.tag });
    };
    while let [a, b, c, d, sub_data @ ..] = pd {
      builder.push_param(Ty::try_from(u32::from_be_bytes([*a, *b, *c, *d]))?);
      pd = sub_data;
    }

    let msg2 = Self::fetch_msg_from_stream(nb, stream).await?;
    match msg2.ty {
      MessageTy::RowDescription(mut rd) => {
        while !rd.is_empty() {
          let (read, msg_field) = MsgField::parse(rd)?;
          builder.push_column(Column {
            name: msg_field.name.try_into()?,
            value: msg_field.type_oid.try_into()?,
          });
          rd = rd.get(read..).unwrap_or_default();
        }
      }
      MessageTy::NoData => {}
      _ => return Err(crate::Error::UnexpectedDatabaseMessage { received: msg2.tag }),
    }

    let msg3 = Self::fetch_msg_from_stream(nb, stream).await?;
    let MessageTy::ReadyForQuery = msg3.ty else {
      return Err(crate::Error::UnexpectedDatabaseMessage { received: msg3.tag });
    };

    if let Some(stmt) = builder.finish().get_by_stmt_hash(stmt_hash) {
      Ok((stmt_hash, id_str, stmt))
    } else {
      unreachable!()
    }
  }

  fn stmt_str(stmt_hash: u64) -> crate::Result<ArrayString<22>> {
    Ok(ArrayString::try_from(format_args!("s{stmt_hash}"))?)
  }
}
