# Object–Relational Mapping

A very rudimentary ORM that currently supports very few operations that are not well tested. You probably should look for other similar projects.

Activation feature is called `orm`.

```ignore,rust,edition2021
use wtx::database::{
  orm::{Crud, FromSuffixRslt, NoTableAssociation, Table, TableField, TableParams},
  Database, FromRecords, Record, TableSuffix,
};

struct User<'conn> {
  id: u32,
  name: &'conn str,
  password: &'conn str,
}

impl<'conn> FromRecords for User<'conn> {
  type Database = ();
  type Error = wtx::Error;

  fn from_records(
    _: &mut String,
    curr_record: &<Self::Database as Database>::Record<'_>,
    _: &<Self::Database as Database>::Records<'_>,
    _: TableSuffix,
  ) -> Result<(usize, Self), Self::Error> {
    let id = curr_record.decode(0)?;
    let name = curr_record.decode(1)?;
    let password = curr_record.decode(2)?;
    Ok((1, Self { id, name, password }))
  }
}

impl<'conn, 'entity> Table<'entity> for User<'conn> {
  const PRIMARY_KEY_NAME: &'static str = "id";
  const TABLE_NAME: &'static str = "user";

  type Associations = NoTableAssociation<wtx::Error>;
  type Error = wtx::Error;
  type Fields = (TableField<&'conn str>, TableField<&'conn str>);
  type PrimaryKeyValue = &'entity u32;

  fn type_instances(_: TableSuffix) -> FromSuffixRslt<'entity, Self> {
    (NoTableAssociation::new(), (TableField::new("name"), TableField::new("password")))
  }

  fn update_all_table_fields(entity: &'entity Self, table: &mut TableParams<'entity, Self>) {
    *table.id_field_mut().value_mut() = Some((&entity.id).into());
    *table.fields_mut().0.value_mut() = Some((entity.name).into());
    *table.fields_mut().1.value_mut() = Some((entity.password).into());
  }
}

async fn all_users<'conn>(
  crud: &'conn mut impl Crud<Database = ()>,
) -> wtx::Result<Vec<User<'conn>>> {
  let mut buffer = String::new();
  let mut results = Vec::new();
  crud.read_all::<User<'conn>>(&mut buffer, &mut results, &TableParams::default()).await?;
  Ok(results)
}
```