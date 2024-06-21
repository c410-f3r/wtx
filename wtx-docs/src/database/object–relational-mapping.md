# Object–Relational Mapping

A very rudimentary ORM that currently supports very few operations that are not well tested. You probably should look for other similar projects.

Activation feature is called `orm`.

```rust,edition2021
extern crate wtx;

use wtx::{
  database::{
    client::postgres::{Postgres, Record, Records},
    orm::{Crud, FromSuffixRslt, NoTableAssociation, Table, TableField, TableParams},
    FromRecords, Record as _, TableSuffix,
  },
  misc::AsyncBounds,
};

type Db = Postgres<wtx::Error>;

struct User<'entity> {
  id: u32,
  name: &'entity str,
  password: &'entity str,
}

impl<'any> FromRecords<'any, Db> for User<'any> {
  fn from_records(
    _: &mut String,
    curr_record: &Record<'any, wtx::Error>,
    _: &Records<'any, wtx::Error>,
    _: TableSuffix,
  ) -> Result<(usize, Self), wtx::Error> {
    let id = curr_record.decode(0)?;
    let name = curr_record.decode(1)?;
    let password = curr_record.decode(2)?;
    Ok((1, Self { id, name, password }))
  }
}

impl<'entity> Table<'entity> for User<'entity> {
  const TABLE_NAME: &'static str = "user";

  type Associations = NoTableAssociation<wtx::Error>;
  type Database = Db;
  type Fields = (TableField<&'entity u32>, TableField<&'entity str>, TableField<&'entity str>);

  fn type_instances(_: TableSuffix) -> FromSuffixRslt<'entity, Self> {
    (
      NoTableAssociation::new(),
      (TableField::new("id"), TableField::new("name"), TableField::new("password")),
    )
  }

  fn update_all_table_fields(&'entity self, table: &mut TableParams<'entity, Self>) {
    *table.fields_mut().0.value_mut() = Some(&self.id);
    *table.fields_mut().1.value_mut() = Some(self.name);
    *table.fields_mut().2.value_mut() = Some(self.password);
  }
}

async fn all_users<C>(crud: &mut C) -> wtx::Result<Vec<User<'_>>>
where
  C: AsyncBounds + Crud<Database = Db>,
{
  let mut buffer = String::new();
  let mut results = Vec::new();
  crud
    .read_all(&mut buffer, &TableParams::default(), |result| {
      results.push(result);
      Ok(())
    })
    .await?;
  Ok(results)
}
```