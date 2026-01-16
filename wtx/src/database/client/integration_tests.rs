use crate::{
  collection::Vector,
  database::{Database, Executor, Typed, record::Record, records::Records},
  de::{DEController, Decode, Encode, u16_string},
  executor::Runtime,
};
use alloc::format;
use core::fmt::Debug;

pub(crate) fn execute<D, E>(fut: impl Future<Output = E>)
where
  D: Database<Error = crate::Error>,
  E: Executor<Database = D>,
  for<'any> &'any str: Decode<'any, E::Database>,
{
  Runtime::new()
    .block_on(async {
      let mut executor = fut.await;
      let mut idx: u16 = 0;
      let mut records = Vector::new();
      executor
        .execute_many(
          &mut records,
          "
            SELECT 0,1,2 UNION SELECT 3,4,5;
            SELECT 6,7,8 UNION SELECT 9,10,11;
            SELECT 12,13,14 UNION SELECT 15,16,17;
          ",
          |record| {
            assert_eq!(record.decode::<_, &str>(0).unwrap(), u16_string(idx).as_str());
            idx = idx.wrapping_add(1);
            assert_eq!(record.decode::<_, &str>(1).unwrap(), u16_string(idx).as_str());
            idx = idx.wrapping_add(1);
            assert_eq!(record.decode::<_, &str>(2).unwrap(), u16_string(idx).as_str());
            idx = idx.wrapping_add(1);
            Ok(())
          },
        )
        .await
        .unwrap();
      assert_eq!(records.len(), 3);

      let records0 = records.get(0).unwrap();
      let records00 = records0.get(0).unwrap();
      let records01 = records0.get(1).unwrap();
      assert_eq!(records0.len(), 2);
      assert_eq!(records00.len(), 3);
      assert_eq!(records00.decode::<_, &str>(0).unwrap(), "0");
      assert_eq!(records00.decode::<_, &str>(1).unwrap(), "1");
      assert_eq!(records00.decode::<_, &str>(2).unwrap(), "2");
      assert_eq!(records01.len(), 3);
      assert_eq!(records01.decode::<_, &str>(0).unwrap(), "3");
      assert_eq!(records01.decode::<_, &str>(1).unwrap(), "4");
      assert_eq!(records01.decode::<_, &str>(2).unwrap(), "5");

      let records1 = records.get(1).unwrap();
      let records10 = records1.get(0).unwrap();
      let records11 = records1.get(1).unwrap();
      assert_eq!(records1.len(), 2);
      assert_eq!(records10.len(), 3);
      assert_eq!(records10.decode::<_, &str>(0).unwrap(), "6");
      assert_eq!(records10.decode::<_, &str>(1).unwrap(), "7");
      assert_eq!(records10.decode::<_, &str>(2).unwrap(), "8");
      assert_eq!(records11.len(), 3);
      assert_eq!(records11.decode::<_, &str>(0).unwrap(), "9");
      assert_eq!(records11.decode::<_, &str>(1).unwrap(), "10");
      assert_eq!(records11.decode::<_, &str>(2).unwrap(), "11");

      let records2 = records.get(2).unwrap();
      let records20 = records2.get(0).unwrap();
      let records21 = records2.get(1).unwrap();
      assert_eq!(records2.len(), 2);
      assert_eq!(records20.len(), 3);
      assert_eq!(records20.decode::<_, &str>(0).unwrap(), "12");
      assert_eq!(records20.decode::<_, &str>(1).unwrap(), "13");
      assert_eq!(records20.decode::<_, &str>(2).unwrap(), "14");
      assert_eq!(records21.len(), 3);
      assert_eq!(records21.decode::<_, &str>(0).unwrap(), "15");
      assert_eq!(records21.decode::<_, &str>(1).unwrap(), "16");
      assert_eq!(records21.decode::<_, &str>(2).unwrap(), "17");
    });
}

