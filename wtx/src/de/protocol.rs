//! Different request and response formats.
//!
//! #### Noteworthy
//!
//! The `GraphQL` structure only contains the data expected for requests and responses,
//! which means that the elaboration of queries or other elements should be handled elsewhere. For
//! example, you can write your own operations or rely on third-parties dependencies.

mod graph_ql;
mod json_rpc;
mod misc;
mod verbatim;

pub use graph_ql::*;
pub use json_rpc::*;
pub use verbatim::*;
