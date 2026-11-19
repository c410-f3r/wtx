use crate::http::{SessionState, session::SessionKey};

/// Abstraction for different session storages.
pub trait SessionStore<CS, E> {
  /// Stores a new [`SessionState`].
  fn create(&mut self, state: &SessionState<CS>) -> impl Future<Output = Result<(), E>>;

  /// Removes the [`SessionState`] that is identified by `session_key`.
  fn delete(&mut self, session_key: &SessionKey) -> impl Future<Output = Result<(), E>>;

  /// Removes all expired sessions.
  fn delete_expired(&mut self) -> impl Future<Output = Result<(), E>>;

  /// Loads the [`SessionState`] that is identified by `session_key`.
  fn read(
    &mut self,
    session_key: SessionKey,
  ) -> impl Future<Output = Result<Option<SessionState<CS>>, E>>;

  /// Overwrites the [`SessionState`] that is identified by `session_key` with the contents of
  /// `state`.
  fn update(
    &mut self,
    session_key: &SessionKey,
    state: &SessionState<CS>,
  ) -> impl Future<Output = Result<(), E>>;
}

impl<CS, E> SessionStore<CS, E> for () {
  #[inline]
  async fn create(&mut self, _: &SessionState<CS>) -> Result<(), E> {
    Ok(())
  }

  #[inline]
  async fn delete(&mut self, _: &SessionKey) -> Result<(), E> {
    Ok(())
  }

  #[inline]
  async fn delete_expired(&mut self) -> Result<(), E> {
    Ok(())
  }

  #[inline]
  async fn read(&mut self, _: SessionKey) -> Result<Option<SessionState<CS>>, E> {
    Ok(None)
  }

  #[inline]
  async fn update(&mut self, _: &SessionKey, _: &SessionState<CS>) -> Result<(), E> {
    Ok(())
  }
}

impl<CS, E, T> SessionStore<CS, E> for &mut T
where
  T: SessionStore<CS, E>,
{
  #[inline]
  async fn create(&mut self, state: &SessionState<CS>) -> Result<(), E> {
    (*self).create(state).await
  }

  #[inline]
  async fn delete(&mut self, session_key: &SessionKey) -> Result<(), E> {
    (*self).delete(session_key).await
  }

  #[inline]
  async fn delete_expired(&mut self) -> Result<(), E> {
    (*self).delete_expired().await
  }

  #[inline]
  async fn read(&mut self, session_key: SessionKey) -> Result<Option<SessionState<CS>>, E> {
    (*self).read(session_key).await
  }

  #[inline]
  async fn update(&mut self, session_key: &SessionKey, state: &SessionState<CS>) -> Result<(), E> {
    (*self).update(session_key, state).await
  }
}

#[cfg(feature = "postgres")]
mod postgres {
  use crate::{
    codec::Encode,
    collections::ArrayStrU8,
    database::{
      Database, DbClient as _, Record as _, RecordValues, Typed,
      client::postgres::{Postgres, PostgresClient},
      record_values::encode,
    },
    http::{
      SessionCustomState, SessionState,
      session::{SessionKey, SessionStore},
    },
    net::Stream,
    pool::{ResourceManager, SimplePool},
    tls::TlsCtx,
  };
  use core::marker::PhantomData;

