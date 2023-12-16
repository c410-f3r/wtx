use crate::{item_with_attr_span::ItemWithAttrSpan, misc::push_doc_if_inexistent};
use proc_macro2::Ident;
use syn::Item;

#[derive(Debug)]
pub(crate) struct FirResItemValues<'module> {
  pub(crate) res_ident: &'module Ident,
}

impl<'module> TryFrom<ItemWithAttrSpan<(), &'module mut Item>> for FirResItemValues<'module> {
  type Error = crate::Error;

  fn try_from(from: ItemWithAttrSpan<(), &'module mut Item>) -> Result<Self, Self::Error> {
    let (attrs, res_ident, generics) = match *from.item {
      Item::Enum(ref mut item) => (&mut item.attrs, &item.ident, &item.generics),
      Item::Struct(ref mut item) => (&mut item.attrs, &item.ident, &item.generics),
      Item::Type(ref mut item) => (&mut item.attrs, &item.ident, &item.generics),
      _ => return Err(crate::Error::NoEnumStructOrType(from.span)),
    };
    push_doc_if_inexistent(attrs, "Expected data response returned by the server.");
    if !res_ident.to_string().ends_with("Res") {
      return Err(crate::Error::BadRes(res_ident.span()));
    }
    if !generics.params.is_empty() {
      return Err(crate::Error::ResponsesCanNotHaveTypeParams(res_ident.span()));
    }
    Ok(Self { res_ident })
  }
}
