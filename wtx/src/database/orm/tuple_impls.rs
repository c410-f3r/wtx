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
  ($(
    $tuple_len:tt {
      $(($idx:tt) -> $T:ident $U:ident)+
    }
  )+) => {
    $(
      impl<'entity, $($T, $U,)+> TableAssociations for ($( TableAssociationWrapper<'entity, $U, $T>, )+)
      where
        $(
          $T: AsRef<[TableParams<'entity, $U>]> + SingleTypeStorage<Item = TableParams<'entity, $U>>,
          $U: Table<'entity>,
        )+
      {
        type FullTableAssociations = array::IntoIter<FullTableAssociation, $tuple_len>;

        #[inline]
        fn full_associations(&self) -> Self::FullTableAssociations {
          [
            $(
              FullTableAssociation::new(
                self.$idx.association,
                $U::TABLE_NAME,
                $U::TABLE_NAME_ALIAS,
                self.$idx.guide.table_suffix()
              ),
            )+
          ].into_iter()
        }
      }

      impl<'entity, ERR, $($T, $U,)+> SqlWriter for ($( TableAssociationWrapper<'entity, $U, $T>, )+)
      where
        ERR: From<crate::Error>,
        $(
          $T: AsRef<[TableParams<'entity, $U>]> + SingleTypeStorage<Item = TableParams<'entity, $U>>,
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
          $(
            for elem in self.$idx.tables.as_ref() {
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
          $(
            if let Some(ref mut elem) = table_source_association.as_mut() {
              *elem.source_field_mut() = self.$idx.association.to_id();
            }
            for elem in self.$idx.tables.as_ref() {
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
          $(
            self.$idx.guide.write_select(buffer_cmd, order_by, limit, where_cb)?;
          )+
          Ok(())
        }

        #[inline]
        fn write_select_associations(
          &self,
            buffer_cmd: &mut String,
        ) -> Result<(), Self::Error> {
          $(
            self.$idx.guide.write_select_associations(buffer_cmd)?;
          )+
          Ok(())
        }

        #[inline]
        fn write_select_fields(
          &self,
            buffer_cmd: &mut String,
        ) -> Result<(), Self::Error> {
          $(
            self.$idx.guide.write_select_fields(buffer_cmd)?;
          )+
          Ok(())
        }

        #[inline]
        fn write_select_orders_by(&self, buffer_cmd: &mut String) -> Result<(), Self::Error> {
          $(
            self.$idx.guide.write_select_orders_by(buffer_cmd)?;
          )+
          Ok(())
        }

        #[inline]
        fn write_update(
          &self,
          aux: &mut AuxNodes,
          buffer_cmd: &mut String,
        ) -> Result<(), Self::Error> {
          $(
            for elem in self.$idx.tables.as_ref() {
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
  ($(
    $tuple_len:tt {
      $(($idx:tt) -> $T:ident)+
    }
  )+) => {
    $(
      impl<ERR, $($T: SqlValue<ERR>),+> TableFields<ERR> for ($( TableField<$T>, )+)
      where
        ERR: From<crate::Error>,
      {
        type FieldNames = array::IntoIter<&'static str, $tuple_len>;

        #[inline]
        fn field_names(&self) -> Self::FieldNames {
          [ $( self.$idx.name(), )+ ].into_iter()
        }

        #[inline]
        fn write_insert_values(&self, buffer_cmd: &mut String) -> Result<(), ERR> {
          $(
            if let &Some(ref elem) = self.$idx.value() {
              elem.write(buffer_cmd)?;
              buffer_cmd.push(',');
            }
          )+
          Ok(())
        }

        #[inline]
        fn write_update_values(&self, buffer_cmd: &mut String) -> Result<(), ERR> {
          $(
            if let &Some(ref elem) = self.$idx.value() {
              buffer_cmd.write_fmt(format_args!("{}=", self.$idx.name())).map_err(From::from)?;
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
  1 {
    (0) -> A B
  }
  2 {
    (0) -> A B
    (1) -> C D
  }
  3 {
    (0) -> A B
    (1) -> C D
    (2) -> E F
  }
  4 {
    (0) -> A B
    (1) -> C D
    (2) -> E F
    (3) -> G H
  }
  5 {
    (0) -> A B
    (1) -> C D
    (2) -> E F
    (3) -> G H
    (4) -> I J
  }
  6 {
    (0) -> A B
    (1) -> C D
    (2) -> E F
    (3) -> G H
    (4) -> I J
    (5) -> K L
  }
  7 {
    (0) -> A B
    (1) -> C D
    (2) -> E F
    (3) -> G H
    (4) -> I J
    (5) -> K L
    (6) -> M N
  }
  8 {
    (0) -> A B
    (1) -> C D
    (2) -> E F
    (3) -> G H
    (4) -> I J
    (5) -> K L
    (6) -> M N
    (7) -> O P
  }
  9 {
    (0) -> A B
    (1) -> C D
    (2) -> E F
    (3) -> G H
    (4) -> I J
    (5) -> K L
    (6) -> M N
    (7) -> O P
    (8) -> Q R
  }
  10 {
    (0) -> A B
    (1) -> C D
    (2) -> E F
    (3) -> G H
    (4) -> I J
    (5) -> K L
    (6) -> M N
    (7) -> O P
    (8) -> Q R
    (9) -> S T
  }
  11 {
    (0) -> A B
    (1) -> C D
    (2) -> E F
    (3) -> G H
    (4) -> I J
    (5) -> K L
    (6) -> M N
    (7) -> O P
    (8) -> Q R
    (9) -> S T
    (10) -> U V
  }
  12 {
    (0) -> A B
    (1) -> C D
    (2) -> E F
    (3) -> G H
    (4) -> I J
    (5) -> K L
    (6) -> M N
    (7) -> O P
    (8) -> Q R
    (9) -> S T
    (10) -> U V
    (11) -> W X
  }
  13 {
    (0) -> A B
    (1) -> C D
    (2) -> E F
    (3) -> G H
    (4) -> I J
    (5) -> K L
    (6) -> M N
    (7) -> O P
    (8) -> Q R
    (9) -> S T
    (10) -> U V
    (11) -> W X
    (12) -> Y Z
  }
  14 {
    (0) -> A B
    (1) -> C D
    (2) -> E F
    (3) -> G H
    (4) -> I J
    (5) -> K L
    (6) -> M N
    (7) -> O P
    (8) -> Q R
    (9) -> S T
    (10) -> U V
    (11) -> W X
    (12) -> Y Z
    (13) -> AA AB
  }
  15 {
    (0) -> A B
    (1) -> C D
    (2) -> E F
    (3) -> G H
    (4) -> I J
    (5) -> K L
    (6) -> M N
    (7) -> O P
    (8) -> Q R
    (9) -> S T
    (10) -> U V
    (11) -> W X
    (12) -> Y Z
    (13) -> AA AB
    (14) -> AC AD
  }
  16 {
    (0) -> A B
    (1) -> C D
    (2) -> E F
    (3) -> G H
    (4) -> I J
    (5) -> K L
    (6) -> M N
    (7) -> O P
    (8) -> Q R
    (9) -> S T
    (10) -> U V
    (11) -> W X
    (12) -> Y Z
    (13) -> AA AB
    (14) -> AC AD
    (15) -> AE AF
  }
}

tuple_impls! {
  1 {
    (0) -> A
  }
  2 {
    (0) -> A
    (1) -> B
  }
  3 {
    (0) -> A
    (1) -> B
    (2) -> C
  }
  4 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
  }
  5 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
  }
  6 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
  }
  7 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
  }
  8 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
  }
  9 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
  }
  10 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
    (9) -> J
  }
  11 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
    (9) -> J
    (10) -> K
  }
  12 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
    (9) -> J
    (10) -> K
    (11) -> L
  }
  13 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
    (9) -> J
    (10) -> K
    (11) -> L
    (12) -> M
  }
  14 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
    (9) -> J
    (10) -> K
    (11) -> L
    (12) -> M
    (13) -> N
  }
  15 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
    (9) -> J
    (10) -> K
    (11) -> L
    (12) -> M
    (13) -> N
    (14) -> O
  }
  16 {
    (0) -> A
    (1) -> B
    (2) -> C
    (3) -> D
    (4) -> E
    (5) -> F
    (6) -> G
    (7) -> H
    (8) -> I
    (9) -> J
    (10) -> K
    (11) -> L
    (12) -> M
    (13) -> N
    (14) -> O
    (15) -> P
  }
}
