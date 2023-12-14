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
    RecordValues,
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
  pub(crate) async fn do_prepare_send_and_await<'stmts, E, RV>(
    cmd: &str,
    nb: &mut PartitionedFilledBuffer,
    rv: RV,
    stmts: &'stmts mut Statements,
    stream: &mut S,
    tys: &[Ty],
  ) -> Result<Statement<'stmts>, E>
  where
    E: From<crate::Error>,
    RV: RecordValues<Postgres, E>,
  {
    let (stmt_id, stmt) = Self::do_prepare(cmd, nb, stmts, stream, tys).await?;
    let mut fbw = FilledBufferWriter::from(&mut *nb);
    bind(&mut fbw, "", rv, &stmt_id)?;
    execute(&mut fbw, 0, "")?;
    sync(&mut fbw)?;
    stream.write_all(fbw._curr_bytes()).await?;

    let bind_msg = Self::fetch_msg_from_stream(nb, stream).await?;
    let MessageTy::BindComplete = bind_msg.ty else {
      return Err(crate::Error::UnexpectedDatabaseMessage { received: bind_msg.tag }.into());
    };
    Ok(stmt)
  }

  async fn do_prepare<'stmts>(
    cmd: &str,
    nb: &mut PartitionedFilledBuffer,
    stmts: &'stmts mut Statements,
    stream: &mut S,
    tys: &[Ty],
  ) -> crate::Result<(ArrayString<22>, Statement<'stmts>)> {
    let (stmt_id, mut builder) = match stmts.push(cmd) {
      PushRslt::Builder(builder) => (Self::stmt_id(builder.cmd_hash())?, builder),
      PushRslt::Stmt(cmd_hash, stmt) => return Ok((Self::stmt_id(cmd_hash)?, stmt)),
    };

    let mut fbw = FilledBufferWriter::from(&mut *nb);
    parse(cmd, &mut fbw, tys.iter().copied().map(Into::into), &stmt_id)?;
    describe(&stmt_id, &mut fbw, b'S')?;
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

    let cmd_hash = builder.cmd_hash();
    if let Some(stmt) = builder.finish().get_by_cmd_hash(cmd_hash) {
      Ok((stmt_id, stmt))
    } else {
      unreachable!()
    }
  }

  fn stmt_id(cmd_hash: u64) -> crate::Result<ArrayString<22>> {
    Ok(ArrayString::try_from(format_args!("s{cmd_hash}"))?)
  }
}
