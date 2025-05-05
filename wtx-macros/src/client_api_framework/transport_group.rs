use crate::misc::single_elem;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens as _;
use syn::{Meta, NestedMeta, Path, PathSegment, Token, punctuated::Punctuated};

#[derive(Debug)]
pub(crate) enum TransportGroup {
  Custom(TokenStream),
  Http,
  Stub,
  WebSocket,
}

impl<'attrs> TryFrom<&'attrs NestedMeta> for TransportGroup {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &'attrs NestedMeta) -> Result<Self, Self::Error> {
    let err = |span| Err(crate::Error::UnknownTransport(span));
    match from {
      NestedMeta::Meta(Meta::Path(path)) => {
        let ps = single_path_segment(path)?;
        Ok(match ps.ident.to_string().as_str() {
          "http" => Self::Http,
          "stub" => Self::Stub,
          "ws" => Self::WebSocket,
          _ => return err(ps.ident.span()),
        })
      }
      NestedMeta::Meta(Meta::List(meta_list)) => {
        let ps = single_path_segment(&meta_list.path)?;
        if ps.ident.to_string().as_str() == "custom" {
          if let NestedMeta::Meta(Meta::Path(path)) = single_nested(&meta_list.nested)? {
            return Ok(Self::Custom(path.to_token_stream()));
          }
        }
        err(ps.ident.span())
      }
      NestedMeta::Lit(_) | NestedMeta::Meta(_) => err(Span::mixed_site()),
    }
  }
}

fn single_nested(nested: &Punctuated<NestedMeta, Token![,]>) -> crate::Result<&NestedMeta> {
  single_elem(nested.iter()).ok_or_else(|| crate::Error::UnknownTransport(Span::mixed_site()))
}

fn single_path_segment(path: &Path) -> crate::Result<&PathSegment> {
  single_elem(path.segments.iter())
    .ok_or_else(|| crate::Error::UnknownTransport(Span::mixed_site()))
}
