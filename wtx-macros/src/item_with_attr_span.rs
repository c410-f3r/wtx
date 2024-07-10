use core::mem;
use proc_macro2::{Span, TokenStream};
use syn::Item;

#[derive(Debug)]
pub(crate) struct ItemWithAttrSpan<C, I> {
  pub(crate) _content: C,
  pub(crate) item: I,
  pub(crate) span: Span,
}

impl<'module, C> From<(C, &'module Item, Span)> for ItemWithAttrSpan<C, &'module Item> {
  fn from((content, item, span): (C, &'module Item, Span)) -> Self {
    Self { _content: content, item, span }
  }
}

impl<'module, C> From<(C, &'module mut Item, Span)> for ItemWithAttrSpan<C, &'module mut Item> {
  fn from((content, item, span): (C, &'module mut Item, Span)) -> Self {
    Self { _content: content, item, span }
  }
}

impl<'module, C> From<(C, &'module mut Item, Span)> for ItemWithAttrSpan<C, Item> {
  fn from((content, item, span): (C, &'module mut Item, Span)) -> Self {
    let mut actual_item = Item::Verbatim(TokenStream::new());
    mem::swap(item, &mut actual_item);
    Self { _content: content, item: actual_item, span }
  }
}
