//! Migration TOML parser

use crate::{
  database::schema_manager::SchemaManagerError,
  misc::{ArrayString, ArrayVector, str_split1},
};
use alloc::string::String;
use std::io::{BufRead, BufReader, Read};

pub(crate) const EXPR_ARRAY_MAX_LEN: usize = 8;

pub(crate) type ExprArrayTy = ArrayVector<ExprStringTy, EXPR_ARRAY_MAX_LEN>;
pub(crate) type ExprStringTy = ArrayString<128>;
pub(crate) type IdentTy = ArrayString<64>;
pub(crate) type RootParamsTy = ArrayVector<(IdentTy, Expr), 2>;

#[expect(clippy::large_enum_variant, reason = "work in progress")]
#[derive(Debug, PartialEq)]
pub(crate) enum Expr {
  Array(ExprArrayTy),
  String(ExprStringTy),
}

#[inline]
pub(crate) fn toml<R>(read: R) -> crate::Result<RootParamsTy>
where
  R: Read,
{
  let mut br = BufReader::new(read);
  let mut is_in_array_context = None;
  let mut buffer = String::new();
  let mut root_params = ArrayVector::new();

  macro_rules! clear_and_continue {
    () => {
      buffer.clear();
      continue;
    };
  }

  loop {
    if br.read_line(&mut buffer)? == 0 {
      break;
    }

    let buffer_ref = buffer.trim();

    if buffer_ref.starts_with('#') {
      clear_and_continue!();
    }

    if let Some(ident) = is_in_array_context.as_ref() {
      if buffer_ref.ends_with(']') {
        let inner = buffer_ref.get(0..buffer_ref.len().saturating_sub(1)).unwrap_or_default();
        parse_and_push_toml_expr_array(inner, *ident, &mut root_params)?;
        is_in_array_context = None;
        buffer.clear();
      }
      continue;
    }

    let mut root_param_iter = buffer_ref.split('=');

    let ident = if let Some(el) = root_param_iter.next() {
      el.trim().try_into().map_err(|_err| SchemaManagerError::TomlValueIsTooLarge)?
    } else {
      clear_and_continue!();
    };

    let expr_raw = if let Some(el) = root_param_iter.next() {
      el.trim()
    } else {
      clear_and_continue!();
    };

    if expr_raw.starts_with('[') {
      if expr_raw.ends_with(']') {
        let inner = expr_raw.get(1..expr_raw.len().saturating_sub(1)).unwrap_or_default();
        parse_and_push_toml_expr_array(inner, ident, &mut root_params)?;
      } else {
        is_in_array_context = Some(ident);
      }
    } else {
      parse_and_push_toml_expr_string(expr_raw, ident, &mut root_params)?;
    }

    buffer.clear();
  }

  Ok(root_params)
}

#[inline]
fn parse_and_push_toml_expr_array(
  s: &str,
  ident: IdentTy,
  root_params: &mut RootParamsTy,
) -> crate::Result<()> {
  let expr_array = parse_expr_array(s)?;
  root_params
    .push((ident, Expr::Array(expr_array)))
    .map_err(|_err| SchemaManagerError::TomlValueIsTooLarge)?;
  Ok(())
}

#[inline]
fn parse_and_push_toml_expr_string(
  s: &str,
  ident: IdentTy,
  root_params: &mut RootParamsTy,
) -> crate::Result<()> {
  let expr_string = parse_expr_string(s)?;
  root_params
    .push((ident, Expr::String(expr_string)))
    .map_err(|_err| SchemaManagerError::TomlValueIsTooLarge)?;
  Ok(())
}

#[inline]
fn parse_expr_array(s: &str) -> crate::Result<ExprArrayTy> {
  let mut array = ArrayVector::new();
  if s.is_empty() {
    return Ok(array);
  }
  for elem in str_split1(s, b',') {
    let expr_string = parse_expr_string(elem.trim())?;
    array.push(expr_string).map_err(|_err| SchemaManagerError::TomlValueIsTooLarge)?;
  }
  Ok(array)
}

#[inline]
fn parse_expr_string(s: &str) -> crate::Result<ExprStringTy> {
  let mut iter = str_split1(s, b'"');
  let _ = iter.next().ok_or(SchemaManagerError::TomlParserOnlySupportsStringsAndArraysOfStrings)?;
  let value =
    iter.next().ok_or(SchemaManagerError::TomlParserOnlySupportsStringsAndArraysOfStrings)?;
  let _ = iter.next().ok_or(SchemaManagerError::TomlParserOnlySupportsStringsAndArraysOfStrings)?;
  if iter.next().is_some() {
    return Err(SchemaManagerError::TomlParserOnlySupportsStringsAndArraysOfStrings.into());
  }
  Ok(value.trim().try_into().map_err(|_err| SchemaManagerError::TomlValueIsTooLarge)?)
}

#[cfg(test)]
mod tests {
  use crate::{
    database::schema_manager::toml_parser::{Expr, ExprArrayTy, toml},
    misc::ArrayVector,
  };

  #[test]
  fn toml_parses_root_parameter_array_in_a_single_line() {
    let array = toml(
      &br#"
    foo = ["1", "2"]

    bar=[]
    "#[..],
    )
    .unwrap();
    assert_eq!(
      array[0],
      (
        "foo".try_into().unwrap(),
        Expr::Array({
          let mut elems = ArrayVector::new();
          elems.push("1".try_into().unwrap()).unwrap();
          elems.push("2".try_into().unwrap()).unwrap();
          elems
        })
      )
    );
    assert_eq!(array[1], ("bar".try_into().unwrap(), Expr::Array(ExprArrayTy::default())));
  }

  #[test]
  fn toml_parses_root_parameter_array_in_multiple_lines() {
    let array = toml(
      &br#"
    foo=[
      "1",
      "2",
      "3"
    ]
    "#[..],
    )
    .unwrap();
    assert_eq!(
      array[0],
      (
        "foo".try_into().unwrap(),
        Expr::Array({
          let mut elems = ArrayVector::new();
          elems.push("1".try_into().unwrap()).unwrap();
          elems.push("2".try_into().unwrap()).unwrap();
          elems.push("3".try_into().unwrap()).unwrap();
          elems
        })
      )
    );
  }

  #[test]
  fn toml_parses_root_parameter_string() {
    let array = toml(&br#"foo="bar""#[..]).unwrap();
    assert_eq!(array[0], ("foo".try_into().unwrap(), Expr::String("bar".try_into().unwrap())));
  }

  #[test]
  fn toml_ignores_comments() {
    let array = toml(
      &br#"
    # Foo

    foo="bar"
    "#[..],
    )
    .unwrap();
    assert_eq!(array[0], ("foo".try_into().unwrap(), Expr::String("bar".try_into().unwrap())));
  }
}
