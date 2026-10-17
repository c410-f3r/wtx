//! Database clients

#[cfg(all(feature = "postgres", feature = "_integration-tests", test))]
mod integration_tests;
pub mod mock_db;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "postgres")]
mod rdbms;
