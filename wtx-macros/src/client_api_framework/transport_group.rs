use crate::misc::single_elem;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens as _;
use syn::{Meta, Path, PathSegment, Token, punctuated::Punctuated, spanned::Spanned as _};

#[derive(Debug)]
pub(crate) enum TransportGroup {
  Custom(TokenStream),
  Http,
  Stub,
  WebSocket,
}

impl<'attrs> TryFrom<&'attrs Meta> for TransportGroup {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &'attrs Meta) -> Result<Self, Self::Error> {
    let err = |span: Span| Err(crate::Error::UnknownTransport(span));
    match from {
      Meta::Path(path) => {
        let ps = single_path_segment(path)?;
        Ok(match ps.ident.to_string().as_str() {
          "http" => Self::Http,
          "stub" => Self::Stub,
          "ws" => Self::WebSocket,
          _ => return err(ps.ident.span()),
        })
      }
      Meta::List(meta_list) => {
        let ps = single_path_segment(&meta_list.path)?;
        if ps.ident == "custom" {
          let metas = meta_list.parse_args_with(Punctuated::parse_terminated)?;
          if let Meta::Path(path) = single_nested(&metas)? {
            return Ok(Self::Custom(path.to_token_stream()));
          }
        }
        err(ps.ident.span())
      }
      Meta::NameValue(mnv) => err(mnv.span()),
    }
  }
}

fn single_nested(nested: &Punctuated<Meta, Token![,]>) -> crate::Result<&Meta> {
  single_elem(nested.iter()).ok_or_else(|| crate::Error::UnknownTransport(Span::mixed_site()))
}

fn single_path_segment(path: &Path) -> crate::Result<&PathSegment> {
  single_elem(path.segments.iter())
    .ok_or_else(|| crate::Error::UnknownTransport(Span::mixed_site()))
}
