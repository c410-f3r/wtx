use crate::client_api_framework::pkg::{
  enum_struct_or_type::EnumStructOrType, fir::fir_custom_field_field_attr::FirCustomFieldFieldAttr,
};
use syn::{punctuated::Punctuated, GenericParam, Ident, Token, Type, WherePredicate};

#[derive(Clone, Copy, Debug)]
pub(crate) struct FirCustomItemValuesRef<'any, 'module> {
  pub(crate) fields_attrs: &'any Vec<Option<FirCustomFieldFieldAttr>>,
  pub(crate) ident: &'module Ident,
  pub(crate) item: EnumStructOrType<'module>,
  pub(crate) params: &'module Punctuated<GenericParam, Token![,]>,
  pub(crate) ty: &'any Type,
  pub(crate) where_predicates: &'module Punctuated<WherePredicate, Token![,]>,
}

macro_rules! create_fir_custom_item_values {
  (
    $doc:literal,
    $struct:ident,
    $fields_attrs:ident,
    $ident:ident,
    $item:ident,
    $params:ident,
    $ty:ident,
    $where_predicates:ident,
    $($cb:expr)?,
  ) => {
    use crate::{
      client_api_framework::item_with_attr_span::ItemWithAttrSpan,
      misc::{parts_from_generics, push_doc_if_inexistent},
      client_api_framework::pkg::{
        enum_struct_or_type::EnumStructOrType,
        fir::{
          fir_custom_field_attr::FirCustomFieldAttr,
          fir_custom_field_field_attr::FirCustomFieldFieldAttr,
          fir_custom_item_values::FirCustomItemValuesRef,
        },
        misc::take_unique_pkg_attr,
      },
    };
    use proc_macro2::Ident;
    use syn::{
      punctuated::Punctuated, Field, Fields, GenericParam, Item, Type, Token, WherePredicate
    };

    #[derive(Debug)]
    pub(crate) struct $struct<'module> {
      pub(crate) $fields_attrs: Vec<Option<FirCustomFieldFieldAttr>>,
      pub(crate) $ident: &'module Ident,
      pub(crate) $item: EnumStructOrType<'module>,
      pub(crate) $params: &'module Punctuated<GenericParam, Token![,]>,
      pub(crate) $ty: Type,
      pub(crate) $where_predicates: &'module Punctuated<WherePredicate, Token![,]>,
    }

    impl<'any, 'module> From<&'any $struct<'module>> for FirCustomItemValuesRef<'any, 'module> {
      fn from(from: &'any $struct<'module>) -> Self {
        Self {
          fields_attrs: &from.$fields_attrs,
          ident: from.$ident,
          item: from.$item,
          params: from.$params,
          ty: &from.$ty,
          where_predicates: from.$where_predicates,
        }
      }
    }

    impl<'module> TryFrom<ItemWithAttrSpan<(), &'module mut Item>> for $struct<'module> {
      type Error = crate::Error;

      fn try_from(from: ItemWithAttrSpan<(), &'module mut Item>) -> Result<Self, Self::Error> {
        let local_generics;
        let local_ident;
        let local_item;
        let mut local_fields_attrs = Vec::new();
        match *from.item {
          Item::Enum(ref mut item) => {
            push_doc_if_inexistent(&mut item.attrs, $doc);
            local_generics = &item.generics;
            local_ident = &item.ident;
            local_item = EnumStructOrType::Enum;
          }
          Item::Struct(ref mut item) => {
            push_doc_if_inexistent(&mut item.attrs, $doc);
            local_generics = &item.generics;
            local_ident = &item.ident;
            match item.fields {
              Fields::Named(ref mut fields_named) => {
                manage_struct_field_attrs(&mut fields_named.named, &mut local_fields_attrs)?;
              }
              Fields::Unnamed(ref mut fields_unnamed) => {
                manage_struct_field_attrs(&mut fields_unnamed.unnamed, &mut local_fields_attrs)?;
              }
              Fields::Unit => {}
            }
            local_item = EnumStructOrType::Struct(item);
          }
          Item::Type(ref mut item) => {
            push_doc_if_inexistent(&mut item.attrs, $doc);
            local_generics = &item.generics;
            local_ident = &item.ident;
            local_item = EnumStructOrType::Type(item);
          }
          _ => return Err(crate::Error::NoEnumStructOrType(from.span)),
        };
        let (local_params, local_where_predicates) = parts_from_generics(&local_generics);
        let local_ty = syn::parse2(quote::quote!(#local_ident<#local_params>))?;
        let mut this = Self {
          $fields_attrs: local_fields_attrs,
          $ident: local_ident,
          $item: local_item,
          $params: local_params,
          $ty: local_ty,
          $where_predicates: local_where_predicates,
        };
        $(
          let cb: fn(&mut $struct<'_>) -> crate::Result<()> = $cb;
          cb(&mut this)?;
        )?
        Ok(this)
      }
    }

    fn manage_struct_field_attrs(
      fields: &mut Punctuated<Field, Token![,]>,
      req_fields_attrs: &mut Vec<Option<FirCustomFieldFieldAttr>>,
    ) -> crate::Result<()> {
      for field in fields {
        if let Some(FirCustomFieldAttr::Field(elem)) = take_unique_pkg_attr(&mut field.attrs)? {
          req_fields_attrs.push(Some(elem));
        } else {
          req_fields_attrs.push(None);
        };
      }
      Ok(())
    }
  };
}
