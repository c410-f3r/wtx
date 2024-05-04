#![allow(
  // Meta variable expressions
  non_snake_case
)]

use crate::{
  database::orm::{
    AuxNodes, FullTableAssociation, SelectLimit, SelectOrderBy, SqlValue, SqlWriter, Table,
    TableAssociationWrapper, TableAssociations, TableField, TableFields, TableParams,
    TableSourceAssociation,
  },
  misc::SingleTypeStorage,
};
use alloc::string::String;
use core::{
  array,
  fmt::{Display, Write},
};

macro_rules! double_tuple_impls {
  ($( ($($T:ident $U:ident),+) )+) => {
    $(
      impl<'entity, $($T, $U,)+> TableAssociations for ($( TableAssociationWrapper<'entity, $U, $T>, )+)
      where
        $(
          $T: crate::misc::Lease<[TableParams<'entity, $U>]> + SingleTypeStorage<Item = TableParams<'entity, $U>>,
          $U: Table<'entity>,
        )+
      {
        type FullTableAssociations = array::IntoIter<
          FullTableAssociation,
          {
            let mut len: usize = 0;
            $({ const $T: usize = 1; len = len.wrapping_add($T); })+
            len
          }
        >;

        #[inline]
        fn full_associations(&self) -> Self::FullTableAssociations {
          let ($($T,)+) = self;
          [
            $(
              FullTableAssociation::new(
                $T.association,
                $U::TABLE_NAME,
                $U::TABLE_NAME_ALIAS,
                $T.guide.table_suffix()
              ),
            )+
          ].into_iter()
        }
      }

      impl<'entity, ERR, $($T, $U,)+> SqlWriter for ($( TableAssociationWrapper<'entity, $U, $T>, )+)
      where
        ERR: From<crate::Error>,
        $(
          $T: crate::misc::Lease<[TableParams<'entity, $U>]> + SingleTypeStorage<Item = TableParams<'entity, $U>>,
          $U: Table<'entity, Error = ERR>,
          $U::Associations: SqlWriter<Error = ERR>,
        )+
      {
        type Error = ERR;

        #[inline]
        fn write_delete(
          &self,
          aux: &mut AuxNodes,
          buffer_cmd: &mut String,
        ) -> Result<(), Self::Error> {
          let ($($T,)+) = self;
          $(
            for elem in $T.tables.lease() {
              elem.write_delete(aux, buffer_cmd)?;
            }
          )+
          Ok(())
        }

        #[inline]
        fn write_insert<VALUE>(
          &self,
          aux: &mut AuxNodes,
          buffer_cmd: &mut String,
          table_source_association: &mut Option<TableSourceAssociation<'_, VALUE>>
        ) -> Result<(), Self::Error>
        where
          VALUE: Display
        {
          let ($($T,)+) = self;
          $(
            if let Some(ref mut elem) = table_source_association.as_mut() {
              *elem.source_field_mut() = $T.association.to_id();
            }
            for elem in $T.tables.lease() {
              elem.write_insert(aux, buffer_cmd, table_source_association)?;
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
          where_cb: &mut impl FnMut(&mut String) -> Result<(), Self::Error>,
        ) -> Result<(), Self::Error> {
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
        ) -> Result<(), Self::Error> {
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
        ) -> Result<(), Self::Error> {
          let ($($T,)+) = self;
          $(
            $T.guide.write_select_fields(buffer_cmd)?;
          )+
          Ok(())
        }

        #[inline]
        fn write_select_orders_by(&self, buffer_cmd: &mut String) -> Result<(), Self::Error> {
          let ($($T,)+) = self;
          $(
            $T.guide.write_select_orders_by(buffer_cmd)?;
          )+
          Ok(())
        }

        #[inline]
        fn write_update(
          &self,
          aux: &mut AuxNodes,
          buffer_cmd: &mut String,
        ) -> Result<(), Self::Error> {
          let ($($T,)+) = self;
          $(
            for elem in $T.tables.lease() {
              elem.write_update(aux, buffer_cmd)?;
            }
          )+
          Ok(())
        }
      }
    )+
  }
}

macro_rules! tuple_impls {
  ($( ($($T:ident),+) )+) => {
    $(
      impl<ERR, $($T: SqlValue<ERR>),+> TableFields<ERR> for ($( TableField<$T>, )+)
      where
        ERR: From<crate::Error>,
      {
        type FieldNames = array::IntoIter<
          &'static str,
          {
            let mut len: usize = 0;
            $({ const $T: usize = 1; len = len.wrapping_add($T); })+
            len
          }
        >;

        #[inline]
        fn field_names(&self) -> Self::FieldNames {
          let ($($T,)+) = self;
          [ $( $T.name(), )+ ].into_iter()
        }

        #[inline]
        fn write_insert_values(&self, buffer_cmd: &mut String) -> Result<(), ERR> {
          let ($($T,)+) = self;
          $(
            if let &Some(ref elem) = $T.value() {
              elem.write(buffer_cmd)?;
              buffer_cmd.push(',');
            }
          )+
          Ok(())
        }

        #[inline]
        fn write_update_values(&self, buffer_cmd: &mut String) -> Result<(), ERR> {
          let ($($T,)+) = self;
          $(
            if let &Some(ref elem) = $T.value() {
              buffer_cmd.write_fmt(format_args!("{}=", $T.name())).map_err(From::from)?;
              elem.write(buffer_cmd)?;
              buffer_cmd.push(',');
            }
          )+
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
  (A, B)
  (A, B, C)
  (A, B, C, D)
  (A, B, C, D, E)
  (A, B, C, D, E, F)
  (A, B, C, D, E, F, G)
  (A, B, C, D, E, F, G, H)
  (A, B, C, D, E, F, G, H, I)
  (A, B, C, D, E, F, G, H, I, J)
  (A, B, C, D, E, F, G, H, I, J, K)
  (A, B, C, D, E, F, G, H, I, J, K, L)
  (A, B, C, D, E, F, G, H, I, J, K, L, M)
  (A, B, C, D, E, F, G, H, I, J, K, L, M, N)
  (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O)
  (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)
}
