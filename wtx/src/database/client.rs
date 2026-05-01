//! Database clients

#[cfg(all(feature = "postgres", feature = "_integration-tests", test))]
mod integration_tests;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "postgres")]
mod rdbms;