pub(crate) fn execute_interleaved<D, E>(fut: impl Future<Output = E>)
where
  D: Database<Error = crate::Error>,
  E: Executor<Database = D>,
  for<'any> &'any str: Decode<'any, E::Database>,
{
  Runtime::new()
    .block_on(async {
      let mut executor = fut.await;
      let mut records = Vector::new();
      executor
        .execute_many(
          &mut records,
          "
            DROP TABLE IF EXISTS foo;
            SELECT 0;
            DROP TABLE IF EXISTS bar;
            SELECT 1;
          ",
          |_| Ok(()),
        )
        .await
        .unwrap();
      assert_eq!(records.len(), 2);

      let records0 = records.get(0).unwrap();
      let records00 = records0.get(0).unwrap();
      assert_eq!(records0.len(), 1);
      assert_eq!(records00.len(), 1);
      assert_eq!(records00.decode::<_, &str>(0).unwrap(), "0");

      let records1 = records.get(1).unwrap();
      let records10 = records1.get(0).unwrap();
      assert_eq!(records1.len(), 1);
      assert_eq!(records10.len(), 1);
      assert_eq!(records10.decode::<_, &str>(0).unwrap(), "1");
    });
}

pub(crate) fn execute_stmt_inserts<D, E>(fut: impl Future<Output = E>)
where
  D: Database<Error = crate::Error>,
  E: Executor<Database = D>,
  for<'any> u32: Decode<'any, E::Database>,
{
  Runtime::new()
    .block_on(async {
      let mut executor = fut.await;
      assert_eq!(
        executor
          .execute_stmt_many("DROP TABLE IF EXISTS execute_test", (), |_| Ok(()))
          .await
          .unwrap()
          .len(),
        0
      );
      assert_eq!(
        executor
          .execute_stmt_many("CREATE TABLE IF NOT EXISTS execute_test(id INT)", (), |_| Ok(()))
          .await
          .unwrap()
          .len(),
        0
      );
      assert_eq!(
        executor
          .execute_stmt_many("INSERT INTO execute_test VALUES (1)", (), |_| Ok(()))
          .await
          .unwrap()
          .len(),
        0
      );
      assert_eq!(
        executor
          .execute_stmt_many("INSERT INTO execute_test VALUES (2), (3)", (), |_| Ok(()))
          .await
          .unwrap()
          .len(),
        0
      );
      let select = "SELECT * FROM execute_test";
      let records = executor.execute_stmt_many(select, (), |_| Ok(())).await.unwrap();
      assert_eq!(records.len(), 3);
      assert_eq!(records.get(0).unwrap().len(), 1);
      assert_eq!(records.get(0).unwrap().decode::<_, u32>(0).unwrap(), 1);
      assert_eq!(records.get(1).unwrap().len(), 1);
      assert_eq!(records.get(1).unwrap().decode::<_, u32>(0).unwrap(), 2);
      assert_eq!(records.get(2).unwrap().len(), 1);
      assert_eq!(records.get(2).unwrap().decode::<_, u32>(0).unwrap(), 3);
    });
}

