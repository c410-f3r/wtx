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

impl<CS, E> SessionStore<CS, E> for ()
where
  CS: Default,
{
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

#[cfg(feature = "pool")]
mod pool {
  use crate::{
    database::client::postgres::Postgres,
    http::session::{SessionKey, SessionState, SessionStore},
    misc::{Decode, Encode, Lock},
    pool::{ResourceManager, SimplePool, SimplePoolResource},
  };

  impl<CS, E, R, RL, RM> SessionStore<CS, E> for SimplePool<RL, RM>
  where
    CS: for<'de> Decode<'de, Postgres<E>> + Encode<Postgres<E>>,
    E: From<crate::Error>,
    R: SessionStore<CS, E>,
    RL: Lock<Resource = SimplePoolResource<R>>,
    RM: ResourceManager<CreateAux = (), Error = E, RecycleAux = (), Resource = R>,
    for<'any> RL: 'any,
    for<'any> RM: 'any,
  {
    #[inline]
    async fn create(&mut self, state: &SessionState<CS>) -> Result<(), E> {
      self.get().await?.create(state).await
    }

    #[inline]
    async fn delete(&mut self, session_key: &SessionKey) -> Result<(), E> {
      self.get().await?.delete(session_key).await
    }

    #[inline]
    async fn delete_expired(&mut self) -> Result<(), E> {
      self.get().await?.delete_expired().await
    }

    #[inline]
    async fn read(&mut self, session_key: SessionKey) -> Result<Option<SessionState<CS>>, E> {
      self.get().await?.read(session_key).await
    }

    #[inline]
    async fn update(
      &mut self,
      session_key: &SessionKey,
      state: &SessionState<CS>,
    ) -> Result<(), E> {
      self.get().await?.update(session_key, state).await
    }
  }
}

#[cfg(feature = "postgres")]
mod postgres {
  use crate::{
    database::{
      Executor as _, Record, Typed,
      client::postgres::{ExecutorBuffer, Postgres, PostgresExecutor},
    },
    http::session::{SessionKey, SessionState, SessionStore},
    misc::{Decode, Encode, LeaseMut, Stream},
  };

  /// Expects the following SQL table definition in your database. Column names can NOT be changed.
  ///
  /// ```sql
  /// CREATE TABLE "session" (
  ///   key VARCHAR(32) NOT NULL PRIMARY KEY,
  ///   csrf VARCHAR(32) NOT NULL,
  ///   custom_state SOME_CUSTOM_TY NOT NULL,
  ///   expires_at TIMESTAMPTZ NOT NULL
  /// );
  /// ```
  ///
  /// Change `SOME_CUSTOM_TY` to any type you want, just make sure that it implements [`Decode`] and
  /// [`Encode`] in the Rust side.
  impl<CS, E, EB, S> SessionStore<CS, E> for PostgresExecutor<E, EB, S>
  where
    CS: for<'de> Decode<'de, Postgres<E>> + Encode<Postgres<E>> + Typed<Postgres<E>>,
    E: From<crate::Error>,
    EB: LeaseMut<ExecutorBuffer>,
    S: Stream,
  {
    #[inline]
    async fn create(&mut self, state: &SessionState<CS>) -> Result<(), E> {
      let SessionState { custom_state, expires_at, session_csrf, session_key } = state;
      let _ = self
        .execute_with_stmt(
          "INSERT INTO session (key, csrf, custom_state, expires_at) VALUES ($1, $2, $3, $4)",
          (session_key, session_csrf, custom_state, expires_at),
        )
        .await?;
      Ok(())
    }

    #[inline]
    async fn delete(&mut self, session_key: &SessionKey) -> Result<(), E> {
      let _ = self.execute_with_stmt("DELETE FROM session WHERE key=$1", (session_key,)).await?;
      Ok(())
    }

    #[inline]
    async fn delete_expired(&mut self) -> Result<(), E> {
      self.execute("DELETE FROM session WHERE expires_at <= NOW()", |_| Ok(())).await?;
      Ok(())
    }

    #[inline]
    async fn read(&mut self, session_key: SessionKey) -> Result<Option<SessionState<CS>>, E> {
      let rec = self
        .fetch_with_stmt(
          "SELECT csrf, custom_state, expires_at FROM session WHERE key=$1",
          (&session_key,),
        )
        .await?;
      Ok(Some(SessionState {
        session_csrf: rec.decode::<_, &[u8]>(0)?.try_into()?,
        custom_state: rec.decode(1)?,
        expires_at: Some(rec.decode(2)?),
        session_key,
      }))
    }

    #[inline]
    async fn update(
      &mut self,
      session_key: &SessionKey,
      state: &SessionState<CS>,
    ) -> Result<(), E> {
      let SessionState { session_csrf, custom_state, expires_at, session_key: state_session_key } =
        state;
      let _ = self
        .execute_with_stmt(
          "UPDATE session SET key=$1,csrf=$2,custom_state=$3,expires_at=$4 WHERE key=$5",
          (state_session_key, session_csrf, custom_state, expires_at, session_key),
        )
        .await?;
      Ok(())
    }
  }
}
