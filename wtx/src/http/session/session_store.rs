use crate::http::{session::SessionId, SessionState};
use core::future::Future;

/// Abstraction for different session storages.
pub trait SessionStore<CS, E> {
  /// Stores a new [`SessionState`].
  fn create(&mut self, state: &SessionState<CS>) -> impl Future<Output = Result<(), E>>;

  /// Removes the [`SessionState`] that is identified by `id`.
  fn delete(&mut self, id: &SessionId) -> impl Future<Output = Result<(), E>>;

  /// Removes all expired sessions.
  fn delete_expired(&mut self) -> impl Future<Output = Result<(), E>>;

  /// Loads the [`SessionState`] that is identified by `id`.
  fn read(&mut self, id: &SessionId) -> impl Future<Output = Result<Option<SessionState<CS>>, E>>;

  /// Overwrites the [`SessionState`] that is identified by `id` with the contents of
  /// `state`.
  fn update(
    &mut self,
    id: &SessionId,
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
  async fn delete(&mut self, _: &SessionId) -> Result<(), E> {
    Ok(())
  }

  #[inline]
  async fn delete_expired(&mut self) -> Result<(), E> {
    Ok(())
  }

  #[inline]
  async fn read(&mut self, _: &SessionId) -> Result<Option<SessionState<CS>>, E> {
    Ok(None)
  }

  #[inline]
  async fn update(&mut self, _: &SessionId, _: &SessionState<CS>) -> Result<(), E> {
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
  async fn delete(&mut self, id: &SessionId) -> Result<(), E> {
    (*self).delete(id).await
  }

  #[inline]
  async fn delete_expired(&mut self) -> Result<(), E> {
    (*self).delete_expired().await
  }

  #[inline]
  async fn read(&mut self, id: &SessionId) -> Result<Option<SessionState<CS>>, E> {
    (*self).read(id).await
  }

  #[inline]
  async fn update(&mut self, id: &SessionId, state: &SessionState<CS>) -> Result<(), E> {
    (*self).update(id, state).await
  }
}

#[cfg(feature = "pool")]
mod pool {
  use crate::{
    database::{client::postgres::Postgres, Decode, Encode},
    http::session::{SessionId, SessionState, SessionStore},
    misc::Lock,
    pool::{ResourceManager, SimplePool, SimplePoolResource},
    Error,
  };

  impl<CS, E, R, RL, RM> SessionStore<CS, E> for SimplePool<RL, RM>
  where
    CS: for<'de> Decode<'de, Postgres<E>> + Encode<Postgres<E>>,
    E: From<Error>,
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
    async fn delete(&mut self, id: &SessionId) -> Result<(), E> {
      self.get().await?.delete(id).await
    }

    #[inline]
    async fn delete_expired(&mut self) -> Result<(), E> {
      self.get().await?.delete_expired().await
    }

    #[inline]
    async fn read(&mut self, id: &SessionId) -> Result<Option<SessionState<CS>>, E> {
      self.get().await?.read(id).await
    }

    #[inline]
    async fn update(&mut self, id: &SessionId, state: &SessionState<CS>) -> Result<(), E> {
      self.get().await?.update(id, state).await
    }
  }
}

#[cfg(feature = "postgres")]
mod postgres {
  use crate::{
    database::{
      client::postgres::{Executor, ExecutorBuffer, Postgres},
      Decode, Encode, Executor as _, Record,
    },
    http::session::{SessionId, SessionState, SessionStore},
    misc::{LeaseMut, Stream},
    Error,
  };

  /// Expects the following SQL table definition in your database. Column names can be changed
  /// but not their sequence.
  ///
  /// ```sql
  /// CREATE TABLE session (
  ///   id BYTEA NOT NULL PRIMARY KEY,
  ///   expires_at TIMESTAMPTZ NOT NULL,
  ///   custom_state SOME_CUSTOM_TY NOT NULL
  /// );
  /// ```
  ///
  /// Change `SOME_TY` to any type you want, just make sure that it implements [`Decode`] and
  /// [`Encode`] in the Rust side.
  impl<CS, E, EB, S> SessionStore<CS, E> for Executor<E, EB, S>
  where
    CS: for<'de> Decode<'de, Postgres<E>> + Encode<Postgres<E>>,
    E: From<Error>,
    EB: LeaseMut<ExecutorBuffer>,
    S: Stream,
  {
    #[inline]
    async fn create(&mut self, state: &SessionState<CS>) -> Result<(), E> {
      let _ = self
        .execute_with_stmt(
          "INSERT INTO session VALUES ($1, $2, $3)",
          (state.id.as_slice(), state.expires, &state.custom_state),
        )
        .await?;
      Ok(())
    }

    #[inline]
    async fn delete(&mut self, id: &SessionId) -> Result<(), E> {
      let _ = self.execute_with_stmt("DELETE FROM session WHERE id=$1", (id.as_slice(),)).await?;
      Ok(())
    }

    #[inline]
    async fn delete_expired(&mut self) -> Result<(), E> {
      self.execute("DELETE FROM session WHERE expires_at <= NOW()", |_| {}).await?;
      Ok(())
    }

    #[inline]
    async fn read(&mut self, id: &SessionId) -> Result<Option<SessionState<CS>>, E> {
      let rec = self.fetch_with_stmt("SELECT * FROM session WHERE id=$1", (id.as_slice(),)).await?;
      Ok(Some(SessionState {
        custom_state: rec.decode(2)?,
        expires: Some(rec.decode(1)?),
        id: *id,
      }))
    }

    #[inline]
    async fn update(&mut self, id: &SessionId, state: &SessionState<CS>) -> Result<(), E> {
      let _ = self
        .execute_with_stmt(
          "UPDATE session SET id=$1,expires_at=$2,user_state=$3 WHERE id=$4",
          (state.id.as_slice(), state.expires, &state.custom_state, id.as_slice()),
        )
        .await?;
      Ok(())
    }
  }
}
