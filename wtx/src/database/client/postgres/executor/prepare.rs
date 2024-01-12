#![allow(
  // Borrow checker limitation
  clippy::unreachable
)]

use crate::{
  database::{
    client::postgres::{
      bind, describe, execute,
      executor::commons::FetchWithStmtCommons,
      executor_buffer::ExecutorBuffer,
      parse,
      statements::{Column, PushRslt, Statement},
      sync,
      ty::{CustomTy, Ty, TyKind},
      Executor, MessageTy, MsgField, Postgres, Statements,
    },
    Identifier, Record as _, RecordValues, Stmt,
  },
  misc::{FilledBufferWriter, PartitionedFilledBuffer, Stream},
};
use alloc::{sync::Arc, vec::Vec};
use arrayvec::ArrayString;
use core::{borrow::BorrowMut, ops::Range};
use hashbrown::HashMap;

impl<E, EB, S> Executor<E, EB, S>
where
  E: From<crate::Error>,
  EB: BorrowMut<ExecutorBuffer>,
  S: Stream,
{
  pub(crate) async fn write_send_await_stmt_initial<RV>(
    fwsc: &mut FetchWithStmtCommons<'_, S>,
    nb: &mut PartitionedFilledBuffer,
    rv: RV,
    stmt: &Statement<'_>,
    stmt_id_str: &str,
  ) -> Result<(), E>
  where
    RV: RecordValues<Postgres<E>>,
  {
    let mut fbw = FilledBufferWriter::from(&mut *nb);
    bind(&mut fbw, "", rv, stmt, stmt_id_str)?;
    execute(&mut fbw, 0, "")?;
    sync(&mut fbw)?;
    fwsc.stream.write_all(fbw._curr_bytes()).await?;
    let msg = Self::fetch_msg_from_stream(fwsc.is_closed, nb, fwsc.stream).await?;
    let MessageTy::BindComplete = msg.ty else {
      return Err(crate::Error::UnexpectedDatabaseMessage { received: msg.tag }.into());
    };
    Ok(())
  }

  pub(crate) async fn write_send_await_stmt_prot<'stmts, STMT>(
    fwsc: &mut FetchWithStmtCommons<'_, S>,
    nb: &mut PartitionedFilledBuffer,
    stmt: STMT,
    stmts: &'stmts mut Statements,
    vb: &mut Vec<(bool, Range<usize>)>,
  ) -> Result<(u64, ArrayString<22>, Statement<'stmts>), E>
  where
    STMT: Stmt,
  {
    let stmt_hash = stmt.hash(stmts.hasher_mut());
    let (stmt_id_str, mut builder) = match stmts.push(stmt_hash) {
      PushRslt::Builder(builder) => (Self::stmt_id_str(stmt_hash)?, builder),
      PushRslt::Stmt(stmt) => return Ok((stmt_hash, Self::stmt_id_str(stmt_hash)?, stmt)),
    };

    let stmt_cmd = stmt.cmd().ok_or(crate::Error::UnknownStatementId)?;

    let mut fbw = FilledBufferWriter::from(&mut *nb);
    parse(stmt_cmd, &mut fbw, fwsc.tys.iter().map(Into::into), &stmt_id_str)?;
    describe(&stmt_id_str, &mut fbw, b'S')?;
    sync(&mut fbw)?;
    fwsc.stream.write_all(fbw._curr_bytes()).await?;

    let msg0 = Self::fetch_msg_from_stream(fwsc.is_closed, nb, fwsc.stream).await?;
    let MessageTy::ParseComplete = msg0.ty else {
      return Err(crate::Error::UnexpectedDatabaseMessage { received: msg0.tag }.into());
    };

    let msg1 = Self::fetch_msg_from_stream(fwsc.is_closed, nb, fwsc.stream).await?;
    let MessageTy::ParameterDescription(mut pd) = msg1.ty else {
      return Err(crate::Error::UnexpectedDatabaseMessage { received: msg1.tag }.into());
    };
    while let [a, b, c, d, sub_data @ ..] = pd {
      let id = u32::from_be_bytes([*a, *b, *c, *d]);
      let ty = if let Some(elem) = Self::known_type(id, fwsc.tb) {
        elem
      } else {
        fwsc.ftb.push((builder.params_len(), id));
        Ty::Custom(Arc::new(CustomTy { kind: TyKind::Simple, name: ArrayString::new(), oid: id }))
      };
      builder.push_param(ty);
      pd = sub_data;
    }

    let msg2 = Self::fetch_msg_from_stream(fwsc.is_closed, nb, fwsc.stream).await?;
    let params_cut_point = fwsc.ftb.len();
    match msg2.ty {
      MessageTy::NoData => {}
      MessageTy::RowDescription(mut rd) => {
        while !rd.is_empty() {
          let (read, msg_field) = MsgField::parse(rd)?;
          let ty = if let Some(elem) = Self::known_type(msg_field.type_oid, fwsc.tb) {
            elem
          } else {
            fwsc.ftb.push((builder.columns_len(), msg_field.type_oid));
            Ty::Custom(Arc::new(CustomTy {
              kind: TyKind::Simple,
              name: ArrayString::new(),
              oid: msg_field.type_oid,
            }))
          };
          builder.push_column(Column { name: msg_field.name.try_into().map_err(Into::into)?, ty });
          rd = rd.get(read..).unwrap_or_default();
        }
      }
      _ => return Err(crate::Error::UnexpectedDatabaseMessage { received: msg2.tag }.into()),
    }

    let msg3 = Self::fetch_msg_from_stream(fwsc.is_closed, nb, fwsc.stream).await?;
    let MessageTy::ReadyForQuery = msg3.ty else {
      return Err(crate::Error::UnexpectedDatabaseMessage { received: msg3.tag }.into());
    };

    let bc_stmts = builder.finish();
    let mut params_counter = 0;
    loop {
      if params_counter >= params_cut_point {
        break;
      }
      let Some((idx, oid)) = fwsc.ftb.pop() else {
        break;
      };
      params_counter = params_counter.wrapping_add(1);
      let ty = Box::pin(Self::fetch_type(fwsc, nb, oid, bc_stmts, vb)).await?;
      *bc_stmts.param_mut(idx).unwrap() = ty;
    }
    while let Some((idx, oid)) = fwsc.ftb.pop() {
      let ty = Box::pin(Self::fetch_type(fwsc, nb, oid, bc_stmts, vb)).await?;
      bc_stmts.column_mut(idx).unwrap().ty = ty;
    }
    if let Some(stmt) = bc_stmts.get_by_stmt_hash(stmt_hash) {
      Ok((stmt_hash, stmt_id_str, stmt))
    } else {
      unreachable!()
    }
  }

  async fn fetch_range_cached(
    fwsc: &mut FetchWithStmtCommons<'_, S>,
    name: Identifier,
    nb: &mut PartitionedFilledBuffer,
    oid: u32,
    stmts: &mut Statements,
    vb: &mut Vec<(bool, Range<usize>)>,
  ) -> Result<Ty, E> {
    let elem_oid: u32 = Self::write_send_await_fetch_with_stmt(
      fwsc,
      nb,
      (oid,),
      "SELECT rngsubtype FROM pg_catalog.pg_range WHERE rngtypid = $1",
      stmts,
      vb,
    )
    .await?
    .decode(0)?;
    Ok(Ty::Custom(Arc::new(CustomTy {
      kind: TyKind::Range(Self::fetch_type_cached(fwsc, nb, elem_oid, stmts, vb).await?),
      name,
      oid,
    })))
  }

  async fn fetch_type(
    fwsc: &mut FetchWithStmtCommons<'_, S>,
    nb: &mut PartitionedFilledBuffer,
    oid: u32,
    stmts: &mut Statements,
    vb: &mut Vec<(bool, Range<usize>)>,
  ) -> Result<Ty, E> {
    let record = Self::write_send_await_fetch_with_stmt(
      fwsc,
      nb,
      (oid,),
      "SELECT pt.typbasetype, pt.typcategory, pt.typelem, pt.typname, pt.typtype FROM pg_catalog.pg_type pt WHERE pt.oid = $1",
      stmts,
      vb,
    )
    .await?;

    let basetype: u32 = record.decode(0)?;
    let category: u8 = record.decode(1)?;
    let elem: u32 = record.decode(2)?;
    let name: Identifier = record.decode(3)?;
    let ty_u8: u8 = record.decode(4)?;

    let ty = match (ty_u8, category) {
      (b'b', b'A') => Ty::Custom(Arc::new(CustomTy {
        kind: TyKind::Array(Self::fetch_type_cached(fwsc, nb, elem, stmts, vb).await?),
        name,
        oid,
      })),
      (b'c', b'C') => Ty::Custom(Arc::new(CustomTy { kind: TyKind::Composite, name, oid })),
      (b'd', _) => Ty::Custom(Arc::new(CustomTy {
        kind: TyKind::Domain(Self::fetch_type_cached(fwsc, nb, basetype, stmts, vb).await?),
        name,
        oid,
      })),
      (b'e', b'E') => Ty::Custom(Arc::new(CustomTy { kind: TyKind::Enum, name, oid })),
      (b'p', b'P') => Ty::Custom(Arc::new(CustomTy { kind: TyKind::Pseudo, name, oid })),
      (b'r', b'R') => Self::fetch_range_cached(fwsc, name, nb, oid, stmts, vb).await?,
      _ => Ty::Custom(Arc::new(CustomTy { kind: TyKind::Simple, name, oid })),
    };

    let _ = fwsc.tb.insert(oid, ty.clone());
    Ok(ty)
  }

  async fn fetch_type_cached(
    fwsc: &mut FetchWithStmtCommons<'_, S>,
    nb: &mut PartitionedFilledBuffer,
    oid: u32,
    stmts: &mut Statements,
    vb: &mut Vec<(bool, Range<usize>)>,
  ) -> Result<Ty, E> {
    if let Some(rslt) = Self::known_type(oid, fwsc.tb) {
      return Ok(rslt);
    }
    Box::pin(Self::fetch_type(fwsc, nb, oid, stmts, vb)).await
  }

  fn known_type(oid: u32, tb: &HashMap<u32, Ty>) -> Option<Ty> {
    if let Ok(rslt) = Ty::try_from(oid) {
      return Some(rslt);
    }
    tb.get(&oid).cloned()
  }

  fn stmt_id_str(stmt_hash: u64) -> crate::Result<ArrayString<22>> {
    Ok(ArrayString::try_from(format_args!("s{stmt_hash}"))?)
  }
}
