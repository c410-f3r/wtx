mod authentication;
mod fetch;
mod prepare;
mod simple_query;

use crate::{
  database::{
    client::postgres::{
      encrypted_conn,
      executor_buffer::{ExecutorBuffer, ExecutorBufferPartsMut},
      initial_conn_msg, Config, MessageTy, Postgres, Record, Records, TransactionManager,
    },
    Database, RecordValues, StmtId, TransactionManager as _,
  },
  misc::{FilledBufferWriter, Stream, TlsStream},
  rng::Rng,
};
use core::{borrow::BorrowMut, future::Future};

/// Executor
#[derive(Debug)]
pub struct Executor<EB, S> {
  pub(crate) eb: EB,
  pub(crate) process_id: i32,
  pub(crate) secret_key: i32,
  pub(crate) stream: S,
}

impl<EB, S> Executor<EB, S>
where
  EB: BorrowMut<ExecutorBuffer>,
  S: Stream,
{
  /// Connects with an unencrypted stream.
  #[inline]
  pub async fn connect<RNG>(
    config: &Config<'_>,
    mut eb: EB,
    rng: &mut RNG,
    stream: S,
  ) -> crate::Result<Self>
  where
    RNG: Rng,
  {
    eb.borrow_mut().clear_all();
    Self::do_connect(config, eb, rng, stream, None).await
  }

  /// Initially connects with an unencrypted stream that should be later upgraded to an encrypted
  /// stream.
  #[inline]
  pub async fn connect_encrypted<F, IS, RNG>(
    config: &Config<'_>,
    mut eb: EB,
    mut initial_stream: IS,
    rng: &mut RNG,
    cb: impl FnOnce(IS) -> F,
  ) -> crate::Result<Self>
  where
    F: Future<Output = crate::Result<S>>,
    IS: Stream,
    RNG: Rng,
    S: TlsStream,
  {
    eb.borrow_mut().clear_all();
    let mut fbw = FilledBufferWriter::from(&mut eb.borrow_mut().nb);
    encrypted_conn(&mut fbw)?;
    initial_stream.write_all(fbw._curr_bytes()).await?;
    let mut buf = [0];
    let _ = initial_stream.read(&mut buf).await?;
    if buf[0] != b'S' {
      return Err(crate::Error::ServerDoesNotSupportEncryption);
    }
    let stream = cb(initial_stream).await?;
    let tls_server_end_point = stream.tls_server_end_point()?;
    Self::do_connect(config, eb, rng, stream, tls_server_end_point.as_ref().map(AsRef::as_ref))
      .await
  }

  #[inline]
  async fn do_connect<RNG>(
    config: &Config<'_>,
    eb: EB,
    rng: &mut RNG,
    stream: S,
    tls_server_end_point: Option<&[u8]>,
  ) -> crate::Result<Self>
  where
    RNG: Rng,
  {
    let mut this = Self { eb, process_id: 0, secret_key: 0, stream };
    this.send_initial_conn_msg(config).await?;
    this.manage_authentication(config, rng, tls_server_end_point).await?;
    this.read_after_authentication_data().await?;
    Ok(this)
  }

  async fn send_initial_conn_msg(&mut self, config: &Config<'_>) -> crate::Result<()> {
    let mut fbw = FilledBufferWriter::from(&mut self.eb.borrow_mut().nb);
    initial_conn_msg(config, &mut fbw)?;
    self.stream.write_all(fbw._curr_bytes()).await?;
    Ok(())
  }
}

