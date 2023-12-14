//! Different request and response formats.
//!
//! #### Noteworthy
//!
//! The GraphQL and Protobuf structures only contain the data expected for requests and responses,
//! which means that the elaboration of queries or other elements should be handled elsewhere. For
//! example, you can write your own operations or rely on third-parties dependencies.

mod borsh;
mod graph_ql;
mod json;
mod json_rpc;
mod protobuf;
mod verbatim;
mod xml;
mod yaml;

pub use self::{borsh::*, protobuf::*};
pub use graph_ql::*;
pub use json::*;
pub use json_rpc::*;
pub use verbatim::*;
pub use xml::*;
pub use yaml::*;
