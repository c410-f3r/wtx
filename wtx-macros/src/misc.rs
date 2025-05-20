use proc_macro2::{Ident, Span};
use syn::{
  AttrStyle, Attribute, Expr, ExprLit, GenericParam, Generics, Lit, LitStr, Meta, MetaList,
  MetaNameValue, Path, PathArguments, PathSegment, Token, WherePredicate,
  parse::{Parse, ParseStream},
  punctuated::Punctuated,
  token::{Bracket, Eq, Pound},
};

pub(crate) const EMPTY_WHERE_PREDS: &Punctuated<WherePredicate, Token![,]> = &Punctuated::new();

pub(crate) struct Args(pub(crate) Punctuated<Meta, Token![,]>);

impl Parse for Args {
  fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
    Ok(Self(Punctuated::<Meta, Token![,]>::parse_terminated(input)?))
  }
}

pub(crate) fn create_ident<'suf>(
  string: &mut String,
  suffixes: impl IntoIterator<Item = &'suf str>,
) -> Ident {
  let idx = extend_with_tmp_suffix(string, suffixes);
  let ident = Ident::new(string, Span::mixed_site());
  string.truncate(idx);
  ident
}

pub(crate) fn create_path<'any>(idents: impl IntoIterator<Item = &'any str>) -> Path {
  Path {
    leading_colon: None,
    segments: idents
      .into_iter()
      .map(|ident| PathSegment {
        ident: Ident::new(ident, Span::mixed_site()),
        arguments: PathArguments::None,
      })
      .collect(),
  }
}

pub(crate) fn extend_with_tmp_suffix<'suf>(
  string: &mut String,
  suffixes: impl IntoIterator<Item = &'suf str>,
) -> usize {
  let idx = string.len();
  for suffix in suffixes {
    string.push_str(suffix);
  }
  idx
}

pub(crate) fn fist_meta_path_from_meta_list(meta_list: &MetaList) -> Option<Path> {
  let metas = meta_list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).ok()?;
  if let Meta::Path(path) = metas.into_iter().next()? { Some(path) } else { None }
}

pub(crate) fn has_at_least_one_doc(attrs: &[Attribute]) -> bool {
  attrs_by_names(attrs, ["doc"])[0].is_some()
}

pub(crate) fn parts_from_generics(
  generics: &Generics,
) -> (&Punctuated<GenericParam, Token![,]>, &Punctuated<WherePredicate, Token![,]>) {
  (&generics.params, generics.where_clause.as_ref().map_or(EMPTY_WHERE_PREDS, |el| &el.predicates))
}

pub(crate) fn push_doc(attrs: &mut Vec<Attribute>, doc: &str) {
  push_attr(
    attrs,
    Meta::NameValue(MetaNameValue {
      path: create_path(["doc"]),
      eq_token: Eq { spans: [Span::mixed_site()] },
      value: Expr::Lit(ExprLit {
        attrs: Vec::new(),
        lit: Lit::Str(LitStr::new(doc, Span::mixed_site())),
      }),
    }),
  );
}

pub(crate) fn push_doc_if_inexistent(attrs: &mut Vec<Attribute>, doc: &str) {
  if !has_at_least_one_doc(attrs) {
    push_doc(attrs, doc);
  }
}

pub(crate) fn single_elem<T>(mut iter: impl Iterator<Item = T>) -> Option<T> {
  let first = iter.next()?;
  if iter.next().is_some() {
    return None;
  }
  Some(first)
}

fn attrs_by_names<'attrs, const N: usize>(
  attrs: &'attrs [Attribute],
  names: [&str; N],
) -> [Option<&'attrs Attribute>; N] {
  let mut rslt = [None; N];
  for attr in attrs {
    let Some(last) = attr.meta.path().segments.last() else {
      continue;
    };
    let s = last.ident.to_string();
    for (name, rslt_attr) in names.iter().zip(&mut rslt) {
      if rslt_attr.is_some() {
        continue;
      }
      if name == &s {
        *rslt_attr = Some(attr);
        break;
      }
    }
  }
  rslt
}

fn push_attr(attrs: &mut Vec<Attribute>, meta: Meta) {
  attrs.push(Attribute {
    pound_token: Pound(Span::mixed_site()),
    style: AttrStyle::Outer,
    bracket_token: Bracket(Span::mixed_site()),
    meta,
  });
}

#[cfg(test)]
mod tests {
  use crate::misc::{attrs_by_names, create_path, push_attr};

  #[test]
  fn has_names_in_attrs_has_correct_output() {
    let mut attrs = Vec::new();
    push_attr(&mut attrs, syn::Meta::Path(create_path(["foo"])));
    push_attr(&mut attrs, syn::Meta::Path(create_path(["baz"])));
    assert_eq!(
      attrs_by_names(&attrs, ["foo", "bar", "baz"]),
      [Some(&attrs[0]), None, Some(&attrs[1])]
    );
    let attrs = Vec::new();
    assert_eq!(attrs_by_names(&attrs, ["foo", "bar", "baz"]), [None, None, None]);
  }
}
