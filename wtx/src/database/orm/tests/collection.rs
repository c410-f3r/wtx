// C --> A/B

use crate::database::{
  orm::{
    AuxNodes, FromSuffixRslt, NoTableAssociation, SelectLimit, SelectOrderBy, SqlWriter, Table,
    TableAssociation, TableAssociationWrapper, TableField, TableParams,
  },
  TableSuffix,
};
use alloc::{string::String, vec, vec::Vec};
use core::mem;

#[derive(Debug)]
struct A {
  id: i32,
  name: &'static str,
}

impl<'entity> Table<'entity> for A {
  const PRIMARY_KEY_NAME: &'static str = "id";
  const TABLE_NAME: &'static str = "a";

  type Associations = NoTableAssociation<crate::Error>;
  type Database = ();
  type Fields = (TableField<&'static str>,);
  type PrimaryKeyValue = &'entity i32;

  fn type_instances(_: TableSuffix) -> FromSuffixRslt<'entity, Self> {
    (NoTableAssociation::new(), (TableField::new("name"),))
  }

  fn update_all_table_fields(entity: &'entity Self, table: &mut TableParams<'entity, Self>) {
    *table.id_field_mut().value_mut() = Some((&entity.id).into());

    *table.fields_mut().0.value_mut() = Some((entity.name).into());
  }
}

struct B {
  id: i32,
  name: &'static str,
}

impl<'entity> Table<'entity> for B {
  const PRIMARY_KEY_NAME: &'static str = "id";
  const TABLE_NAME: &'static str = "b";

  type Associations = NoTableAssociation<crate::Error>;
  type Database = ();
  type Fields = (TableField<&'static str>,);
  type PrimaryKeyValue = &'entity i32;

  fn type_instances(_: TableSuffix) -> FromSuffixRslt<'entity, Self> {
    (NoTableAssociation::new(), (TableField::new("name"),))
  }

  fn update_all_table_fields(entity: &'entity Self, table: &mut TableParams<'entity, Self>) {
    *table.id_field_mut().value_mut() = Some((&entity.id).into());

    *table.fields_mut().0.value_mut() = Some((entity.name).into());
  }
}

struct C {
  r#as: Vec<A>,
  bs: Vec<B>,
  id: i32,
  name: &'static str,
}

impl<'entity> Table<'entity> for C {
  const PRIMARY_KEY_NAME: &'static str = "id";
  const TABLE_NAME: &'static str = "c";

  type Associations = (
    TableAssociationWrapper<'entity, A, Vec<TableParams<'entity, A>>>,
    TableAssociationWrapper<'entity, B, Vec<TableParams<'entity, B>>>,
  );
  type Database = ();
  type Fields = (TableField<&'static str>,);
  type PrimaryKeyValue = &'entity i32;

  fn type_instances(ts: TableSuffix) -> FromSuffixRslt<'entity, Self> {
    (
      (
        TableAssociationWrapper {
          association: TableAssociation::new("id", "id_a"),
          guide: TableParams::new(ts + 1),
          tables: vec![],
        },
        TableAssociationWrapper {
          association: TableAssociation::new("id", "id_b"),
          guide: TableParams::new(ts + 2),
          tables: vec![],
        },
      ),
      (TableField::new("name"),),
    )
  }

  fn update_all_table_fields(entity: &'entity Self, table: &mut TableParams<'entity, Self>) {
    *table.id_field_mut().value_mut() = Some((&entity.id).into());

    *table.fields_mut().0.value_mut() = Some((entity.name).into());

    table.associations_mut().0.tables.clear();
    for a in entity.r#as.iter() {
      let mut elem = TableParams::new(table.table_suffix() + 1);
      elem.update_all_table_fields(a);
      table.associations_mut().0.tables.push(elem);
    }

    table.associations_mut().1.tables.clear();
    for b in entity.bs.iter() {
      let mut elem = TableParams::new(table.table_suffix() + 2);
      elem.update_all_table_fields(b);
      table.associations_mut().1.tables.push(elem);
    }
  }
}