  /// Expects the following SQL table definition in your database. Column ordering CAN NOT change.
  ///
  /// ```sql
  /// CREATE TABLE _wtx.session_state (
  ///   key VARCHAR(32) NOT NULL PRIMARY KEY,
  ///   csrf VARCHAR(32) NOT NULL,
  ///   expires_at BIGINT NOT NULL,
  ///   expires_at_ub BIGINT NOT NULL
  /// );
  /// ```
  ///
  /// Every column returned by [`SessionCustomState::IDENTIFIERS`] must also be declared, in
  /// the same order.
  impl<CS, E, S, TCX> SessionStore<CS, E> for PostgresClient<E, S, TCX>
  where
    CS: SessionCustomState<Postgres<E>>,
    E: From<crate::Error>,
    S: Stream,
    TCX: TlsCtx,
  {
    #[inline]
    async fn create(&mut self, state: &SessionState<CS>) -> Result<(), E> {
      let SessionState { custom_state, expires_at, expires_at_ub, session_csrf, session_key } =
        state;
      let query = insert_query(CS::NAMESPACE, CS::IDENTIFIERS)?;
      self
        .execute_stmt_none(
          query.as_str(),
          Values {
            csrf: session_csrf.as_str(),
            custom_state: custom_state.values(),
            expires_at,
            expires_at_ub,
            key: session_key.as_str(),
            phantom: PhantomData,
          },
        )
        .await?;
      Ok(())
    }

    #[inline]
    async fn delete(&mut self, session_key: &SessionKey) -> Result<(), E> {
      let query = query(["DELETE FROM ", CS::NAMESPACE, ".session_state WHERE key=$1"])?;
      self.execute_stmt_none(query.as_str(), (session_key.as_str(),)).await?;
      Ok(())
    }

    #[inline]
    async fn delete_expired(&mut self) -> Result<(), E> {
      let query = query([
        "DELETE FROM ",
        CS::NAMESPACE,
        ".session_state WHERE expires_at <= EXTRACT(EPOCH FROM NOW())::BIGINT OR expires_at_ub <= EXTRACT(EPOCH FROM NOW())::BIGINT",
      ])?;
      self.execute_stmt_none(query.as_str(), ()).await?;
      Ok(())
    }

    #[inline]
    async fn read(&mut self, session_key: SessionKey) -> Result<Option<SessionState<CS>>, E> {
      let query = read_query(CS::NAMESPACE, CS::IDENTIFIERS)?;
      let Some(record) =
        self.execute_stmt_optional(query.as_str(), (&session_key.as_str(),)).await?
      else {
        return Ok(None);
      };
      Ok(Some(SessionState {
        custom_state: CS::new(&record)?,
        expires_at: record.decode(1)?,
        expires_at_ub: record.decode(2)?,
        session_csrf: record.decode::<_, &[u8]>(0)?.try_into()?,
        session_key,
      }))
    }

    #[inline]
    async fn update(
      &mut self,
      session_key: &SessionKey,
      state: &SessionState<CS>,
    ) -> Result<(), E> {
      let SessionState {
        custom_state,
        expires_at,
        expires_at_ub,
        session_csrf,
        session_key: state_session_key,
      } = state;
      let query = update_query(CS::NAMESPACE, CS::IDENTIFIERS)?;
      self
        .execute_stmt_none(
          query.as_str(),
          UpdateValues {
            values: Values {
              key: state_session_key.as_str(),
              csrf: session_csrf.as_str(),
              expires_at,
              expires_at_ub,
              custom_state: custom_state.values(),
              phantom: PhantomData,
            },
            prev_key: session_key.as_str(),
          },
        )
        .await?;
      Ok(())
    }
  }

  impl<CS, E, RM> SessionStore<CS, E> for SimplePool<RM>
  where
    E: From<crate::Error>,
    RM: ResourceManager<CreateAux = (), Error = E, RecycleAux = ()>,
    RM::Resource: SessionStore<CS, E>,
  {
    #[inline]
    async fn create(&mut self, state: &SessionState<CS>) -> Result<(), E> {
      self.get_with_unit().await?.create(state).await
    }

    #[inline]
    async fn delete(&mut self, session_key: &SessionKey) -> Result<(), E> {
      self.get_with_unit().await?.delete(session_key).await
    }

    #[inline]
    async fn delete_expired(&mut self) -> Result<(), E> {
      self.get_with_unit().await?.delete_expired().await
    }

    #[inline]
    async fn read(&mut self, session_key: SessionKey) -> Result<Option<SessionState<CS>>, E> {
      self.get_with_unit().await?.read(session_key).await
    }

    #[inline]
    async fn update(
      &mut self,
      session_key: &SessionKey,
      state: &SessionState<CS>,
    ) -> Result<(), E> {
      self.get_with_unit().await?.update(session_key, state).await
    }
  }

  struct Values<'any, CS, D>
  where
    D: Database,
  {
    csrf: &'any str,
    custom_state: CS,
    expires_at: &'any u64,
    expires_at_ub: &'any u64,
    key: &'any str,
    phantom: PhantomData<fn() -> D>,
  }

