use crate::{
  codec::{Decode, Encode},
  collections::Vector,
  database::{
    DbClient as _, Record, Typed,
    client::postgres::{
      ClientBuffer, Config, Postgres, PostgresClient, PostgresDecodeWrapper, PostgresEncodeWrapper,
      StructDecoder, StructEncoder, Ty,
    },
    records::Records,
  },
  executor::StdRuntime,
  misc::UriRef,
  rng::{ChaCha20, CryptoSeedableRng},
  tests::_vars,
  tls::{TlsConfig, TlsConnectorBuilder, TlsModePlainText},
};
use alloc::string::String;
use core::ops::Range;
use std::net::TcpStream;

#[test]
fn array() {
  StdRuntime::new().block_on(async {
    let mut executor = executor().await;
    let array = [1i32, 2, 3];
    let record = executor.execute_stmt_single("SELECT $1", (&array,)).await.unwrap();
    assert_eq!(record.decode::<_, [i32; 3]>(0).unwrap(), array);
  });
}

#[test]
fn batch() {
  StdRuntime::new().block_on(async {
    let mut executor = executor().await;
    let mut idx: u32 = 0;
    let mut records = Vector::new();
    let mut batch = executor.batch();
    batch.stmt("SELECT 0,1,2 UNION SELECT 3,4,$1", (5,)).unwrap();
    batch.stmt("SELECT 6,7,8 UNION SELECT 9,10,$1", (11,)).unwrap();
    batch.stmt("SELECT 12,13,14 UNION SELECT 15,16,$1", (17,)).unwrap();
    batch
      .flush(&mut records, |record| {
        assert_eq!(record.decode::<_, u32>(0).unwrap(), idx);
        idx = idx.wrapping_add(1);
        assert_eq!(record.decode::<_, u32>(1).unwrap(), idx);
        idx = idx.wrapping_add(1);
        assert_eq!(record.decode::<_, u32>(2).unwrap(), idx);
        idx = idx.wrapping_add(1);
        Ok(())
      })
      .await
      .unwrap();
    assert_eq!(records.len(), 3);

    let records0 = records.first().unwrap();
    let records00 = records0.get(0).unwrap();
    let records01 = records0.get(1).unwrap();
    assert_eq!(records0.len(), 2);
    assert_eq!(records00.len(), 3);
    assert_eq!(records00.decode::<_, u32>(0).unwrap(), 0);
    assert_eq!(records00.decode::<_, u32>(1).unwrap(), 1);
    assert_eq!(records00.decode::<_, u32>(2).unwrap(), 2);
    assert_eq!(records01.len(), 3);
    assert_eq!(records01.decode::<_, u32>(0).unwrap(), 3);
    assert_eq!(records01.decode::<_, u32>(1).unwrap(), 4);
    assert_eq!(records01.decode::<_, u32>(2).unwrap(), 5);

    let records1 = records.get(1).unwrap();
    let records10 = records1.get(0).unwrap();
    let records11 = records1.get(1).unwrap();
    assert_eq!(records1.len(), 2);
    assert_eq!(records10.len(), 3);
    assert_eq!(records10.decode::<_, u32>(0).unwrap(), 6);
    assert_eq!(records10.decode::<_, u32>(1).unwrap(), 7);
    assert_eq!(records10.decode::<_, u32>(2).unwrap(), 8);
    assert_eq!(records11.len(), 3);
    assert_eq!(records11.decode::<_, u32>(0).unwrap(), 9);
    assert_eq!(records11.decode::<_, u32>(1).unwrap(), 10);
    assert_eq!(records11.decode::<_, u32>(2).unwrap(), 11);

    let records2 = records.get(2).unwrap();
    let records20 = records2.get(0).unwrap();
    let records21 = records2.get(1).unwrap();
    assert_eq!(records2.len(), 2);
    assert_eq!(records20.len(), 3);
    assert_eq!(records20.decode::<_, u32>(0).unwrap(), 12);
    assert_eq!(records20.decode::<_, u32>(1).unwrap(), 13);
    assert_eq!(records20.decode::<_, u32>(2).unwrap(), 14);
    assert_eq!(records21.len(), 3);
    assert_eq!(records21.decode::<_, u32>(0).unwrap(), 15);
    assert_eq!(records21.decode::<_, u32>(1).unwrap(), 16);
    assert_eq!(records21.decode::<_, u32>(2).unwrap(), 17);
  });
}

#[test]
fn bytes() {
  StdRuntime::new().block_on(async {
    let id = 1;
    let bytes = &[255, 2, 3];
    let mut executor = executor().await;
    executor.execute_none("DROP TABLE IF EXISTS bytes_test").await.unwrap();
    executor.execute_none("CREATE TABLE bytes_test(id INT, foo BYTEA[])").await.unwrap();
    executor.execute_stmt_none("INSERT INTO bytes_test VALUES($1, $2)", (id, bytes)).await.unwrap();
    let record = executor
      .execute_stmt_single("SELECT id,foo FROM bytes_test WHERE foo = $1", (bytes,))
      .await
      .unwrap();
    assert_eq!(record.decode::<_, i32>(0).unwrap(), 1);
    assert_eq!(record.decode::<_, [u8; 3]>(1).unwrap(), *bytes);
  });
}

