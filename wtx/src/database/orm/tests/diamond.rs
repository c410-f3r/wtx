// CREATE TABLE d (
//   id INT PRIMARY KEY NOT NULL,
//   name VARCHAR(64) NOT NULL
// );
//
// CREATE TABLE c (
//   id INT PRIMARY KEY NOT NULL,
//   id_d INT NOT NULL,
//   name VARCHAR(64) NOT NULL
// );
// ALTER TABLE c ADD FOREIGN KEY (id_d) REFERENCES d(id);
//
// CREATE TABLE b (
//   id INT PRIMARY KEY NOT NULL,
//   id_d INT NOT NULL,
//   name VARCHAR(64) NOT NULL
// );
// ALTER TABLE b ADD FOREIGN KEY (id_d) REFERENCES d(id);
//
// CREATE TABLE a (
//   id INT PRIMARY KEY NOT NULL,
//   id_b INT NOT NULL,
//   id_c INT NOT NULL,
//   name VARCHAR(64) NOT NULL
// );
// ALTER TABLE a ADD FOREIGN KEY (id_b) REFERENCES b(id);
// ALTER TABLE a ADD FOREIGN KEY (id_c) REFERENCES c(id);
//
// D
// |--> B |
// |--> C |
//        |--> A

use crate::database::{
  orm::{
    AuxNodes, FromSuffixRslt, NoTableAssociation, SelectLimit, SelectOrderBy, SqlWriter, Table,
    TableAssociation, TableAssociationWrapper, TableField, TableParams,
  },
  TableSuffix,
};
use alloc::string::String;

const A: A = A { id: 1, name: "foo1" };
const B: B = B { a: A, id: 2, name: "foo2" };
const C: C = C { a: A, id: 3, name: "foo3" };
const D: D = D { b: B, c: C, id: 4, name: "foo4" };

#[derive(Debug)]
struct A {
  id: u32,
  name: &'static str,
}

impl<'entity> Table<'entity> for A {
  const TABLE_NAME: &'static str = "a";

  type Associations = NoTableAssociation<crate::Error>;
  type Database = ();
  type Fields = (TableField<&'entity u32>, TableField<&'static str>);

  fn type_instances(_: TableSuffix) -> FromSuffixRslt<'entity, Self> {
    (NoTableAssociation::new(), (TableField::new("id"), TableField::new("name")))
  }

  fn update_all_table_fields(&'entity self, table: &mut TableParams<'entity, Self>) {
    *table.fields_mut().0.value_mut() = Some(&self.id);
    *table.fields_mut().1.value_mut() = Some(self.name);
  }
}

struct B {
  a: A,
  id: u32,
  name: &'static str,
}

impl<'entity> Table<'entity> for B {
  const TABLE_NAME: &'static str = "b";

  type Associations = (TableAssociationWrapper<'entity, A, [TableParams<'entity, A>; 1]>,);
  type Database = ();
  type Fields = (TableField<&'entity u32>, TableField<&'static str>);

  fn type_instances(ts: TableSuffix) -> FromSuffixRslt<'entity, Self> {
    (
      (TableAssociationWrapper {
        association: TableAssociation::new("id", false, false, "id_b"),
        guide: TableParams::from_ts(ts + 1),
        tables: [TableParams::from_ts(ts + 1)],
      },),
      (TableField::new("id"), TableField::new("name")),
    )
  }

  fn update_all_table_fields(&'entity self, table: &mut TableParams<'entity, Self>) {
    *table.fields_mut().0.value_mut() = Some(&self.id);
    *table.fields_mut().1.value_mut() = Some(self.name);

    table.associations_mut().0.guide.update_all_table_fields(&self.a);
    table.associations_mut().0.tables[0].update_all_table_fields(&self.a);
  }
}

struct C {
  a: A,
  id: u32,
  name: &'static str,
}

impl<'entity> Table<'entity> for C {
  const TABLE_NAME: &'static str = "c";

