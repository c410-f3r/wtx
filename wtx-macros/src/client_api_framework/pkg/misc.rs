use proc_macro2::Span;
use quote::ToTokens as _;
use syn::{
  parse::Parse, punctuated::Punctuated, spanned::Spanned as _, token::Paren,
  AngleBracketedGenericArguments, Attribute, GenericArgument, GenericParam, Generics,
  PathArguments, PathSegment, Token, Type, TypePath, TypeTuple, WherePredicate,
};

pub(crate) const EMPTY_GEN_ARGS: &Punctuated<GenericArgument, Token![,]> = &Punctuated::new();
pub(crate) const EMPTY_GEN_PARAMS: &Punctuated<GenericParam, Token![,]> = &Punctuated::new();
pub(crate) const EMPTY_PATH_SEGS: &Punctuated<PathSegment, Token![::]> = &Punctuated::new();
pub(crate) const EMPTY_WHERE_PREDS: &Punctuated<WherePredicate, Token![,]> = &Punctuated::new();

pub(crate) fn from_camel_case_to_snake_case(string: &str) -> String {
  let mut snake_case_string = String::new();
  let mut iter = string.as_bytes().windows(2);
  if let Some(&[a, b]) = iter.next() {
    match (a.is_ascii_uppercase(), b.is_ascii_uppercase()) {
      (true, true) => {
        snake_case_string.push(a.to_ascii_lowercase().into());
        snake_case_string.push(b.to_ascii_lowercase().into());
      }
      (true, false) => {
        snake_case_string.push(a.to_ascii_lowercase().into());
        snake_case_string.push(b.into());
      }
      (false, true) => {
        snake_case_string.push(a.into());
        snake_case_string.push('_');
        snake_case_string.push(b.to_ascii_lowercase().into());
      }
      (false, false) => {
        snake_case_string.push(a.into());
        snake_case_string.push(b.into());
      }
    }
  }
  while let Some(&[a, b]) = iter.next() {
    match (a.is_ascii_uppercase(), b.is_ascii_uppercase()) {
      (true, true) => {
        snake_case_string.push(b.to_ascii_lowercase().into());
      }
      (false, true) => {
        snake_case_string.push('_');
        snake_case_string.push(b.to_ascii_lowercase().into());
      }
      (true | false, false) => {
        snake_case_string.push(b.into());
      }
    }
  }
  snake_case_string
}

pub(crate) fn inner_angle_bracketed_values(
  ty: &Type,
) -> Option<(&TypePath, &PathSegment, &AngleBracketedGenericArguments)> {
  let Type::Path(type_path) = ty else {
    return None;
  };
  let last_segment_path = type_path.path.segments.last()?;
  if let PathArguments::AngleBracketed(elem) = &last_segment_path.arguments {
    Some((type_path, last_segment_path, elem))
  } else {
    None
  }
}

pub(crate) fn is_unit_type(ty_tuple: &TypeTuple) -> bool {
  ty_tuple.elems.is_empty()
}

pub(crate) fn manage_unique_attribute<T>(opt: Option<&T>, span: Span) -> crate::Result<()> {
  if opt.is_some() {
    Err(crate::Error::DuplicatedGlobalPkgAttr(span))
  } else {
    Ok(())
  }
}

pub(crate) fn parts_from_generics(
  generics: &Generics,
) -> (&Punctuated<GenericParam, Token![,]>, &Punctuated<WherePredicate, Token![,]>) {
  (&generics.params, generics.where_clause.as_ref().map_or(EMPTY_WHERE_PREDS, |el| &el.predicates))
}

pub(crate) fn split_params(
  params: &Punctuated<GenericParam, Token![,]>,
) -> (impl Iterator<Item = &GenericParam>, impl Iterator<Item = &GenericParam>) {
  let idx = params
    .iter()
    .position(|elem| matches!(*elem, GenericParam::Type(_)))
    .unwrap_or_else(|| params.len());
  (params.iter().take(idx), params.iter().skip(idx))
}

pub(crate) fn take_unique_pkg_attr<A>(attrs: &mut Vec<Attribute>) -> crate::Result<Option<A>>
where
  A: Parse,
{
  let mut iter = attrs.iter().enumerate().filter_map(|(idx, attr)| {
    attr.path.segments.first().is_some_and(|segment| segment.ident == "pkg").then_some(idx)
  });
  let Some(idx) = iter.next() else { return Ok(None) };
  let has_more_than_one = iter.next().is_some();
  let attr = attrs.remove(idx);
  if has_more_than_one {
    return Err(crate::Error::DuplicatedLocalPkgAttr(attr.span()));
  }
  Ok(Some(syn::parse2(attr.into_token_stream())?))
}

pub(crate) fn unit_type() -> Type {
  Type::Tuple(TypeTuple { paren_token: Paren(Span::mixed_site()), elems: Punctuated::new() })
}
