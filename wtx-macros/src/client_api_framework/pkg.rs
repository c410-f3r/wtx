mod data_format;
mod data_format_elems;
mod enum_struct_or_type;
mod fir;
mod keywords;
mod misc;
mod sir;

use crate::{
  client_api_framework::{
    item_with_attr_span::ItemWithAttrSpan,
    pkg::{fir::fir_after_sending_item_values::FirAfterSendingItemValues, misc::unit_type},
  },
  misc::{Args, push_doc},
};
use fir::{
  fir_aux_item_values::FirAuxItemValues,
  fir_before_sending_item_values::FirBeforeSendingItemValues, fir_items_values::FirItemsValues,
  fir_params_items_values::FirParamsItemValues, fir_pkg_attr::FirPkgAttr,
  fir_req_item_values::FirReqItemValues, fir_res_item_values::FirResItemValues,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens as _;
use sir::{sir_final_values::SirFinalValues, sir_pkg_attr::SirPkaAttr};
use syn::{
  Generics, Item, ItemMod, ItemType, Visibility,
  punctuated::Punctuated,
  token::{Eq, Pub, Semi, Type},
};

pub(crate) fn pkg(
  attrs: proc_macro::TokenStream,
  item: proc_macro::TokenStream,
) -> crate::Result<proc_macro::TokenStream> {
  let attr_args: Args = syn::parse(attrs)?;
  let mut item_mod: ItemMod = syn::parse(item)?;
  let items_stub = &mut Vec::new();
  let fiv = FirItemsValues::try_from((
    item_mod.content.as_mut().map_or(items_stub, |el| &mut el.1),
    item_mod.ident.span(),
  ))?;
  let freqdiv = FirReqItemValues::try_from(fiv.req_data)?;
  let mut camel_case_id = {
    let mut string = freqdiv.freqdiv_ident.to_string();
    if let Some(idx) = string.rfind("Req") {
      string.truncate(idx);
    }
    string
  };
  let mut params_item_unit = Item::Verbatim(TokenStream::new());
  let fpiv = if let Some(elem) = fiv.params {
    FirParamsItemValues::try_from(elem)?
  } else {
    params_item_unit = params_item_unit_fn(&mut camel_case_id);
    FirParamsItemValues::try_from(ItemWithAttrSpan {
      _content: (),
      item: &mut params_item_unit,
      span: Span::mixed_site(),
    })?
  };
  let fasiv = fiv.after_sending.map(FirAfterSendingItemValues::try_from).transpose()?;
  let fbsiv = fiv.before_sending.map(FirBeforeSendingItemValues::try_from).transpose()?;
  let faiv = fiv.aux.map(FirAuxItemValues::try_from).transpose()?;
  let fresdiv = FirResItemValues::try_from(fiv.res_data)?;
  let spa = SirPkaAttr::try_from(FirPkgAttr::try_from(&attr_args.0)?)?;
  let SirFinalValues { auxs, package, package_impls } = SirFinalValues::try_from((
    &mut camel_case_id,
    fpiv,
    freqdiv,
    fresdiv,
    spa,
    fasiv,
    faiv,
    fbsiv,
  ))?;
  if let Some(content) = item_mod.content.as_mut() {
    content.1.push(Item::Verbatim(quote::quote!(
      #params_item_unit

      #(#auxs)*
      #package
      #(#package_impls)*
    )));
  }
  Ok(item_mod.into_token_stream().into())
}

fn params_item_unit_fn(camel_case_id: &mut String) -> Item {
  Item::Type(ItemType {
    attrs: {
      let mut attrs = Vec::new();
      push_doc(
        &mut attrs,
        "Corresponding package does not expect any additional custom parameter.",
      );
      attrs
    },
    vis: Visibility::Public(Pub { span: Span::mixed_site() }),
    type_token: Type(Span::mixed_site()),
    ident: {
      let idx = camel_case_id.len();
      camel_case_id.push_str("Params");
      let ident = Ident::new(camel_case_id, Span::mixed_site());
      camel_case_id.truncate(idx);
      ident
    },
    generics: Generics {
      lt_token: None,
      params: Punctuated::new(),
      gt_token: None,
      where_clause: None,
    },
    eq_token: Eq(Span::mixed_site()),
    ty: Box::new(unit_type()),
    semi_token: Semi(Span::mixed_site()),
  })
}
