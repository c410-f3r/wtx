use alloc::{boxed::Box, string::String, vec::Vec};

/// Segment of a `GraphQL` document.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug)]
pub enum GraphQlPathSegment {
  /// Represents a named field.
  Field(Box<str>),
  /// Represents an index offset.
  Index(i32),
}

/// Line and column of a `GraphQL` document.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug)]
pub struct GraphQlLocation {
  /// Document column
  pub column: i32,
  /// Document line
  pub line: i32,
}

/// Describes an unsuccessful request.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct GraphQlResponseError<E> {
  /// Any user custom value.
  pub extensions: Option<E>,
  /// List of columns and lines
  pub locations: Option<Vec<GraphQlLocation>>,
  /// Error describer
  pub message: String,
  /// Full path to the result field where the error was raised.
  pub path: Option<Vec<GraphQlPathSegment>>,
}