  type Associations = (TableAssociationWrapper<'entity, A, [TableParams<'entity, A>; 1]>,);
  type Database = ();
  type Fields = (TableField<&'entity u32>, TableField<&'static str>);

  fn type_instances(ts: TableSuffix) -> FromSuffixRslt<'entity, Self> {
    (
      (TableAssociationWrapper {
        association: TableAssociation::new("id", false, false, "id_c"),
        guide: TableParams::from_ts(ts + 1),
        tables: [TableParams::from_ts(ts + 1)],
      },),
      (TableField::new("id"), TableField::new("name")),
    )
  }

  fn update_all_table_fields(&'entity self, table: &mut TableParams<'entity, Self>) {
    *table.fields_mut().0.value_mut() = Some(&self.id);
    *table.fields_mut().1.value_mut() = Some(self.name);

    table.associations_mut().0.guide.update_all_table_fields(&self.a);
    table.associations_mut().0.tables[0].update_all_table_fields(&self.a);
  }
}

struct D {
  b: B,
  c: C,
  id: u32,
  name: &'static str,
}

impl<'entity> Table<'entity> for D {
  const TABLE_NAME: &'static str = "d";

  type Associations = (
    TableAssociationWrapper<'entity, B, [TableParams<'entity, B>; 1]>,
    TableAssociationWrapper<'entity, C, [TableParams<'entity, C>; 1]>,
  );
  type Database = ();
  type Fields = (TableField<&'entity u32>, TableField<&'static str>);

  fn type_instances(suffix: TableSuffix) -> FromSuffixRslt<'entity, Self> {
    (
      (
        TableAssociationWrapper {
          association: TableAssociation::new("id", false, false, "id_d"),
          guide: TableParams::from_ts(suffix + 1),
          tables: [TableParams::from_ts(suffix + 1)],
        },
        TableAssociationWrapper {
          association: TableAssociation::new("id", false, false, "id_d"),
          guide: TableParams::from_ts(suffix + 2),
          tables: [TableParams::from_ts(suffix + 2)],
        },
      ),
      (TableField::new("id"), TableField::new("name")),
    )
  }

  fn update_all_table_fields(&'entity self, table: &mut TableParams<'entity, Self>) {
    *table.fields_mut().0.value_mut() = Some(&self.id);
    *table.fields_mut().1.value_mut() = Some(self.name);

    table.associations_mut().0.guide.update_all_table_fields(&self.b);
    table.associations_mut().0.tables[0].update_all_table_fields(&self.b);

    table.associations_mut().1.guide.update_all_table_fields(&self.c);
    table.associations_mut().1.tables[0].update_all_table_fields(&self.c);
  }
}