#[test]
fn custom_composite_type() {
  StdRuntime::new()
    .block_on(async {
      #[derive(Debug, PartialEq)]
      struct CustomCompositeType(i32, Option<String>, i64);

      impl Decode<'_, Postgres<crate::Error>> for CustomCompositeType {
        #[inline]
        fn decode(dw: &mut PostgresDecodeWrapper<'_, '_>) -> crate::Result<Self> {
          let mut sd = StructDecoder::<crate::Error>::new(dw);
          Ok(Self(sd.decode()?, sd.decode_opt()?, sd.decode()?))
        }
      }

      impl Encode<Postgres<crate::Error>> for CustomCompositeType {
        #[inline]
        fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> crate::Result<()> {
          let _ev = StructEncoder::<crate::Error>::new(ew)?
            .encode(self.0)?
            .encode_with_ty(&self.1, Ty::Varchar)?
            .encode(self.2)?;
          Ok(())
        }
      }

      impl Typed<Postgres<crate::Error>> for CustomCompositeType {
        #[inline]
        fn runtime_ty(&self) -> Option<Ty> {
          None
        }

        #[inline]
        fn static_ty() -> Option<Ty> {
          None
        }
      }

      let mut executor = executor().await;
      executor
        .execute_ignored(
          "
            DROP TYPE IF EXISTS custom_composite_type CASCADE;
            DROP TABLE IF EXISTS custom_composite_table;
            CREATE TYPE custom_composite_type AS (int_value INT, varchar_value VARCHAR, bigint_value BIGINT);
            CREATE TABLE custom_composite_table (id INT, type custom_composite_type);
          ",
        )
        .await
        .unwrap();
      executor
        .execute_stmt_none(
          "INSERT INTO custom_composite_table VALUES ($1, $2::custom_composite_type)",
          (1, CustomCompositeType(2, None, 4)),
        )
        .await
        .unwrap();
      let record = executor.execute_stmt_single("SELECT * FROM custom_composite_table", ()).await.unwrap();
      assert_eq!(record.decode::<_, i32>(0).unwrap(), 1);
      assert_eq!(
        record.decode::<_, CustomCompositeType>(1).unwrap(),
        CustomCompositeType(2, None, 4)
      );
    });
}

#[test]
fn custom_domain() {
  StdRuntime::new().block_on(async {
    #[derive(Debug, PartialEq)]
    struct CustomDomain(String);

    impl Decode<'_, Postgres<crate::Error>> for CustomDomain {
      #[inline]
      fn decode(dw: &mut PostgresDecodeWrapper<'_, '_>) -> crate::Result<Self> {
        Ok(Self(<_ as Decode<Postgres<crate::Error>>>::decode(dw)?))
      }
    }

    impl Encode<Postgres<crate::Error>> for CustomDomain {
      #[inline]
      fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> crate::Result<()> {
        <_ as Encode<Postgres<crate::Error>>>::encode(&self.0, ew)?;
        Ok(())
      }
    }

    impl Typed<Postgres<crate::Error>> for CustomDomain {
      #[inline]
      fn runtime_ty(&self) -> Option<Ty> {
        None
      }

      #[inline]
      fn static_ty() -> Option<Ty> {
        None
      }
    }

    let mut executor = executor().await;
    executor
      .execute_ignored(
        "
            DROP TYPE IF EXISTS custom_domain CASCADE;
            DROP TABLE IF EXISTS custom_domain_table;
            CREATE DOMAIN custom_domain AS VARCHAR(64);
            CREATE TABLE custom_domain_table (id INT, domain custom_domain);
          ",
      )
      .await
      .unwrap();
    executor
      .execute_stmt_none(
        "INSERT INTO custom_domain_table VALUES ($1, $2)",
        (1, CustomDomain(String::from("23"))),
      )
      .await
      .unwrap();
    let record =
      executor.execute_stmt_single("SELECT * FROM custom_domain_table;", ()).await.unwrap();
    assert_eq!(record.decode::<_, i32>(0).unwrap(), 1);
    assert_eq!(record.decode::<_, CustomDomain>(1).unwrap(), CustomDomain(String::from("23")));
  });
}

