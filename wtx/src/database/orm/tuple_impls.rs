#![expect(non_snake_case, reason = "meta variable expressions")]

use crate::{
  database::{
    orm::{
      AuxNodes, FullTableAssociation, IdValue, SelectLimit, SelectOrderBy, SqlValue, SqlWriter,
      Table, TableAssociationWrapper, TableAssociations, TableField, TableFields, TableParams,
    },
    Database, Encode, Executor,
  },
  misc::FilledBufferWriter,
};
use alloc::string::String;
use core::fmt::Write;

macro_rules! double_tuple_impls {
  ($( ($($T:ident $U:ident),+) )+) => {
    $(
      impl<'entity, DB, $($T, $U,)+> TableAssociations for ($( TableAssociationWrapper<'entity, $U, $T>, )+)
      where
        DB: Database,
        $(
          $T: crate::misc::LeaseMut<[TableParams<'entity, $U>]>,
          $U: Table<'entity, Database = DB>,
          $U::Associations: SqlWriter<DB>,
        )+
      {
        #[inline]
        fn full_associations(&self) -> impl Iterator<Item = FullTableAssociation> {
          let ($($T,)+) = self;
          [
            $(
              FullTableAssociation::new(
                $T.association,
                $T.guide.fields().id().value().map(|el| el.generic()),
                $U::TABLE_NAME,
                $U::TABLE_NAME_ALIAS,
                $T.guide.table_suffix()
              ),
            )+
          ].into_iter()
        }
      }

      impl<'entity, DB, $($T, $U,)+> SqlWriter<DB> for ($( TableAssociationWrapper<'entity, $U, $T>, )+)
      where
        DB: Database,
        $(
          $T: crate::misc::LeaseMut<[TableParams<'entity, $U>]>,
          $U: Table<'entity, Database = DB>,
          $U::Associations: SqlWriter<DB>,
        )+
      {
        #[inline]
        async fn write_delete<EX>(
          &mut self,
          aux: &mut AuxNodes,
          buffer_cmd: &mut String,
          executor: &mut EX
        ) -> Result<(), DB::Error>
        where
          EX: Executor<Database = DB>
        {
          let ($($T,)+) = self;
          $(
            for elem in $T.tables.lease_mut() {
              elem.write_delete(aux, buffer_cmd, executor).await?;
            }
          )+
          Ok(())
        }

        #[inline]
        async fn write_insert<EX>(
          &mut self,
          aux: &mut AuxNodes,
          buffer_cmd: &mut String,
          executor: &mut EX,
          (is_initial, _): (bool, Option<(&'static str, u64)>),
        ) -> Result<(), DB::Error>
        where
          EX: Executor<Database = DB>
        {
          let ($($T,)+) = self;
          $(
            if !$T.association.skip_insert() {
              match (is_initial, $T.association.has_inverse_flow()) {
                (false, false) => {
                  for elem in $T.tables.lease_mut() {
                    elem.write_insert(
                      aux,
                      buffer_cmd,
                      executor,
                      (false, $T.guide.fields().id().value().map(|el| ($T.association.to_id_name(), el.generic())))
                    ).await?;
                  }
                },
                (true, true) => {
                  for elem in $T.tables.lease_mut() {
                    elem.write_insert(aux, buffer_cmd, executor, (false, None)).await?;
                  }
                },
                (false, true) | (true, false) => {},
              }
            }
          )+
          Ok(())
        }

        #[inline]
        fn write_select(
          &self,
          buffer_cmd: &mut String,
          order_by: SelectOrderBy,
          limit: SelectLimit,
          where_cb: &mut impl FnMut(&mut String) -> Result<(), DB::Error>,
        ) -> Result<(), DB::Error> {
          let ($($T,)+) = self;
          $(
            $T.guide.write_select(buffer_cmd, order_by, limit, where_cb)?;
          )+
          Ok(())
        }

        #[inline]
        fn write_select_associations(
          &self,
            buffer_cmd: &mut String,
        ) -> Result<(), DB::Error> {
          let ($($T,)+) = self;
          $(
            $T.guide.write_select_associations(buffer_cmd)?;
          )+
          Ok(())
        }

        #[inline]
        fn write_select_fields(
          &self,
            buffer_cmd: &mut String,
        ) -> Result<(), DB::Error> {
          let ($($T,)+) = self;
          $(
            $T.guide.write_select_fields(buffer_cmd)?;
          )+
          Ok(())
        }

        #[inline]
        fn write_select_orders_by(&self, buffer_cmd: &mut String) -> Result<(), DB::Error> {
          let ($($T,)+) = self;
          $(
            $T.guide.write_select_orders_by(buffer_cmd)?;
          )+
          Ok(())
        }

        #[inline]
        async fn write_update<EX>(
          &mut self,
          aux: &mut AuxNodes,
          buffer_cmd: &mut String,
          executor: &mut EX
        ) -> Result<(), DB::Error>
        where
          EX: Executor
        {
          let ($($T,)+) = self;
          $(
            for elem in $T.tables.lease_mut() {
              elem.write_update(aux, buffer_cmd, executor).await?;
            }
          )+
          Ok(())
        }
      }
    )+
  }
}