#[cfg(target_pointer_width = "64")]
#[test]
fn assert_sizes() {
  assert_eq!(size_of::<TableParams<'_, A>>(), 64);
  assert_eq!(size_of::<TableParams<'_, B>>(), 232);
  assert_eq!(size_of::<TableParams<'_, C>>(), 232);
  assert_eq!(size_of::<TableParams<'_, D>>(), 1072);
}

#[tokio::test]
async fn multi_referred_table() {
  let mut buffer = String::new();
  let mut d_table_defs = TableParams::<D>::default();

  d_table_defs
    .write_select(&mut buffer, SelectOrderBy::Ascending, SelectLimit::All, &mut |_| Ok(()))
    .unwrap();
  assert_eq!(
    &buffer,
    r#"SELECT "d0".id AS d0__id,"d0".name AS d0__name,"b1".id AS b1__id,"b1".name AS b1__name,"a2".id AS a2__id,"a2".name AS a2__name,"c2".id AS c2__id,"c2".name AS c2__name,"a3".id AS a3__id,"a3".name AS a3__name FROM "d" AS "d0" LEFT JOIN "b" AS "b1" ON "d0".id = "b1".id_d LEFT JOIN "c" AS "c2" ON "d0".id = "c2".id_d LEFT JOIN "a" AS "a2" ON "b1".id = "a2".id_b LEFT JOIN "a" AS "a3" ON "c2".id = "a3".id_c  ORDER BY "d0".id,"b1".id,"a2".id,"c2".id,"a3".id ASC LIMIT ALL"#
  );

  d_table_defs.update_all_table_fields(&D);

  buffer.clear();
  d_table_defs.write_delete(&mut AuxNodes::default(), &mut buffer, &mut ()).await.unwrap();
  assert_eq!(
    &buffer,
    r#"DELETE FROM a WHERE id='1';DELETE FROM b WHERE id='2';DELETE FROM c WHERE id='3';DELETE FROM d WHERE id='4';"#
  );

  buffer.clear();
  d_table_defs
    .write_insert(&mut AuxNodes::default(), &mut buffer, &mut (), (false, None))
    .await
    .unwrap();
  assert_eq!(
    &buffer,
    // FIXME
    // INSERT INTO "d" (id,name) VALUES ($1,$2);INSERT INTO "b" (id,name,id_d) VALUES ($1,$2,$3);INSERT INTO "c" (id,name,id_d) VALUES ($1,$2,$3);INSERT INTO "a" (id,name,id_b,id_c) VALUES ($1,$2,$3);
    r#"INSERT INTO "d" (id,name) VALUES ($1,$2);INSERT INTO "b" (id,name,id_d) VALUES ($1,$2,2);INSERT INTO "a" (id,name,id_b) VALUES ($1,$2,1);INSERT INTO "c" (id,name,id_d) VALUES ($1,$2,3);"#
  );

  buffer.clear();
  d_table_defs.write_update(&mut AuxNodes::default(), &mut buffer, &mut ()).await.unwrap();
  assert_eq!(
    &buffer,
    r#"UPDATE d SET id='4',name='foo4' WHERE id='4';UPDATE b SET id='2',name='foo2' WHERE id='2';UPDATE a SET id='1',name='foo1' WHERE id='1';UPDATE c SET id='3',name='foo3' WHERE id='3';"#
  );
}

#[tokio::test]
async fn referred_table() {
  let mut buffer = String::new();
  let mut b_table_defs = TableParams::<B>::default();
  b_table_defs
    .write_select(&mut buffer, SelectOrderBy::Ascending, SelectLimit::All, &mut |_| Ok(()))
    .unwrap();
  assert_eq!(
    &buffer,
    r#"SELECT "b0".id AS b0__id,"b0".name AS b0__name,"a1".id AS a1__id,"a1".name AS a1__name FROM "b" AS "b0" LEFT JOIN "a" AS "a1" ON "b0".id = "a1".id_b  ORDER BY "b0".id,"a1".id ASC LIMIT ALL"#
  );

  b_table_defs.update_all_table_fields(&B);

  buffer.clear();
  b_table_defs.write_delete(&mut AuxNodes::default(), &mut buffer, &mut ()).await.unwrap();
  assert_eq!(&buffer, r#"DELETE FROM a WHERE id='1';DELETE FROM b WHERE id='2';"#);

  buffer.clear();
  b_table_defs
    .write_insert(&mut AuxNodes::default(), &mut buffer, &mut (), (false, None))
    .await
    .unwrap();
  assert_eq!(
    &buffer,
    r#"INSERT INTO "b" (id,name) VALUES ($1,$2);INSERT INTO "a" (id,name,id_b) VALUES ($1,$2,1);"#
  );

  buffer.clear();
  b_table_defs.write_update(&mut AuxNodes::default(), &mut buffer, &mut ()).await.unwrap();
  assert_eq!(
    &buffer,
    r#"UPDATE b SET id='2',name='foo2' WHERE id='2';UPDATE a SET id='1',name='foo1' WHERE id='1';"#
  );
}

#[tokio::test]
async fn standalone_table() {
  let mut buffer = String::new();
  let mut a_table_defs = TableParams::<A>::default();
  a_table_defs
    .write_select(&mut buffer, SelectOrderBy::Ascending, SelectLimit::All, &mut |_| Ok(()))
    .unwrap();
  assert_eq!(
    &buffer,
    r#"SELECT "a0".id AS a0__id,"a0".name AS a0__name FROM "a" AS "a0"  ORDER BY "a0".id ASC LIMIT ALL"#
  );

  a_table_defs.update_all_table_fields(&A);

  buffer.clear();
  a_table_defs.write_delete(&mut AuxNodes::default(), &mut buffer, &mut ()).await.unwrap();
  assert_eq!(&buffer, r#"DELETE FROM a WHERE id='1';"#);

  buffer.clear();
  a_table_defs
    .write_insert(&mut AuxNodes::default(), &mut buffer, &mut (), (false, None))
    .await
    .unwrap();
  assert_eq!(&buffer, r#"INSERT INTO "a" (id,name) VALUES ($1,$2);"#);

  buffer.clear();
  a_table_defs.write_update(&mut AuxNodes::default(), &mut buffer, &mut ()).await.unwrap();
  assert_eq!(&buffer, r#"UPDATE a SET id='1',name='foo1' WHERE id='1';"#);
}
