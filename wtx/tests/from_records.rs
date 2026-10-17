//! FromRecords tests

#![cfg(feature = "database")]

use wtx::{
  collections::VectorU8,
  database::{
    DbClient, FromRecords,
    client::mock_db::{MockClient, MockDb},
  },
};

// 1 2 | 1 1 2
// 1 2 | 2 3 4
#[wtx::test]
async fn from_records_basic_nested() {
  #[derive(Debug, PartialEq, wtx::FromRecords)]
  #[from_records(MockDb)]
  struct Group {
    #[from_records(id)]
    id: u32,

    field0: u8,
    #[from_records(many)]
    field1: VectorU8<Single>,
  }

  #[derive(Debug, PartialEq, wtx::FromRecords)]
  #[from_records(MockDb)]
  struct Single {
    #[from_records(id)]
    id: u32,

    field1: u8,
    field2: u8,
  }

  let mut records = VectorU8::new();
  {
    let mut record = VectorU8::new();
    record.push(("id".try_into().unwrap(), 1u32.to_be_bytes().try_into().unwrap())).unwrap();
    record.push(("field0".try_into().unwrap(), 2u8.to_be_bytes().try_into().unwrap())).unwrap();
    record.push(("id".try_into().unwrap(), 1u32.to_be_bytes().try_into().unwrap())).unwrap();
    record.push(("field1".try_into().unwrap(), 1u8.to_be_bytes().try_into().unwrap())).unwrap();
    record.push(("field2".try_into().unwrap(), 2u8.to_be_bytes().try_into().unwrap())).unwrap();
    records.push(record).unwrap();
  }
  {
    let mut record = VectorU8::new();
    record.push(("id".try_into().unwrap(), 1u32.to_be_bytes().try_into().unwrap())).unwrap();
    record.push(("field0".try_into().unwrap(), 2u8.to_be_bytes().try_into().unwrap())).unwrap();
    record.push(("id".try_into().unwrap(), 2u32.to_be_bytes().try_into().unwrap())).unwrap();
    record.push(("field1".try_into().unwrap(), 3u8.to_be_bytes().try_into().unwrap())).unwrap();
    record.push(("field2".try_into().unwrap(), 4u8.to_be_bytes().try_into().unwrap())).unwrap();
    records.push(record).unwrap();
  }
  let mut client = MockClient::new(records);
  let records = client.execute_stmt_many("", (), |_| Ok(())).await.unwrap();
  assert_eq!(
    Group::single(&records).unwrap(),
    Group {
      id: 1,
      field0: 2,
      field1: VectorU8::from_iterator([
        Single { id: 1, field1: 1, field2: 2 },
        Single { id: 2, field1: 3, field2: 4 },
      ])
      .unwrap()
    }
  );
  assert_eq!(
    Group::many(&records).collect::<Result<Vec<_>, wtx::Error>>().unwrap(),
    vec![Group {
      id: 1,
      field0: 2,
      field1: VectorU8::from_iterator([
        Single { id: 1, field1: 1, field2: 2 },
        Single { id: 2, field1: 3, field2: 4 },
      ])
      .unwrap()
    }]
  );
}

// 1 2 | 1 1 2
// 1
#[wtx::test]
async fn from_records_no_progress_error() -> wtx::Result<()> {
  #[derive(Debug, PartialEq, wtx::FromRecords)]
  #[from_records(MockDb)]
  struct Group {
    #[from_records(id)]
    id: u32,

    field0: u8,
    #[from_records(many)]
    field1: VectorU8<Single>,
  }

  #[derive(Debug, PartialEq, wtx::FromRecords)]
  #[from_records(MockDb)]
  struct Single {
    #[from_records(id)]
    id: u32,

    field1: u8,
    field2: u8,
  }

  let mut records = VectorU8::new();
  {
    let mut record = VectorU8::new();
    record.push(("id".try_into().unwrap(), 1u32.to_be_bytes().try_into().unwrap())).unwrap();
    record.push(("field0".try_into().unwrap(), 2u8.to_be_bytes().try_into().unwrap())).unwrap();
    record.push(("id".try_into().unwrap(), 1u32.to_be_bytes().try_into().unwrap())).unwrap();
    record.push(("field1".try_into().unwrap(), 1u8.to_be_bytes().try_into().unwrap())).unwrap();
    record.push(("field2".try_into().unwrap(), 2u8.to_be_bytes().try_into().unwrap())).unwrap();
    records.push(record).unwrap();
  }
  {
    let mut record = VectorU8::new();
    record.push(("id".try_into().unwrap(), 1u8.to_be_bytes().try_into().unwrap())).unwrap();
    records.push(record).unwrap();
  }
  let mut client = MockClient::new(records);
  let records = client.execute_stmt_many("", (), |_| Ok(())).await?;
  assert_eq!(
    Group::single(&records)?,
    Group {
      id: 1,
      field0: 2,
      field1: VectorU8::from_iterator([Single { id: 1, field1: 1, field2: 2 }]).unwrap()
    }
  );
  assert!(matches!(
    Group::many(&records).collect::<Result<Vec<_>, wtx::Error>>(),
    Err(wtx::Error::TryFromSlice(_))
  ));
  Ok(())
}
