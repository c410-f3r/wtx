use crate::{
  database::{
    DatabaseError, Executor as _, Record, Records as _, Typed,
    client::postgres::{
      Config, DecodeWrapper, EncodeWrapper, Postgres, PostgresExecutor, PostgresExecutorBuffer,
      StructDecoder, StructEncoder, Ty,
    },
  },
  de::{Decode, Encode},
  executor::Runtime,
  misc::UriRef,
  rng::{ChaCha20, SeedableRng},
  tests::_32_bytes_seed,
};
use alloc::string::String;
use std::{env, sync::LazyLock};

static URI: LazyLock<String> = LazyLock::new(|| env::var("DATABASE_URI_POSTGRES").unwrap());

#[test]
fn custom_composite_type() {
  Runtime::new()
    .block_on(async {
      #[derive(Debug, PartialEq)]
      struct CustomCompositeType(u32, Option<String>, u64);

      impl Decode<'_, Postgres<crate::Error>> for CustomCompositeType {
        #[inline]
        fn decode(_: &mut (), dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, crate::Error> {
          let mut sd = StructDecoder::<crate::Error>::new(dw);
          Ok(Self(sd.decode()?, sd.decode_opt()?, sd.decode()?))
        }
      }

      impl Encode<Postgres<crate::Error>> for CustomCompositeType {
        #[inline]
        fn encode(&self, _: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), crate::Error> {
          let _ev = StructEncoder::<crate::Error>::new(ew)?
            .encode(self.0)?
            .encode_with_ty(&self.1, Ty::Varchar)?
            .encode(&self.2)?;
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

      let mut exec = executor::<crate::Error>().await;
      exec
        .execute(
          "
            DROP TYPE IF EXISTS custom_composite_type CASCADE;
            DROP TABLE IF EXISTS custom_composite_table;
            CREATE TYPE custom_composite_type AS (int_value INT, varchar_value VARCHAR, bigint_value BIGINT);
            CREATE TABLE custom_composite_table (id INT, type custom_composite_type);
          ",
          |_| Ok(()),
        )
        .await
        .unwrap();
      let _ = exec
        .execute_with_stmt(
          "INSERT INTO custom_composite_table VALUES ($1, $2::custom_composite_type)",
          (1, CustomCompositeType(2, None, 4)),
        )
        .await
        .unwrap();
      let record = exec.fetch_with_stmt("SELECT * FROM custom_composite_table", ()).await.unwrap();
      assert_eq!(record.decode::<_, i32>(0).unwrap(), 1);
      assert_eq!(
        record.decode::<_, CustomCompositeType>(1).unwrap(),
        CustomCompositeType(2, None, 4)
      );
    })
    .unwrap();
}

#[test]
fn custom_domain() {
  Runtime::new()
    .block_on(async {
      #[derive(Debug, PartialEq)]
      struct CustomDomain(String);

      impl Decode<'_, Postgres<crate::Error>> for CustomDomain {
        #[inline]
        fn decode(aux: &mut (), dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, crate::Error> {
          Ok(Self(<_ as Decode<Postgres<crate::Error>>>::decode(aux, dw)?))
        }
      }

      impl Encode<Postgres<crate::Error>> for CustomDomain {
        #[inline]
        fn encode(&self, aux: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), crate::Error> {
          <_ as Encode<Postgres<crate::Error>>>::encode(&self.0, aux, ew)?;
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

      let mut exec = executor::<crate::Error>().await;
      exec
        .execute(
          "
            DROP TYPE IF EXISTS custom_domain CASCADE;
            DROP TABLE IF EXISTS custom_domain_table;
            CREATE DOMAIN custom_domain AS VARCHAR(64);
            CREATE TABLE custom_domain_table (id INT, domain custom_domain);
          ",
          |_| Ok(()),
        )
        .await
        .unwrap();
      let _ = exec
        .execute_with_stmt(
          "INSERT INTO custom_domain_table VALUES ($1, $2)",
          (1, CustomDomain(String::from("23"))),
        )
        .await
        .unwrap();
      let record = exec.fetch_with_stmt("SELECT * FROM custom_domain_table;", ()).await.unwrap();
      assert_eq!(record.decode::<_, i32>(0).unwrap(), 1);
      assert_eq!(record.decode::<_, CustomDomain>(1).unwrap(), CustomDomain(String::from("23")));
    })
    .unwrap();
}

#[test]
fn custom_enum() {
  Runtime::new()
    .block_on(async {
      enum Enum {
        Foo,
        Bar,
        Baz,
      }

      impl Decode<'_, Postgres<crate::Error>> for Enum {
        #[inline]
        fn decode(aux: &mut (), dw: &mut DecodeWrapper<'_, '_>) -> Result<Self, crate::Error> {
          let s = <&str as Decode<Postgres<crate::Error>>>::decode(aux, dw)?;
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
        fn encode(&self, aux: &mut (), ew: &mut EncodeWrapper<'_, '_>) -> Result<(), crate::Error> {
          let s = match self {
            Enum::Foo => "foo",
            Enum::Bar => "bar",
            Enum::Baz => "baz",
          };
          <_ as Encode<Postgres<crate::Error>>>::encode(&s, aux, ew)?;
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

      let mut exec = executor::<crate::Error>().await;
      exec
        .execute(
          "
          DROP TYPE IF EXISTS custom_enum CASCADE;
          DROP TABLE IF EXISTS custom_enum_table;
          CREATE TYPE custom_enum AS ENUM ('foo', 'bar', 'baz');
          CREATE TABLE custom_enum_table (id INT, domain custom_enum);
        ",
          |_| Ok(()),
        )
        .await
        .unwrap();
      let _ = exec
        .execute_with_stmt("INSERT INTO custom_enum_table VALUES ($1, $2)", (1, Enum::Bar))
        .await
        .unwrap();
      let record = exec.fetch_with_stmt("SELECT * FROM custom_enum_table;", ()).await.unwrap();
      assert_eq!(record.decode::<_, i32>(0).unwrap(), 1);
      assert!(matches!(record.decode(1).unwrap(), Enum::Bar));
    })
    .unwrap();
}

#[test]
fn custom_error() {
  Runtime::new()
    .block_on(async {
      #[derive(Debug)]
      enum CustomError {
        Wtx { _err: crate::Error },
      }

      impl From<crate::Error> for CustomError {
        #[inline]
        fn from(from: crate::Error) -> Self {
          Self::Wtx { _err: from }
        }
      }

      let mut exec = executor::<CustomError>().await;
      let _record = exec.fetch_with_stmt("SELECT 1 WHERE 0=$1", (0,)).await.unwrap();
      let _record = exec.fetch_with_stmt("SELECT 1 WHERE 0=0", ()).await.unwrap();
    })
    .unwrap();
}

#[test]
fn execute() {
  Runtime::new()
    .block_on(async {
      let mut exec = executor::<crate::Error>().await;

      assert_eq!(exec.execute_with_stmt("", ()).await.unwrap(), 0);
      assert_eq!(
        exec
          .execute_with_stmt("CREATE TABLE IF NOT EXISTS execute_test(id INT)", ())
          .await
          .unwrap(),
        0
      );
      assert_eq!(
        exec.execute_with_stmt("INSERT INTO execute_test VALUES (1)", ()).await.unwrap(),
        1
      );
      assert_eq!(
        exec.execute_with_stmt("INSERT INTO execute_test VALUES (1), (1)", ()).await.unwrap(),
        2
      );
      assert_eq!(exec.execute_with_stmt("DROP TABLE execute_test", ()).await.unwrap(), 0);
    })
    .unwrap();
}

#[test]
fn multiple_notifications() {
  Runtime::new()
    .block_on(async {
      let mut exec = executor::<crate::Error>().await;
      let _ = exec
        .execute_with_stmt(
          "CREATE TABLE IF NOT EXISTS multiple_notifications_test (id SERIAL PRIMARY KEY, body TEXT)",
          (),
        )
        .await
        .unwrap();
      let _ =
        exec.execute_with_stmt("TRUNCATE TABLE multiple_notifications_test CASCADE", ()).await.unwrap();
    })
    .unwrap();
}

#[test]
fn record() {
  Runtime::new()
    .block_on(async {
      let mut exec = executor::<crate::Error>().await;

      let _0c_0p = exec.fetch_with_stmt("", ()).await;
      assert!(
        matches!(_0c_0p.unwrap_err(), crate::Error::DatabaseError(err) if *err == DatabaseError::MissingRecord)
      );
      let _0c_1p = exec.fetch_with_stmt("SELECT 1 WHERE 0=$1", (1,)).await;
      assert!(
        matches!(_0c_1p.unwrap_err(), crate::Error::DatabaseError(err) if *err == DatabaseError::MissingRecord)
      );
      let _0c_2p = exec.fetch_with_stmt("SELECT 1 WHERE 0=$1 AND 1=$2", (1, 2)).await;
      assert!(
        matches!(_0c_2p.unwrap_err(), crate::Error::DatabaseError(err) if *err == DatabaseError::MissingRecord)
      );

      let _1c_0p = exec.fetch_with_stmt("SELECT 1", ()).await.unwrap();
      assert_eq!(_1c_0p.len(), 1);
      assert_eq!(_1c_0p.decode::<_, u32>(0).unwrap(), 1);
      let _1c_1p = exec.fetch_with_stmt("SELECT 1 WHERE 0=$1", (0,)).await.unwrap();
      assert_eq!(_1c_1p.len(), 1);
      assert_eq!(_1c_1p.decode::<_, u32>(0).unwrap(), 1);
      let _1c_2p = exec.fetch_with_stmt("SELECT 1 WHERE 0=$1 AND 1=$2", (0, 1)).await.unwrap();
      assert_eq!(_1c_2p.len(), 1);
      assert_eq!(_1c_2p.decode::<_, u32>(0).unwrap(), 1);

      let _2c_0p = exec.fetch_with_stmt("SELECT 1,2", ()).await.unwrap();
      assert_eq!(_2c_0p.len(), 2);
      assert_eq!(_2c_0p.decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_2c_0p.decode::<_, u32>(1).unwrap(), 2);
      let _2c_1p = exec.fetch_with_stmt("SELECT 1,2 WHERE 0=$1", (0,)).await.unwrap();
      assert_eq!(_2c_1p.len(), 2);
      assert_eq!(_2c_1p.decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_2c_1p.decode::<_, u32>(1).unwrap(), 2);
      let _2c_2p = exec.fetch_with_stmt("SELECT 1,2 WHERE 0=$1 AND 1=$2", (0, 1)).await.unwrap();
      assert_eq!(_2c_2p.len(), 2);
      assert_eq!(_2c_2p.decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_2c_2p.decode::<_, u32>(1).unwrap(), 2);
    })
    .unwrap();
}

#[test]
fn records() {
  Runtime::new()
    .block_on(async {
      let mut exec = executor::<crate::Error>().await;

      // 0 rows, 0 columns

      let _0r_0c_0p = exec.fetch_many_with_stmt("", (), |_| Ok(())).await.unwrap();
      assert_eq!(_0r_0c_0p.len(), 0);
      let _0r_0c_1p =
        exec.fetch_many_with_stmt("SELECT 1 WHERE 0=$1", (1,), |_| Ok(())).await.unwrap();
      assert_eq!(_0r_0c_1p.len(), 0);
      let _0r_0c_2p = exec
        .fetch_many_with_stmt("SELECT 1 WHERE 0=$1 AND 1=$2", (1, 2), |_| Ok(()))
        .await
        .unwrap();
      assert_eq!(_0r_0c_2p.len(), 0);

      // 1 row,  1 column

      let _1r_1c_0p = exec.fetch_many_with_stmt("SELECT 1", (), |_| Ok(())).await.unwrap();
      assert_eq!(_1r_1c_0p.len(), 1);
      assert_eq!(_1r_1c_0p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_1r_1c_0p.get(0).unwrap().len(), 1);
      let _1r_1c_1p =
        exec.fetch_many_with_stmt("SELECT 1 WHERE 0=$1", (0,), |_| Ok(())).await.unwrap();
      assert_eq!(_1r_1c_1p.len(), 1);
      assert_eq!(_1r_1c_1p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_1r_1c_1p.get(0).unwrap().len(), 1);
      let _1r_1c_2p = exec
        .fetch_many_with_stmt("SELECT 1 WHERE 0=$1 AND 1=$2", (0, 1), |_| Ok(()))
        .await
        .unwrap();
      assert_eq!(_1r_1c_2p.len(), 1);
      assert_eq!(_1r_1c_2p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_1r_1c_2p.get(0).unwrap().len(), 1);

      // 1 row, 2 columns

      let _1r_2c_0p = exec.fetch_many_with_stmt("SELECT 1,2", (), |_| Ok(())).await.unwrap();
      assert_eq!(_1r_2c_0p.len(), 1);
      assert_eq!(_1r_2c_0p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_1r_2c_0p.get(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
      let _1r_2c_1p =
        exec.fetch_many_with_stmt("SELECT 1,2 WHERE 0=$1", (0,), |_| Ok(())).await.unwrap();
      assert_eq!(_1r_2c_1p.len(), 1);
      assert_eq!(_1r_2c_1p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_1r_2c_1p.get(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
      let _1r_2c_2p = exec
        .fetch_many_with_stmt("SELECT 1,2 WHERE 0=$1 AND 1=$2", (0, 1), |_| Ok(()))
        .await
        .unwrap();
      assert_eq!(_1r_2c_2p.len(), 1);
      assert_eq!(_1r_2c_2p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_1r_2c_2p.get(0).unwrap().decode::<_, u32>(1).unwrap(), 2);

      // 2 rows, 1 column

      let _2r_1c_0p = exec
        .fetch_many_with_stmt("SELECT * FROM (VALUES (1), (2)) AS t (foo)", (), |_| Ok(()))
        .await
        .unwrap();
      assert_eq!(_2r_1c_0p.len(), 2);
      assert_eq!(_2r_1c_0p.get(0).unwrap().len(), 1);
      assert_eq!(_2r_1c_0p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_2r_1c_0p.get(1).unwrap().len(), 1);
      assert_eq!(_2r_1c_0p.get(1).unwrap().decode::<_, u32>(0).unwrap(), 2);
      let _2r_1c_1p = exec
        .fetch_many_with_stmt("SELECT * FROM (VALUES (1), (2)) AS t (foo) WHERE 0=$1", (0,), |_| {
          Ok(())
        })
        .await
        .unwrap();
      assert_eq!(_2r_1c_1p.len(), 2);
      assert_eq!(_2r_1c_1p.get(0).unwrap().len(), 1);
      assert_eq!(_2r_1c_1p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_2r_1c_1p.get(1).unwrap().len(), 1);
      assert_eq!(_2r_1c_1p.get(1).unwrap().decode::<_, u32>(0).unwrap(), 2);
      let _2r_1c_2p = exec
        .fetch_many_with_stmt(
          "SELECT * FROM (VALUES (1), (2)) AS t (foo) WHERE 0=$1 AND 1=$2",
          (0, 1),
          |_| Ok(()),
        )
        .await
        .unwrap();
      assert_eq!(_2r_1c_2p.len(), 2);
      assert_eq!(_2r_1c_2p.get(0).unwrap().len(), 1);
      assert_eq!(_2r_1c_2p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_2r_1c_2p.get(1).unwrap().len(), 1);
      assert_eq!(_2r_1c_2p.get(1).unwrap().decode::<_, u32>(0).unwrap(), 2);

      // 2 rows, 2 columns

      let _2r_2c_0p = exec
        .fetch_many_with_stmt("SELECT * FROM (VALUES (1,2), (3,4)) AS t (foo,bar)", (), |_| Ok(()))
        .await
        .unwrap();
      assert_eq!(_2r_2c_0p.len(), 2);
      assert_eq!(_2r_2c_0p.get(0).unwrap().len(), 2);
      assert_eq!(_2r_2c_0p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_2r_2c_0p.get(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
      assert_eq!(_2r_2c_0p.get(1).unwrap().len(), 2);
      assert_eq!(_2r_2c_0p.get(1).unwrap().decode::<_, u32>(0).unwrap(), 3);
      assert_eq!(_2r_2c_0p.get(1).unwrap().decode::<_, u32>(1).unwrap(), 4);
      let _2r_2c_1p = exec
        .fetch_many_with_stmt(
          "SELECT * FROM (VALUES (1,2), (3,4)) AS t (foo,bar) WHERE 0=$1",
          (0,),
          |_| Ok(()),
        )
        .await
        .unwrap();
      assert_eq!(_2r_2c_1p.len(), 2);
      assert_eq!(_2r_2c_1p.get(0).unwrap().len(), 2);
      assert_eq!(_2r_2c_1p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_2r_2c_1p.get(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
      assert_eq!(_2r_2c_1p.get(1).unwrap().len(), 2);
      assert_eq!(_2r_2c_1p.get(1).unwrap().decode::<_, u32>(0).unwrap(), 3);
      assert_eq!(_2r_2c_1p.get(1).unwrap().decode::<_, u32>(1).unwrap(), 4);
      let _2r_2c_2p = exec
        .fetch_many_with_stmt(
          "SELECT * FROM (VALUES (1,2), (3,4)) AS t (foo,bar) WHERE 0=$1 AND 1=$2",
          (0, 1),
          |_| Ok(()),
        )
        .await
        .unwrap();
      assert_eq!(_2r_2c_2p.len(), 2);
      assert_eq!(_2r_2c_2p.get(0).unwrap().len(), 2);
      assert_eq!(_2r_2c_2p.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(_2r_2c_2p.get(0).unwrap().decode::<_, u32>(1).unwrap(), 2);
      assert_eq!(_2r_2c_2p.get(1).unwrap().len(), 2);
      assert_eq!(_2r_2c_2p.get(1).unwrap().decode::<_, u32>(0).unwrap(), 3);
      assert_eq!(_2r_2c_2p.get(1).unwrap().decode::<_, u32>(1).unwrap(), 4);
    })
    .unwrap();
}

#[test]
fn records_after_prepare() {
  Runtime::new()
    .block_on(async {
      let mut exec = executor::<crate::Error>().await;
      let _ = exec.prepare("SELECT 1").await.unwrap();
      let _record = exec.fetch_many_with_stmt("SELECT 1", (), |_| Ok(())).await.unwrap();
    })
    .unwrap();
}

#[test]
fn reuses_cached_statement() {
  Runtime::new()
    .block_on(async {
      let mut exec = executor::<crate::Error>().await;
      let _record = exec.fetch_with_stmt("SELECT 1 WHERE 0=$1", (0,)).await.unwrap();
      let _record = exec.fetch_with_stmt("SELECT 1 WHERE 0=$1", (0,)).await.unwrap();
    })
    .unwrap();
}

#[cfg(feature = "serde_json")]
#[test]
fn serde_json() {
  Runtime::new()
    .block_on(async {
      use crate::database::Json;
      let mut exec = executor::<crate::Error>().await;
      exec
        .execute("CREATE TABLE IF NOT EXISTS serde_json (col JSONB NOT NULL)", |_| Ok(()))
        .await
        .unwrap();
      let col = (1u32, 2i64);
      let _ = exec
        .execute_with_stmt("INSERT INTO serde_json VALUES ($1::jsonb)", (Json(&col),))
        .await
        .unwrap();
      let record = exec.fetch_with_stmt("SELECT * FROM serde_json", ()).await.unwrap();
      assert_eq!(record.decode::<_, Json<(u32, i64)>>(0).unwrap(), Json(col));
    })
    .unwrap();
}

#[cfg(feature = "tokio-rustls")]
#[tokio::test]
async fn tls() {
  let uri_string = &*URI;
  let uri = UriRef::new(uri_string.as_str());
  let mut rng = ChaCha20::from_seed(_32_bytes_seed()).unwrap();
  let _executor = PostgresExecutor::<crate::Error, _, _>::connect_encrypted(
    &Config::from_uri(&uri).unwrap(),
    PostgresExecutorBuffer::new(usize::MAX, &mut rng),
    &mut rng,
    tokio::net::TcpStream::connect(uri.hostname_with_implied_port()).await.unwrap(),
    |stream| async {
      Ok(
        crate::misc::TokioRustlsConnector::default()
          .push_certs(include_bytes!("../../../../../.certs/root-ca.crt"))
          .unwrap()
          .connect_without_client_auth(uri.hostname(), stream)
          .await
          .unwrap(),
      )
    },
  )
  .await
  .unwrap();
}

async fn executor<E>() -> PostgresExecutor<E, PostgresExecutorBuffer, std::net::TcpStream> {
  let uri_string = &*URI;
  let uri = UriRef::new(uri_string.as_str());
  let mut rng = ChaCha20::from_seed(_32_bytes_seed()).unwrap();
  PostgresExecutor::connect(
    &Config::from_uri(&uri).unwrap(),
    PostgresExecutorBuffer::new(usize::MAX, &mut rng),
    &mut rng,
    std::net::TcpStream::connect(uri.hostname_with_implied_port()).unwrap(),
  )
  .await
  .unwrap()
}