impl<EB, S> crate::database::Executor for Executor<EB, S>
where
  EB: BorrowMut<ExecutorBuffer>,
  S: Stream,
{
  type Database = Postgres;
  type TransactionManager<'tm> = TransactionManager<'tm, EB, S>
  where
    Self: 'tm;

  #[inline]
  async fn execute(&mut self, cmd: &str, cb: impl FnMut(u64)) -> crate::Result<()> {
    self.eb.borrow_mut().clear();
    self.simple_query_execute(cmd, cb).await
  }

  #[inline]
  async fn execute_with_stmt<E, SI, RV>(&mut self, stmt_id: SI, rv: RV) -> Result<u64, E>
  where
    E: From<crate::Error>,
    RV: RecordValues<Self::Database, E>,
    SI: StmtId,
  {
    self.eb.borrow_mut().clear();
    let ExecutorBufferPartsMut { nb, stmts, .. } = self.eb.borrow_mut().parts_mut();
    let _ = Self::do_prepare_send_and_await(nb, rv, stmt_id, stmts, &mut self.stream, &[]).await?;
    let mut rows = 0;
    loop {
      let msg = Self::fetch_msg_from_stream(nb, &mut self.stream).await?;
      match msg.ty {
        MessageTy::CommandComplete(local_rows) => {
          rows = local_rows;
        }
        MessageTy::ReadyForQuery => break,
        MessageTy::DataRow(_, _) | MessageTy::EmptyQueryResponse => {}
        _ => return Err(crate::Error::UnexpectedDatabaseMessage { received: msg.tag }.into()),
      }
    }
    Ok(rows)
  }

  #[inline]
  async fn fetch_with_stmt<E, SI, RV>(
    &mut self,
    stmt_id: SI,
    rv: RV,
  ) -> Result<<Self::Database as Database>::Record<'_>, E>
  where
    E: From<crate::Error>,
    RV: RecordValues<Self::Database, E>,
    SI: StmtId,
  {
    self.eb.borrow_mut().clear();
    let ExecutorBufferPartsMut { nb, stmts, vb, .. } = self.eb.borrow_mut().parts_mut();
    let stmt =
      Self::do_prepare_send_and_await(nb, rv, stmt_id, stmts, &mut self.stream, &[]).await?;
    let mut data_row_msg_range = None;
    loop {
      let msg = Self::fetch_msg_from_stream(nb, &mut self.stream).await?;
      match msg.ty {
        MessageTy::DataRow(len, _) => {
          data_row_msg_range = Some((len, nb._current_range()));
        }
        MessageTy::ReadyForQuery => break,
        MessageTy::CommandComplete(_) | MessageTy::EmptyQueryResponse => {}
        _ => return Err(crate::Error::UnexpectedDatabaseMessage { received: msg.tag }.into()),
      }
    }
    if let Some((record_bytes, len)) = data_row_msg_range.and_then(|(len, range)| {
      let record_range = range.start.wrapping_add(7)..range.end;
      Some((nb._buffer().get(record_range)?, len))
    }) {
      Record::parse(record_bytes, 0..record_bytes.len(), stmt, vb, len).map_err(From::from)
    } else {
      Err(crate::Error::NoRecord.into())
    }
  }

  #[inline]
  async fn fetch_many_with_stmt<E, SI, RV>(
    &mut self,
    stmt_id: SI,
    rv: RV,
    mut cb: impl FnMut(<Self::Database as Database>::Record<'_>) -> Result<(), E>,
  ) -> Result<<Self::Database as Database>::Records<'_>, E>
  where
    E: From<crate::Error>,
    RV: RecordValues<Self::Database, E>,
    SI: StmtId,
  {
    self.eb.borrow_mut().clear();
    let ExecutorBufferPartsMut { nb, rb, stmts, vb, .. } = self.eb.borrow_mut().parts_mut();
    let stmt =
      Self::do_prepare_send_and_await(nb, rv, stmt_id, stmts, &mut self.stream, &[]).await?;
    let begin = nb._current_end_idx();
    let begin_data = nb._current_end_idx().wrapping_add(7);
    loop {
      let msg = Self::fetch_msg_from_stream(nb, &mut self.stream).await?;
      match msg.ty {
        MessageTy::DataRow(len, _) => {
          let bytes = nb._buffer().get(begin_data..nb._current_end_idx()).unwrap_or_default();
          let range_begin = nb._antecedent_end_idx().wrapping_sub(begin);
          let range_end = nb._current_end_idx().wrapping_sub(begin_data);
          cb(Record::parse(bytes, range_begin..range_end, stmt.clone(), vb, len)?)?;
          rb.push(vb.len());
        }
        MessageTy::ReadyForQuery => {
          break;
        }
        MessageTy::CommandComplete(_) | MessageTy::EmptyQueryResponse => {}
        _ => {
          return Err(crate::Error::UnexpectedDatabaseMessage { received: msg.tag }.into());
        }
      }
    }
    Ok(Records {
      bytes: nb
        ._buffer()
        .get(begin_data.wrapping_add(4)..nb._current_end_idx())
        .unwrap_or_default(),
      records_values_offsets: rb,
      stmt,
      values_bytes_offsets: vb,
    })
  }

  #[inline]
  async fn prepare(&mut self, cmd: &str) -> crate::Result<u64> {
    self.eb.borrow_mut().clear();
    let ExecutorBufferPartsMut { nb, stmts, .. } = self.eb.borrow_mut().parts_mut();
    let (id, _, _) = Self::do_prepare(nb, cmd, stmts, &mut self.stream, &[]).await?;
    Ok(id)
  }

  #[inline]
  async fn transaction(&mut self) -> crate::Result<Self::TransactionManager<'_>> {
    self.eb.borrow_mut().clear();
    let mut tm = TransactionManager::new(self);
    tm.begin().await?;
    Ok(tm)
  }
}
