//! Database clients

#[cfg(all(any(feature = "mysql", feature = "postgres"), feature = "_integration-tests", test))]
mod integration_tests;
#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(any(feature = "mysql", feature = "postgres"))]
mod rdbms;
