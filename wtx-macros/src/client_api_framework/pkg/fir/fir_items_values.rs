use crate::client_api_framework::{
  contained_attrs::ContainedAttrs as _,
  item_with_attr_span::ItemWithAttrSpan,
  pkg::{
    fir::fir_item_attr::{FirItemAttr, FirItemAttrTy},
    misc::{manage_unique_attribute, take_unique_pkg_attr},
  },
};
use proc_macro2::Span;
use syn::Item;

#[derive(Debug)]
pub(crate) struct FirItemsValues<'module> {
  pub(crate) after_sending: Option<ItemWithAttrSpan<(), &'module Item>>,
  pub(crate) aux: Option<ItemWithAttrSpan<(), &'module mut Item>>,
  pub(crate) before_sending: Option<ItemWithAttrSpan<(), &'module Item>>,
  pub(crate) params: Option<ItemWithAttrSpan<(), &'module mut Item>>,
  pub(crate) req_data: ItemWithAttrSpan<(), &'module mut Item>,
  pub(crate) res_data: ItemWithAttrSpan<(), &'module mut Item>,
}

impl<'module> TryFrom<(&'module mut Vec<Item>, Span)> for FirItemsValues<'module> {
  type Error = crate::Error;

  fn try_from((items, mod_span): (&'module mut Vec<Item>, Span)) -> Result<Self, Self::Error> {
    let mut after_sending = None;
    let mut aux = None;
    let mut before_sending = None;
    let mut req_data = None;
    let mut params = None;
    let mut res_data = None;

    for item in items {
      let Some(attr) = item.contained_attrs() else {
        continue;
      };
      let Some(FirItemAttr { span, ty }) = take_unique_pkg_attr(attr)? else {
        continue;
      };
      match ty {
        FirItemAttrTy::AfterSending => {
          manage_unique_attribute(after_sending.as_ref(), span)?;
          after_sending = Some(ItemWithAttrSpan::from(((), &*item, span)));
        }
        FirItemAttrTy::Aux => {
          manage_unique_attribute(aux.as_ref(), span)?;
          aux = Some(ItemWithAttrSpan::from(((), &mut *item, span)));
        }
        FirItemAttrTy::BeforeSending => {
          manage_unique_attribute(before_sending.as_ref(), span)?;
          before_sending = Some(ItemWithAttrSpan::from(((), &*item, span)));
        }
        FirItemAttrTy::Params => {
          manage_unique_attribute(params.as_ref(), span)?;
          params = Some(ItemWithAttrSpan::from(((), &mut *item, span)));
        }
        FirItemAttrTy::Req => {
          manage_unique_attribute(req_data.as_ref(), span)?;
          req_data = Some(ItemWithAttrSpan::from(((), &mut *item, span)));
        }
        FirItemAttrTy::Res => {
          manage_unique_attribute(res_data.as_ref(), span)?;
          res_data = Some(ItemWithAttrSpan::from(((), &mut *item, span)));
        }
      }
    }

    Ok(Self {
      after_sending,
      aux,
      before_sending,
      params,
      req_data: req_data.ok_or(crate::Error::AbsentReqOrRes(mod_span))?,
      res_data: res_data.ok_or(crate::Error::AbsentReqOrRes(mod_span))?,
    })
  }
}