#[cfg(target_pointer_width = "64")]
#[test]
fn assert_sizes() {
  assert_eq!(mem::size_of::<TableParams<'_, A>>(), 64);
  assert_eq!(mem::size_of::<TableParams<'_, B>>(), 64);
  assert_eq!(mem::size_of::<TableParams<'_, C>>(), 304);
}

#[test]
fn update_some_values_has_correct_behavior() {
  let a1 = A { id: 1, name: "foo1" };
  let a2 = A { id: 2, name: "foo2" };
  let c3 = C { r#as: vec![a1, a2], bs: vec![], id: 3, name: "foo3" };

  let mut buffer = String::new();
  let mut c_table_defs = TableParams::<C>::default();

  *c_table_defs.id_field_mut().value_mut() = Some((&c3.id).into());

  let mut elem = TableParams::new(0);
  *elem.id_field_mut().value_mut() = Some((&c3.r#as[0].id).into());
  c_table_defs.associations_mut().0.tables.push(elem);

  c_table_defs.write_update(&mut AuxNodes::default(), &mut buffer).unwrap();
  assert_eq!(&buffer, r#"UPDATE c SET id='3' WHERE id='3';UPDATE a SET id='1' WHERE id='1';"#);
}

#[test]
fn write_collection_has_correct_params() {
  let a1 = A { id: 1, name: "foo1" };
  let a2 = A { id: 2, name: "foo2" };
  let c3 = C { r#as: vec![a1, a2], bs: vec![], id: 3, name: "foo3" };

  let mut buffer = String::new();
  let mut c_table_defs = TableParams::<C>::default();

  c_table_defs.write_delete(&mut AuxNodes::default(), &mut buffer).unwrap();
  assert_eq!(&buffer, r#""#);

  c_table_defs.write_insert(&mut AuxNodes::default(), &mut buffer, &mut None).unwrap();
  assert_eq!(&buffer, r#""#);

  buffer.clear();
  c_table_defs
    .write_select(&mut buffer, SelectOrderBy::Ascending, SelectLimit::All, &mut |_| Ok(()))
    .unwrap();
  assert_eq!(
    &buffer,
    r#"SELECT "c0".id AS c0__id,"c0".name AS c0__name,"a1".id AS a1__id,"a1".name AS a1__name,"b2".id AS b2__id,"b2".name AS b2__name FROM "c" AS "c0" LEFT JOIN "a" AS "a1" ON "c0".id = "a1".id_a LEFT JOIN "b" AS "b2" ON "c0".id = "b2".id_b  ORDER BY "c0".id,"a1".id,"b2".id ASC LIMIT ALL"#
  );

  buffer.clear();
  c_table_defs.write_update(&mut AuxNodes::default(), &mut buffer).unwrap();
  assert_eq!(&buffer, r#""#);

  c_table_defs.update_all_table_fields(&c3);

  buffer.clear();
  c_table_defs.write_delete(&mut AuxNodes::default(), &mut buffer).unwrap();
  assert_eq!(
    &buffer,
    r#"DELETE FROM a WHERE id='1';DELETE FROM a WHERE id='2';DELETE FROM c WHERE id='3';"#
  );

  buffer.clear();
  c_table_defs.write_insert(&mut AuxNodes::default(), &mut buffer, &mut None).unwrap();
  assert_eq!(
    &buffer,
    r#"INSERT INTO "c" (id,name) VALUES ($1,$2);INSERT INTO "a" (id,name,id_a) VALUES ($1,$2,$3);INSERT INTO "a" (id,name,id_a) VALUES ($1,$2,$3);"#
  );

  buffer.clear();
  c_table_defs
    .write_select(&mut buffer, SelectOrderBy::Ascending, SelectLimit::All, &mut |_| Ok(()))
    .unwrap();
  assert_eq!(
    &buffer,
    r#"SELECT "c0".id AS c0__id,"c0".name AS c0__name,"a1".id AS a1__id,"a1".name AS a1__name,"b2".id AS b2__id,"b2".name AS b2__name FROM "c" AS "c0" LEFT JOIN "a" AS "a1" ON "c0".id = "a1".id_a LEFT JOIN "b" AS "b2" ON "c0".id = "b2".id_b  ORDER BY "c0".id,"a1".id,"b2".id ASC LIMIT ALL"#
  );

  buffer.clear();
  c_table_defs.write_update(&mut AuxNodes::default(), &mut buffer).unwrap();
  assert_eq!(
    &buffer,
    r#"UPDATE c SET id='3',name='foo3' WHERE id='3';UPDATE a SET id='1',name='foo1' WHERE id='1';UPDATE a SET id='2',name='foo2' WHERE id='2';"#
  );
}