#[test]
fn custom_enum() {
  StdRuntime::new().block_on(async {
    enum Enum {
      Foo,
      Bar,
      Baz,
    }

    impl Decode<'_, Postgres<crate::Error>> for Enum {
      #[inline]
      fn decode(dw: &mut PostgresDecodeWrapper<'_, '_>) -> crate::Result<Self> {
        let s = <&str as Decode<Postgres<crate::Error>>>::decode(dw)?;
        Ok(match s {
          "foo" => Self::Foo,
          "bar" => Self::Bar,
          "baz" => Self::Baz,
          _ => panic!(),
        })
      }
    }

    impl Encode<Postgres<crate::Error>> for Enum {
      #[inline]
      fn encode(&self, ew: &mut PostgresEncodeWrapper<'_>) -> crate::Result<()> {
        let s = match self {
          Enum::Foo => "foo",
          Enum::Bar => "bar",
          Enum::Baz => "baz",
        };
        <_ as Encode<Postgres<crate::Error>>>::encode(&s, ew)?;
        Ok(())
      }
    }

    impl Typed<Postgres<crate::Error>> for Enum {
      #[inline]
      fn runtime_ty(&self) -> Option<Ty> {
        None
      }

      #[inline]
      fn static_ty() -> Option<Ty> {
        None
      }
    }

    let mut executor = executor().await;
    executor
      .execute_ignored(
        "
          DROP TYPE IF EXISTS custom_enum CASCADE;
          DROP TABLE IF EXISTS custom_enum_table;
          CREATE TYPE custom_enum AS ENUM ('foo', 'bar', 'baz');
          CREATE TABLE custom_enum_table (id INT, domain custom_enum);
        ",
      )
      .await
      .unwrap();
    executor
      .execute_stmt_none("INSERT INTO custom_enum_table VALUES ($1, $2)", (1, Enum::Bar))
      .await
      .unwrap();
    let record =
      executor.execute_stmt_single("SELECT * FROM custom_enum_table;", ()).await.unwrap();
    assert_eq!(record.decode::<_, i32>(0).unwrap(), 1);
    assert!(matches!(record.decode(1).unwrap(), Enum::Bar));
  });
}

#[test]
fn execute() {
  crate::database::client::integration_tests::execute(executor());
}

#[test]
fn execute_interleaved() {
  crate::database::client::integration_tests::execute_interleaved(executor());
}

#[test]
fn execute_stmt_inserts() {
  crate::database::client::integration_tests::execute_stmt_inserts(executor());
}

#[test]
fn execute_stmt_selects() {
  crate::database::client::integration_tests::execute_stmt_selects(executor(), "$1", "$2");
}

#[test]
fn multiple_notifications() {
  StdRuntime::new().block_on(async {
    let mut executor = executor().await;
    executor
      .execute_stmt_none(
        "CREATE TABLE IF NOT EXISTS multiple_notifications_test (id SERIAL PRIMARY KEY, body TEXT)",
        (),
      )
      .await
      .unwrap();
    executor
      .execute_stmt_none("TRUNCATE TABLE multiple_notifications_test CASCADE", ())
      .await
      .unwrap();
  });
}

#[test]
fn ping() {
  crate::database::client::integration_tests::ping(executor());
}

#[test]
fn range() {
  StdRuntime::new().block_on(async {
    let range = 3..7;
    let mut executor = executor().await;
    let record = executor.execute_stmt_single("SELECT $1", (range.clone(),)).await.unwrap();
    assert_eq!(record.decode::<_, Range<i32>>(0).unwrap(), range);
  });
}

#[test]
fn records_after_prepare() {
  crate::database::client::integration_tests::records_after_prepare(executor());
}

#[test]
fn reuses_cached_statement() {
  crate::database::client::integration_tests::reuses_cached_statement(executor(), "$1");
}

#[cfg(feature = "serde_json")]
#[test]
fn serde_json() {
  StdRuntime::new().block_on(async {
    use crate::database::Json;
    let mut executor = executor().await;
    executor
      .execute_many(
        &mut (),
        "DROP TABLE IF EXISTS serde_json; CREATE TABLE serde_json (col JSONB NOT NULL)",
        |_| Ok(()),
      )
      .await
      .unwrap();
    let col = (1u32, 2i64);
    executor
      .execute_stmt_none("INSERT INTO serde_json VALUES ($1::jsonb)", (Json(&col),))
      .await
      .unwrap();
    let record = executor.execute_stmt_single("SELECT * FROM serde_json", ()).await.unwrap();
    assert_eq!(record.decode::<_, Json<(u32, i64)>>(0).unwrap(), Json(col));
  });
}

async fn executor() -> PostgresClient<crate::Error, TcpStream, TlsModePlainText> {
  let uri = UriRef::new(_vars().database_uri_postgres.as_str());
  let mut rng = ChaCha20::from_std_random().unwrap();
  let client_buffer = ClientBuffer::new(usize::MAX, &mut rng);
  PostgresClient::connect(
    client_buffer,
    &Config::from_uri(&uri).unwrap(),
    TlsConnectorBuilder::std(uri).build(TlsConfig::plaintext(), rng).await.unwrap(),
  )
  .await
  .unwrap()
}
