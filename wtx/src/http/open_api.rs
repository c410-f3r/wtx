//! `OpenAPI` 3 structures
//!
//! <https://learn.openapis.org>

mod handler;

use crate::collections::{ArrayStrU8, ShortBoxSliceU8, ShortBoxStrU8};
use alloc::collections::BTreeMap;
use core::any::type_name;
pub use handler::{
  OpenApiHandler, OpenApiHandlerParameterMeta, OpenApiHandlerRequestBodyMeta,
  OpenApiHandlerResponseMeta, OpenApiHandlerSchemaMeta,
};

/// `OpenAPI` 3 document.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OpenApiDocument {
  /// See [`OpenApiInfo`].
  pub info: OpenApiInfo,
  /// Version.
  pub openapi: &'static str,
  /// See [`OpenApiPath`].
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "BTreeMap::is_empty"))]
  pub paths: BTreeMap<ShortBoxStrU8, OpenApiPath>,
}

impl OpenApiDocument {
  /// All other structures get default values
  #[inline]
  pub const fn from_info(info: OpenApiInfo) -> Self {
    Self { openapi: "3.0.3", info, paths: BTreeMap::new() }
  }
}

/// General API information.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OpenApiInfo {
  /// Title.
  pub title: ShortBoxStrU8,
  /// Version.
  pub version: ShortBoxStrU8,
}

impl OpenApiInfo {
  /// Shortcut
  #[inline]
  pub const fn new(title: ShortBoxStrU8, version: ShortBoxStrU8) -> Self {
    Self { title, version }
  }
}

/// Media type content.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OpenApiMediaType(i32);

/// API operation.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct OpenApiOperation {
  /// Long description.
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
  pub description: Option<ShortBoxStrU8>,
  /// Unique operation identifier.
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
  pub operation_id: Option<ShortBoxStrU8>,
  /// Parameters.
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "ShortBoxSliceU8::is_empty"))]
  pub parameters: ShortBoxSliceU8<OpenApiParameter>,
  /// Request body.
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
  pub request_body: Option<OpenApiRequestBody>,
  /// Responses.
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "BTreeMap::is_empty"))]
  pub responses: BTreeMap<ShortBoxStrU8, OpenApiResponse>,
  /// Short summary.
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
  pub summary: Option<ShortBoxStrU8>,
  /// Tags.
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "ShortBoxSliceU8::is_empty"))]
  pub tags: ShortBoxSliceU8<ShortBoxStrU8>,
}

/// API parameter.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OpenApiParameter {
  /// Parameter description.
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
  pub description: Option<ShortBoxStrU8>,
  /// Parameter location: `path`, `query`, `header` or `cookie`.
  #[cfg_attr(feature = "serde", serde(rename = "in"))]
  pub location: ShortBoxStrU8,
  /// Parameter name.
  pub name: ShortBoxStrU8,
  /// Whether the parameter is required.
  pub required: bool,
}

/// Path
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct OpenApiPath {
  /// Methods mapped to operations.
  pub operations: BTreeMap<ShortBoxStrU8, OpenApiOperation>,
}

/// Request body.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OpenApiRequestBody {
  /// Whether the body is required.
  pub required: bool,
  /// Media types mapped to their schemas.
  pub content: BTreeMap<ShortBoxStrU8, OpenApiMediaType>,
}

/// Response.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OpenApiResponse {
  /// Response description.
  pub description: ShortBoxStrU8,
  /// Media types mapped to their schemas.
  #[cfg_attr(feature = "serde", serde(skip_serializing_if = "BTreeMap::is_empty"))]
  pub content: BTreeMap<ShortBoxStrU8, OpenApiMediaType>,
}

/// Schema definition
#[derive(Debug)]
pub struct OpenApiSchemaDefinition {
  /// Format
  pub format: Option<ArrayStrU8<15>>,
  /// Type
  pub ty: ArrayStrU8<15>,
}

/// A type that can be emitted as a reusable `OpenAPI` schema.
pub trait OpenApiSchema {
  /// See [`OpenApiSchemaDefinition`].
  fn definition() -> OpenApiSchemaDefinition;

  /// Component name.
  #[inline]
  fn name() -> &'static str {
    type_name::<Self>()
  }
}