pub(crate) fn execute_stmt_selects<D, E>(fut: impl Future<Output = E>, ty0: &str, ty1: &str)
where
  D: Database<Error = crate::Error>,
  E: Executor<Database = D>,
  i32: Encode<E::Database>,
  i32: Typed<D>,
  for<'any> &'any str: Decode<'any, E::Database>,
  for<'any> i32: Decode<'any, E::Database>,
{
  Runtime::new()
    .block_on(async {
      let mut executor = fut.await;

      // 0 rows, 0 columns

      {
        let _0r_0c_1p = executor
          .execute_stmt_many(&format!("SELECT '1' WHERE 0={}", ty0), (1,), |_| Ok(()))
          .await
          .unwrap();
        assert_eq!(_0r_0c_1p.len(), 0);
      }
      {
        let _0r_0c_2p = executor
          .execute_stmt_many(
            &format!("SELECT '1' WHERE 0={} AND 1={}", ty0, ty1),
            (1, 2),
            |_| Ok(()),
          )
          .await
          .unwrap();
        assert_eq!(_0r_0c_2p.len(), 0);
      }

      // 1 row,  1 column

      {
        let _1r_1c_0p =
          executor.execute_stmt_many("SELECT '1'", (), |_| Ok(())).await.unwrap();
        assert_eq!(_1r_1c_0p.len(), 1);
        assert_eq!(_1r_1c_0p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
        assert_eq!(_1r_1c_0p.get(0).unwrap().len(), 1);
      }
      {
        let _1r_1c_1p = executor
          .execute_stmt_many(&format!("SELECT '1' WHERE 0={}", ty0), (0,), |_| Ok(()))
          .await
          .unwrap();
        assert_eq!(_1r_1c_1p.len(), 1);
        assert_eq!(_1r_1c_1p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
        assert_eq!(_1r_1c_1p.get(0).unwrap().len(), 1);
      }
      {
        let _1r_1c_2p = executor
          .execute_stmt_many(
            &format!("SELECT '1' WHERE 0={} AND 1={}", ty0, ty1),
            (0, 1),
            |_| Ok(()),
          )
          .await
          .unwrap();
        assert_eq!(_1r_1c_2p.len(), 1);
        assert_eq!(_1r_1c_2p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
        assert_eq!(_1r_1c_2p.get(0).unwrap().len(), 1);
      }

      // 1 row, 2 columns

      {
        let _1r_2c_0p =
          executor.execute_stmt_many("SELECT '1','2'", (), |_| Ok(())).await.unwrap();
        assert_eq!(_1r_2c_0p.len(), 1);
        assert_eq!(_1r_2c_0p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
        assert_eq!(_1r_2c_0p.get(0).unwrap().decode::<_, &str>(1).unwrap(), "2");
      }
      {
        let _1r_2c_1p = executor
          .execute_stmt_many(&format!("SELECT '1','2' WHERE 0={}", ty0), (0,), |_| Ok(()))
          .await
          .unwrap();
        assert_eq!(_1r_2c_1p.len(), 1);
        assert_eq!(_1r_2c_1p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
        assert_eq!(_1r_2c_1p.get(0).unwrap().decode::<_, &str>(1).unwrap(), "2");
      }
      {
        let _1r_2c_2p = executor
          .execute_stmt_many(
            &format!("SELECT '1','2' WHERE 0={} AND 1={}", ty0, ty1),
            (0, 1),
            |_| Ok(()),
          )
          .await
          .unwrap();
        assert_eq!(_1r_2c_2p.len(), 1);
        assert_eq!(_1r_2c_2p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
        assert_eq!(_1r_2c_2p.get(0).unwrap().decode::<_, &str>(1).unwrap(), "2");
      }

      // 2 rows, 1 column

      {
        let _2r_1c_0p = executor
          .execute_stmt_many(
            "SELECT * FROM (SELECT '1' UNION ALL SELECT '2') AS foo",
            (),
            |_| Ok(()),
          )
          .await
          .unwrap();
        assert_eq!(_2r_1c_0p.len(), 2);
        assert_eq!(_2r_1c_0p.get(0).unwrap().len(), 1);
        assert_eq!(_2r_1c_0p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
        assert_eq!(_2r_1c_0p.get(1).unwrap().len(), 1);
        assert_eq!(_2r_1c_0p.get(1).unwrap().decode::<_, &str>(0).unwrap(), "2");
      }
      {
        let _2r_1c_1p = executor
          .execute_stmt_many(
            &format!("SELECT * FROM (SELECT '1' UNION ALL SELECT '2') AS foo  WHERE 0={}", ty0),
            (0,),
            |_| Ok(()),
          )
          .await
          .unwrap();
        assert_eq!(_2r_1c_1p.len(), 2);
        assert_eq!(_2r_1c_1p.get(0).unwrap().len(), 1);
        assert_eq!(_2r_1c_1p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
        assert_eq!(_2r_1c_1p.get(1).unwrap().len(), 1);
        assert_eq!(_2r_1c_1p.get(1).unwrap().decode::<_, &str>(0).unwrap(), "2");
      }
      {
        let _2r_1c_2p = executor
          .execute_stmt_many(
            &format!(
              "SELECT * FROM (SELECT '1' AS foo UNION ALL SELECT '2') AS t (foo) WHERE 0={} AND 1={}",
              ty0, ty1
            ),
            (0, 1),
            |_| Ok(()),
          )
          .await
          .unwrap();
        assert_eq!(_2r_1c_2p.len(), 2);
        assert_eq!(_2r_1c_2p.get(0).unwrap().len(), 1);
        assert_eq!(_2r_1c_2p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
        assert_eq!(_2r_1c_2p.get(1).unwrap().len(), 1);
        assert_eq!(_2r_1c_2p.get(1).unwrap().decode::<_, &str>(0).unwrap(), "2");
      }

      // 2 rows, 2 columns

      {
        let _2r_2c_0p = executor
          .execute_stmt_many(
            "SELECT * FROM (SELECT '1','2' UNION ALL SELECT '3','4') AS t (foo,bar)",
            (),
            |_| Ok(()),
          )
          .await
          .unwrap();
        assert_eq!(_2r_2c_0p.len(), 2);
        assert_eq!(_2r_2c_0p.get(0).unwrap().len(), 2);
        assert_eq!(_2r_2c_0p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
        assert_eq!(_2r_2c_0p.get(0).unwrap().decode::<_, &str>(1).unwrap(), "2");
        assert_eq!(_2r_2c_0p.get(1).unwrap().len(), 2);
        assert_eq!(_2r_2c_0p.get(1).unwrap().decode::<_, &str>(0).unwrap(), "3");
        assert_eq!(_2r_2c_0p.get(1).unwrap().decode::<_, &str>(1).unwrap(), "4");
      }
      {
        let _2r_2c_1p = executor
          .execute_stmt_many(
            &format!(
              "SELECT * FROM (SELECT '1','2' UNION ALL SELECT '3','4') AS t (foo,bar) WHERE 0={}",
              ty0
            ),
            (0,),
            |_| Ok(()),
          )
          .await
          .unwrap();
        assert_eq!(_2r_2c_1p.len(), 2);
        assert_eq!(_2r_2c_1p.get(0).unwrap().len(), 2);
        assert_eq!(_2r_2c_1p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
        assert_eq!(_2r_2c_1p.get(0).unwrap().decode::<_, &str>(1).unwrap(), "2");
        assert_eq!(_2r_2c_1p.get(1).unwrap().len(), 2);
        assert_eq!(_2r_2c_1p.get(1).unwrap().decode::<_, &str>(0).unwrap(), "3");
        assert_eq!(_2r_2c_1p.get(1).unwrap().decode::<_, &str>(1).unwrap(), "4");
      }
      {
        let _2r_2c_2p = executor
          .execute_stmt_many(
            &format!(
              "SELECT * FROM (SELECT '1','2' UNION ALL SELECT '3','4') AS t (foo,bar) WHERE 0={} AND 1={}",
              ty0, ty1
            ),
            (0, 1),
            |_| Ok(()),
          )
          .await
          .unwrap();
        assert_eq!(_2r_2c_2p.len(), 2);
        assert_eq!(_2r_2c_2p.get(0).unwrap().len(), 2);
        assert_eq!(_2r_2c_2p.get(0).unwrap().decode::<_, &str>(0).unwrap(), "1");
        assert_eq!(_2r_2c_2p.get(0).unwrap().decode::<_, &str>(1).unwrap(), "2");
        assert_eq!(_2r_2c_2p.get(1).unwrap().len(), 2);
        assert_eq!(_2r_2c_2p.get(1).unwrap().decode::<_, &str>(0).unwrap(), "3");
        assert_eq!(_2r_2c_2p.get(1).unwrap().decode::<_, &str>(1).unwrap(), "4");
      }
    });
}

pub(crate) fn ping<E>(fut: impl Future<Output = E>)
where
  E: Executor,
  <E::Database as DEController>::Error: Debug,
{
  Runtime::new()
    .block_on(async {
      fut.await.ping().await.unwrap();
    });
}

pub(crate) fn records_after_prepare<D, E>(fut: impl Future<Output = E>)
where
  D: Database<Error = crate::Error>,
  E: Executor<Database = D>,
{
  Runtime::new()
    .block_on(async {
      let mut executor = fut.await;
      let _ = executor.prepare("SELECT 1").await.unwrap();
      let _record = executor.execute_stmt_many("SELECT 1", (), |_| Ok(())).await.unwrap();
    });
}

pub(crate) fn reuses_cached_statement<D, E>(fut: impl Future<Output = E>, ty0: &str)
where
  D: Database<Error = crate::Error>,
  E: Executor<Database = D>,
  i32: Encode<E::Database>,
  i32: Typed<D>,
{
  Runtime::new()
    .block_on(async {
      let mut executor = fut.await;
      {
        let _record =
          executor.execute_stmt_single(&format!("SELECT '1' WHERE 0={}", ty0), (0,)).await.unwrap();
      }
      {
        let _record =
          executor.execute_stmt_single(&format!("SELECT '1' WHERE 0={}", ty0), (0,)).await.unwrap();
      }
    });
}