  impl<'any, CS, D> RecordValues<D> for Values<'any, CS, D>
  where
    D: Database,
    CS: RecordValues<D>,
    &'any u64: Encode<D> + Typed<D>,
    &'any str: Encode<D> + Typed<D>,
  {
    #[inline]
    fn encode_values<'bytes, 'rem, A>(
      &self,
      aux: &mut A,
      ew: &mut D::EncodeWrapper<'bytes, 'bytes, 'rem>,
      mut prefix_cb: impl FnMut(&mut A, &mut D::EncodeWrapper<'bytes, 'bytes, 'rem>) -> usize,
      mut suffix_cb: impl FnMut(
        &mut A,
        &mut D::EncodeWrapper<'bytes, 'bytes, 'rem>,
        bool,
        usize,
      ) -> usize,
    ) -> Result<usize, D::Error> {
      let mut idx = 0;
      encode(aux, &self.key, ew, &mut idx, &mut prefix_cb, &mut suffix_cb)?;
      encode(aux, &self.csrf, ew, &mut idx, &mut prefix_cb, &mut suffix_cb)?;
      encode(aux, &self.expires_at, ew, &mut idx, &mut prefix_cb, &mut suffix_cb)?;
      encode(aux, &self.expires_at_ub, ew, &mut idx, &mut prefix_cb, &mut suffix_cb)?;
      Ok(idx.wrapping_add(self.custom_state.encode_values(aux, ew, prefix_cb, suffix_cb)?))
    }

    #[inline]
    fn len(&self) -> usize {
      4usize.wrapping_add(self.custom_state.len())
    }

    #[inline]
    fn walk(
      &self,
      mut cb: impl FnMut(bool, Option<D::Ty>) -> Result<(), D::Error>,
    ) -> Result<(), D::Error> {
      (self.key, self.csrf, self.expires_at, self.expires_at_ub).walk(&mut cb)?;
      self.custom_state.walk(cb)
    }
  }

  struct UpdateValues<'any, CS, D>
  where
    D: Database,
  {
    values: Values<'any, CS, D>,
    prev_key: &'any str,
  }

  impl<'any, CS, D> RecordValues<D> for UpdateValues<'any, CS, D>
  where
    D: Database,
    CS: RecordValues<D>,
    &'any u64: Encode<D> + Typed<D>,
    &'any str: Encode<D> + Typed<D>,
  {
    #[inline]
    fn encode_values<'bytes, 'rem, A>(
      &self,
      aux: &mut A,
      ew: &mut D::EncodeWrapper<'bytes, 'bytes, 'rem>,
      mut prefix_cb: impl FnMut(&mut A, &mut D::EncodeWrapper<'bytes, 'bytes, 'rem>) -> usize,
      mut suffix_cb: impl FnMut(
        &mut A,
        &mut D::EncodeWrapper<'bytes, 'bytes, 'rem>,
        bool,
        usize,
      ) -> usize,
    ) -> Result<usize, D::Error> {
      let mut idx = self.values.encode_values(aux, ew, &mut prefix_cb, &mut suffix_cb)?;
      encode(aux, &self.prev_key, ew, &mut idx, &mut prefix_cb, &mut suffix_cb)?;
      Ok(idx)
    }

    #[inline]
    fn len(&self) -> usize {
      self.values.len().wrapping_add(1)
    }

    #[inline]
    fn walk(
      &self,
      mut cb: impl FnMut(bool, Option<D::Ty>) -> Result<(), D::Error>,
    ) -> Result<(), D::Error> {
      self.values.walk(&mut cb)?;
      (self.prev_key,).walk(cb)
    }
  }

  #[inline]
  fn query<const N: usize>(parts: [&str; N]) -> crate::Result<ArrayStrU8<255>> {
    let mut query = ArrayStrU8::new();
    let _ = query.push_strs(parts)?;
    Ok(query)
  }

  fn insert_query(namespace: &str, identifiers: &[&str]) -> crate::Result<ArrayStrU8<255>> {
    let mut query =
      query(["INSERT INTO ", namespace, ".session_state (key,csrf,expires_at,expires_at_ub"])?;
    push_identifiers(&mut query, identifiers)?;
    query.push_str(") VALUES ($1,$2,$3,$4")?;
    push_placeholders(&mut query, identifiers.len())?;
    query.push(')')?;
    Ok(query)
  }

  fn push_decimal(query: &mut ArrayStrU8<255>, value: usize) -> crate::Result<()> {
    if value >= 10 {
      push_decimal(query, value / 10)?;
    }
    let num = u32::try_from(value % 10).unwrap_or_default();
    query.push(char::from_digit(num, 10).unwrap_or('0'))
  }

  fn push_identifiers(query: &mut ArrayStrU8<255>, columns: &[&str]) -> crate::Result<()> {
    for column in columns {
      let _ = query.push_strs([",", column])?;
    }
    Ok(())
  }

  fn push_placeholders(query: &mut ArrayStrU8<255>, len: usize) -> crate::Result<()> {
    for idx in 0..len {
      query.push(',')?;
      push_placeholder(query, 5usize.wrapping_add(idx))?;
    }
    Ok(())
  }

  fn push_placeholder(query: &mut ArrayStrU8<255>, value: usize) -> crate::Result<()> {
    query.push('$')?;
    push_decimal(query, value)
  }

  fn read_query(namespace: &str, identifiers: &[&str]) -> crate::Result<ArrayStrU8<255>> {
    let mut query = query(["SELECT csrf,expires_at,expires_at_ub"])?;
    push_identifiers(&mut query, identifiers)?;
    let _ = query.push_strs([" FROM ", namespace, ".session_state WHERE key=$1"])?;
    Ok(query)
  }

  fn update_query(namespace: &str, identifiers: &[&str]) -> crate::Result<ArrayStrU8<255>> {
    let mut query = query([
      "UPDATE ",
      namespace,
      ".session_state SET key=$1,csrf=$2,expires_at=$3,expires_at_ub=$4",
    ])?;
    for (idx, column) in identifiers.iter().enumerate() {
      let _ = query.push_strs([",", column, "="])?;
      push_placeholder(&mut query, 5usize.wrapping_add(idx))?;
    }
    query.push_str(" WHERE key=")?;
    push_placeholder(&mut query, 5usize.wrapping_add(identifiers.len()))?;
    Ok(query)
  }
}