macro_rules! impl_open_api_schema {
  ($($ty:ty => $schema:expr),* $(,)?) => {
    $(
      impl OpenApiSchema for $ty {
        #[inline]
        fn definition() -> OpenApiSchemaDefinition {
          const { $schema }
        }
      }
    )*
  };
}

impl_open_api_schema! {
  bool => OpenApiSchemaDefinition {
    format: None,
    ty: ArrayStrU8::from_str_u8_opt("boolean").unwrap(),
  },
  u8 => OpenApiSchemaDefinition {
    format: Some(ArrayStrU8::from_str_u8_opt("uint8").unwrap()),
    ty: ArrayStrU8::from_str_u8_opt("integer").unwrap(),
  },
  u16 => OpenApiSchemaDefinition {
    format: Some(ArrayStrU8::from_str_u8_opt("uint16").unwrap()),
    ty: ArrayStrU8::from_str_u8_opt("integer").unwrap(),
  },
  u32 => OpenApiSchemaDefinition {
    format: Some(ArrayStrU8::from_str_u8_opt("uint32").unwrap()),
    ty: ArrayStrU8::from_str_u8_opt("integer").unwrap(),
  },
  u64 => OpenApiSchemaDefinition {
    format: Some(ArrayStrU8::from_str_u8_opt("uint64").unwrap()),
    ty: ArrayStrU8::from_str_u8_opt("integer").unwrap(),
  },
  u128 => OpenApiSchemaDefinition {
    format: Some(ArrayStrU8::from_str_u8_opt("uint128").unwrap()),
    ty: ArrayStrU8::from_str_u8_opt("integer").unwrap(),
  },
  usize => OpenApiSchemaDefinition {
    format: Some(ArrayStrU8::from_str_u8_opt("uint64").unwrap()),
    ty: ArrayStrU8::from_str_u8_opt("integer").unwrap(),
  },
  i8 => OpenApiSchemaDefinition {
    format: Some(ArrayStrU8::from_str_u8_opt("int8").unwrap()),
    ty: ArrayStrU8::from_str_u8_opt("integer").unwrap(),
  },
  i16 => OpenApiSchemaDefinition {
    format: Some(ArrayStrU8::from_str_u8_opt("int16").unwrap()),
    ty: ArrayStrU8::from_str_u8_opt("integer").unwrap(),
  },
  i32 => OpenApiSchemaDefinition {
    format: Some(ArrayStrU8::from_str_u8_opt("int32").unwrap()),
    ty: ArrayStrU8::from_str_u8_opt("integer").unwrap(),
  },
  i64 => OpenApiSchemaDefinition {
    format: Some(ArrayStrU8::from_str_u8_opt("int64").unwrap()),
    ty: ArrayStrU8::from_str_u8_opt("integer").unwrap(),
  },
  i128 => OpenApiSchemaDefinition {
    format: Some(ArrayStrU8::from_str_u8_opt("int128").unwrap()),
    ty: ArrayStrU8::from_str_u8_opt("integer").unwrap(),
  },
  isize => OpenApiSchemaDefinition {
    format: Some(ArrayStrU8::from_str_u8_opt("int64").unwrap()),
    ty: ArrayStrU8::from_str_u8_opt("integer").unwrap(),
  },
  f32 => OpenApiSchemaDefinition {
    format: Some(ArrayStrU8::from_str_u8_opt("float").unwrap()),
    ty: ArrayStrU8::from_str_u8_opt("number").unwrap(),
  },
  f64 => OpenApiSchemaDefinition {
    format: Some(ArrayStrU8::from_str_u8_opt("double").unwrap()),
    ty: ArrayStrU8::from_str_u8_opt("number").unwrap(),
  },
  ShortBoxStrU8 => OpenApiSchemaDefinition {
    format: None,
    ty: ArrayStrU8::from_str_u8_opt("string").unwrap(),
  },
  str => OpenApiSchemaDefinition {
    format: None,
    ty: ArrayStrU8::from_str_u8_opt("string").unwrap(),
  },
  () => OpenApiSchemaDefinition {
    format: None,
    ty: ArrayStrU8::from_str_u8_opt("null").unwrap(),
  },
}
