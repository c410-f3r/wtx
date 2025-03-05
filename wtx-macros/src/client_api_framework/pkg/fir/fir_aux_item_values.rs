use crate::{
  client_api_framework::{
    item_with_attr_span::ItemWithAttrSpan,
    pkg::{fir::fir_aux_field_attr::FirAuxFieldAttr, misc::take_unique_pkg_attr},
  },
  misc::parts_from_generics,
};
use proc_macro2::Span;
use syn::{
  GenericParam, ImplItem, ImplItemMethod, Item, Token, Type, Visibility, WherePredicate,
  punctuated::Punctuated,
};

#[derive(Debug)]
pub(crate) struct FirAuxItemValues<'module> {
  pub(crate) faiv_user_data_method: Option<&'module ImplItemMethod>,
  pub(crate) faiv_user_params_method: Option<&'module ImplItemMethod>,
  pub(crate) faiv_params: &'module Punctuated<GenericParam, Token![,]>,
  pub(crate) faiv_ty: &'module Type,
  pub(crate) faiv_where_predicates: &'module Punctuated<WherePredicate, Token![,]>,
}

impl FirAuxItemValues<'_> {
  fn manage_impl_item(
    attr_span: Span,
    ii: &mut ImplItem,
  ) -> crate::Result<(&ImplItemMethod, FirAuxFieldAttr)> {
    let err = || Err(crate::Error::BadAux(attr_span));
    let ImplItem::Method(ref mut iim) = *ii else {
      return err();
    };
    let Some(fafa) = take_unique_pkg_attr::<FirAuxFieldAttr>(&mut iim.attrs)? else {
      return err();
    };
    if iim.vis != Visibility::Inherited {
      return err();
    }
    Ok((iim, fafa))
  }
}

impl<'module> TryFrom<ItemWithAttrSpan<(), &'module mut Item>> for FirAuxItemValues<'module> {
  type Error = crate::Error;

  fn try_from(from: ItemWithAttrSpan<(), &'module mut Item>) -> Result<Self, Self::Error> {
    let Item::Impl(item_impl) = from.item else { return Err(crate::Error::BadAux(from.span)) };

    let (faiv_params, faiv_where_predicates) = parts_from_generics(&item_impl.generics);

    let faiv_ty = &item_impl.self_ty;
    let items = &mut item_impl.items.iter_mut();
    let mut faiv_user_data_method = None;
    let mut faiv_user_params_method = None;

    if items.len() > 2 {
      return Err(crate::Error::BadAux(from.span));
    }

    if let Some(elem) = items.next() {
      let (iim, fafa) = Self::manage_impl_item(from.span, elem)?;
      match fafa {
        FirAuxFieldAttr::AuxData => faiv_user_data_method = Some(iim),
        FirAuxFieldAttr::AuxParams => faiv_user_params_method = Some(iim),
      }
    }

    if let Some(elem) = items.next() {
      let (iim, fafa) = Self::manage_impl_item(from.span, elem)?;
      match fafa {
        FirAuxFieldAttr::AuxData => match (faiv_user_data_method, faiv_user_params_method) {
          (None, None | Some(_)) => faiv_user_data_method = Some(iim),
          (Some(_), None | Some(_)) => return Err(crate::Error::BadAux(from.span)),
        },
        FirAuxFieldAttr::AuxParams => match (faiv_user_data_method, faiv_user_params_method) {
          (None | Some(_), None) => faiv_user_params_method = Some(iim),
          (None | Some(_), Some(_)) => return Err(crate::Error::BadAux(from.span)),
        },
      }
    }

    Ok(Self {
      faiv_user_data_method,
      faiv_user_params_method,
      faiv_params,
      faiv_ty,
      faiv_where_predicates,
    })
  }
}