macro_rules! tuple_impls {
  ($( ($T0:ident $($T:ident)*) )+) => {
    $(
      impl<DB, $T0, $($T,)*> Encode<DB> for (TableField<$T0>, $( TableField<$T>, )*)
      where
        DB: Database,
        $T0: Encode<DB> + SqlValue<DB::Error>,
        $($T: Encode<DB> + SqlValue<DB::Error>,)*
      {
        #[inline]
        fn encode(
          &self,
          fbw: &mut FilledBufferWriter<'_>,
          value: &DB::EncodeValue<'_>,
        ) -> Result<(), DB::Error> {
          let ($T0, $($T,)*) = self;
          $T0.value().encode(fbw, value)?;
          $( $T.value().encode(fbw, value)?; )*
          Ok(())
        }

        #[inline]
        fn is_null(&self) -> bool {
          false
        }
      }

      impl<DB, $T0, $($T,)*> TableFields<DB> for (TableField<$T0>, $( TableField<$T>, )*)
      where
        DB: Database,
        $T0: IdValue<DB::Error>,
        $($T: Encode<DB> + SqlValue<DB::Error>,)*
      {
        type IdValue = $T0;

        #[inline]
        fn field_names(&self) -> impl Iterator<Item = &'static str> {
          let ($T0, $($T,)*) = self;
          [ $T0.name(), $( $T.name(), )* ].into_iter()
        }

        #[inline]
        fn id(&self) -> &TableField<Self::IdValue> {
          &self.0
        }

        #[inline]
        fn opt_fields(&self) -> impl Iterator<Item = bool> {
          let ($T0, $($T,)*) = self;
          [ $T0.value().is_none(), $( $T.value().is_none(), )* ].into_iter()
        }

        #[inline]
        fn write_insert_values(&self, buffer_cmd: &mut String) -> Result<(), DB::Error> {
          let ($T0, $($T,)*) = self;
          if let Some(elem) = $T0.value() {
            elem.write(buffer_cmd)?;
            buffer_cmd.push(',');
          }
          $(
            if let Some(elem) = $T.value() {
              elem.write(buffer_cmd)?;
              buffer_cmd.push(',');
            }
          )*
          Ok(())
        }

        #[inline]
        fn write_update_values(&self, buffer_cmd: &mut String) -> Result<(), DB::Error> {
          let ($T0, $($T,)*) = self;
          if let Some(elem) = $T0.value() {
            buffer_cmd.write_fmt(format_args!("{}=", $T0.name())).map_err(From::from)?;
            elem.write(buffer_cmd)?;
            buffer_cmd.push(',');
          }
          $(
            if let Some(elem) = $T.value() {
              buffer_cmd.write_fmt(format_args!("{}=", $T.name())).map_err(From::from)?;
              elem.write(buffer_cmd)?;
              buffer_cmd.push(',');
            }
          )*
          Ok(())
        }
      }
    )+
  }
}

double_tuple_impls! {
  (A B)
  (A B, C D)
  (A B, C D, E F)
  (A B, C D, E F, G H)
  (A B, C D, E F, G H, I J)
  (A B, C D, E F, G H, I J, K L)
  (A B, C D, E F, G H, I J, K L, M N)
  (A B, C D, E F, G H, I J, K L, M N, O P)
  (A B, C D, E F, G H, I J, K L, M N, O P, Q R)
  (A B, C D, E F, G H, I J, K L, M N, O P, Q R, S T)
  (A B, C D, E F, G H, I J, K L, M N, O P, Q R, S T, U V)
  (A B, C D, E F, G H, I J, K L, M N, O P, Q R, S T, U V, W X)
  (A B, C D, E F, G H, I J, K L, M N, O P, Q R, S T, U V, W X, Y Z)
  (A B, C D, E F, G H, I J, K L, M N, O P, Q R, S T, U V, W X, Y Z, AA AB)
  (A B, C D, E F, G H, I J, K L, M N, O P, Q R, S T, U V, W X, Y Z, AA AB, AC AD)
  (A B, C D, E F, G H, I J, K L, M N, O P, Q R, S T, U V, W X, Y Z, AA AB, AC AD, AE AF)
}

tuple_impls! {
  (A)
  (A B)
  (A B C)
  (A B C D)
  (A B C D E)
  (A B C D E F)
  (A B C D E F G)
  (A B C D E F G H)
  (A B C D E F G H I)
  (A B C D E F G H I J)
  (A B C D E F G H I J K)
  (A B C D E F G H I J K L)
  (A B C D E F G H I J K L M)
  (A B C D E F G H I J K L M N)
  (A B C D E F G H I J K L M N O)
  (A B C D E F G H I J K L M N O P)
}
