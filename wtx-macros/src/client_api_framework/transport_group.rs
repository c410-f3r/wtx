use crate::misc::single_elem;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens as _;
use syn::{punctuated::Punctuated, Meta, NestedMeta, Path, PathSegment, Token};

#[derive(Debug)]
pub(crate) enum TransportGroup {
  Custom(TokenStream),
  Http,
  Stub,
  Tcp,
  Udp,
  WebSocket,
}

impl<'attrs> TryFrom<&'attrs NestedMeta> for TransportGroup {
  type Error = crate::Error;

  fn try_from(from: &'attrs NestedMeta) -> Result<Self, Self::Error> {
    let err = |span| Err(crate::Error::UnknownTransport(span));
    match from {
      NestedMeta::Meta(Meta::Path(path)) => {
        let ps = single_path_segment(path)?;
        Ok(match ps.ident.to_string().as_str() {
          "http" => Self::Http,
          "stub" => Self::Stub,
          "tcp" => Self::Tcp,
          "udp" => Self::Udp,
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
