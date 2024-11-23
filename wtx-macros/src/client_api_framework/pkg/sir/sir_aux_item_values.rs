mod sir_items_values_creators;
mod sir_items_values_pushers;

use crate::{
  client_api_framework::pkg::{
    fir::{
      fir_aux_item_values::FirAuxItemValues, fir_custom_item_values::FirCustomItemValuesRef,
      fir_params_items_values::FirParamsItemValues, fir_req_item_values::FirReqItemValues,
    },
    misc::{from_camel_case_to_snake_case, split_params, EMPTY_GEN_PARAMS},
    sir::sir_pkg_attr::SirPkaAttr,
  },
  misc::{create_ident, extend_with_tmp_suffix},
};
use proc_macro2::{Ident, Span, TokenStream};
use syn::{
  punctuated::Punctuated, GenericArgument, GenericParam, ImplItemMethod, PathSegment, Token,
  WherePredicate,
};

pub(crate) struct SirAuxItemValues {
  pub(crate) saiv_tts: Vec<TokenStream>,
}

impl SirAuxItemValues {
  fn builder_params(
    bcv: BuilderCommonValues<'_>,
  ) -> (impl Iterator<Item = &GenericParam>, impl Iterator<Item = &GenericParam>) {
    let (a_lts, a_tys) = split_params(bcv.faiv.faiv_params);
    let (b_lts, b_tys) = split_params(bcv.fpiv.map_or(EMPTY_GEN_PARAMS, |el| el.fpiv_params));
    let (c_lts, c_tys) = split_params(bcv.freqdiv.map_or(EMPTY_GEN_PARAMS, |el| el.freqdiv_params));
    (a_lts.chain(b_lts).chain(c_lts), a_tys.chain(b_tys).chain(c_tys))
  }

  fn builder_where_predicates(
    bcv: BuilderCommonValues<'_>,
  ) -> impl Iterator<Item = &WherePredicate> {
    bcv
      .faiv
      .faiv_where_predicates
      .iter()
      .chain(bcv.fpiv.into_iter().flat_map(|el| el.fpiv_where_predicates))
      .chain(bcv.freqdiv.into_iter().flat_map(|el| el.freqdiv_where_predicates))
  }

  fn fn_params<'any>(
    faiv_user_method: Option<&'any ImplItemMethod>,
    fcivr: FirCustomItemValuesRef<'_, 'any>,
    snake_case_id: &mut String,
    suffix: &str,
  ) -> crate::Result<(FnCommonValues<'any>, TokenStream, bool)> {
    if let Some(elem) = faiv_user_method {
      let idx = extend_with_tmp_suffix(snake_case_id, [suffix]);
      let tuple = Self::create_manual_fn_params(elem, snake_case_id)?;
      snake_case_id.truncate(idx);
      Ok((tuple.0, tuple.1, false))
    } else {
      Self::create_automatic_fn_params(fcivr)
    }
  }
}

impl<'attrs, 'module, 'others>
  TryFrom<(
    &'others mut String,
    &'others Ident,
    &'others FirAuxItemValues<'module>,
    &'others FirParamsItemValues<'module>,
    &'others FirReqItemValues<'module>,
    &'others SirPkaAttr<'attrs>,
  )> for SirAuxItemValues
{
  type Error = crate::Error;

  fn try_from(
    (camel_case_id, pkg_ident, faiv, fpiv, freqdiv, spa): (
      &'others mut String,
      &'others Ident,
      &'others FirAuxItemValues<'module>,
      &'others FirParamsItemValues<'module>,
      &'others FirReqItemValues<'module>,
      &'others SirPkaAttr<'attrs>,
    ),
  ) -> Result<Self, Self::Error> {
    let mut snake_case_id = from_camel_case_to_snake_case(camel_case_id);

    let (data_builder_fn_common_values, data_builder_fn_ret_constr, data_builder_is_trivial) =
      Self::fn_params(faiv.faiv_user_data_method, freqdiv.into(), &mut snake_case_id, "_data")?;
    let (params_builder_fn_common_values, params_builder_fn_ret_constr, params_builder_is_trivial) =
      Self::fn_params(faiv.faiv_user_params_method, fpiv.into(), &mut snake_case_id, "_params")?;

    let data_builder_fn_name_ident = &Ident::new("data", Span::mixed_site());
    let data_builder_ident = &create_ident(camel_case_id, ["DataBuilder"]);
    let data_format_builder_ident = &create_ident(camel_case_id, ["DataFormatBuilder"]);
    let params_builder_fn_name_ident = &Ident::new("params", Span::mixed_site());
    let params_builder_ident = &create_ident(camel_case_id, ["ParamsBuilder"]);
    let pkgs_aux_fn_name_ident = &Ident::new(&snake_case_id, Span::mixed_site());
    let mut saiv_tts = Vec::new();

    let pkgs_aux_method_ret_values = match (data_builder_is_trivial, params_builder_is_trivial) {
      (false, false) => {
        Self::push_builder_method_returning_builder(
          data_builder_fn_name_ident,
          &data_builder_fn_common_values,
          &mut saiv_tts,
          BuilderCommonValues { faiv, fpiv: None, freqdiv: None, ident: data_builder_ident },
          BuilderExtendedValues {
            bcv: BuilderCommonValues {
              ident: params_builder_ident,
              faiv,
              freqdiv: Some(freqdiv),
              fpiv: None,
            },
            data_field_constr: Some(&data_builder_fn_ret_constr),
            params_field_constr: None,
          },
        );
        Self::push_builder_method_returning_builder(
          params_builder_fn_name_ident,
          &params_builder_fn_common_values,
          &mut saiv_tts,
          BuilderCommonValues {
            faiv,
            fpiv: None,
            freqdiv: Some(freqdiv),
            ident: data_builder_ident,
          },
          BuilderExtendedValues {
            bcv: BuilderCommonValues {
              ident: data_format_builder_ident,
              faiv,
              freqdiv: Some(freqdiv),
              fpiv: Some(fpiv),
            },
            data_field_constr: Some(&quote::quote!(self.data)),
            params_field_constr: Some(&params_builder_fn_ret_constr),
          },
        );
        BuilderExtendedValues {
          bcv: BuilderCommonValues { ident: data_builder_ident, faiv, freqdiv: None, fpiv: None },
          data_field_constr: None,
          params_field_constr: None,
        }
      }
      (false, true) => {
        Self::push_builder_method_returning_builder(
          data_builder_fn_name_ident,
          &data_builder_fn_common_values,
          &mut saiv_tts,
          BuilderCommonValues { faiv, fpiv: Some(fpiv), freqdiv: None, ident: data_builder_ident },
          BuilderExtendedValues {
            bcv: BuilderCommonValues {
              ident: data_format_builder_ident,
              faiv,
              freqdiv: Some(freqdiv),
              fpiv: Some(fpiv),
            },
            data_field_constr: Some(&data_builder_fn_ret_constr),
            params_field_constr: Some(&params_builder_fn_ret_constr),
          },
        );
        BuilderExtendedValues {
          bcv: BuilderCommonValues {
            ident: data_builder_ident,
            faiv,
            freqdiv: None,
            fpiv: Some(fpiv),
          },
          data_field_constr: None,
          params_field_constr: Some(&params_builder_fn_ret_constr),
        }
      }
      (true, false) => {
        Self::push_builder_method_returning_builder(
          params_builder_fn_name_ident,
          &params_builder_fn_common_values,
          &mut saiv_tts,
          BuilderCommonValues {
            faiv,
            fpiv: None,
            freqdiv: Some(freqdiv),
            ident: params_builder_ident,
          },
          BuilderExtendedValues {
            bcv: BuilderCommonValues {
              ident: data_format_builder_ident,
              faiv,
              freqdiv: Some(freqdiv),
              fpiv: Some(fpiv),
            },
            data_field_constr: Some(&data_builder_fn_ret_constr),
            params_field_constr: Some(&params_builder_fn_ret_constr),
          },
        );
        BuilderExtendedValues {
          bcv: BuilderCommonValues {
            ident: params_builder_ident,
            faiv,
            freqdiv: Some(freqdiv),
            fpiv: None,
          },
          data_field_constr: Some(&data_builder_fn_ret_constr),
          params_field_constr: None,
        }
      }
      (true, true) => BuilderExtendedValues {
        bcv: BuilderCommonValues {
          ident: data_format_builder_ident,
          faiv,
          freqdiv: Some(freqdiv),
          fpiv: Some(fpiv),
        },
        data_field_constr: Some(&data_builder_fn_ret_constr),
        params_field_constr: Some(&params_builder_fn_ret_constr),
      },
    };

    Self::push_pkgs_aux_method_returning_builder(
      pkgs_aux_fn_name_ident,
      &mut saiv_tts,
      pkgs_aux_method_ret_values,
    );
    Self::push_dt_methods_returning_pkg(
      &spa.data_formats,
      fpiv,
      freqdiv,
      pkg_ident,
      &mut saiv_tts,
      BuilderCommonValues {
        faiv,
        fpiv: Some(fpiv),
        freqdiv: Some(freqdiv),
        ident: data_format_builder_ident,
      },
    );

    Ok(Self { saiv_tts })
  }
}

#[derive(Clone, Copy, Debug)]
struct BuilderCommonValues<'any> {
  faiv: &'any FirAuxItemValues<'any>,
  fpiv: Option<&'any FirParamsItemValues<'any>>,
  freqdiv: Option<&'any FirReqItemValues<'any>>,
  ident: &'any Ident,
}

#[derive(Clone, Copy, Debug)]
struct BuilderExtendedValues<'any> {
  bcv: BuilderCommonValues<'any>,
  data_field_constr: Option<&'any TokenStream>,
  params_field_constr: Option<&'any TokenStream>,
}

struct CreateMethodReturningBuilderParams<'any> {
  bev: BuilderExtendedValues<'any>,
  builder_aux_field_constr: &'any TokenStream,
  fn_common_values: &'any FnCommonValues<'any>,
  fn_name_ident: &'any Ident,
  fn_this: &'any TokenStream,
}

struct FnCommonValues<'any> {
  fn_args: TokenStream,
  fn_params: &'any Punctuated<GenericParam, Token![,]>,
  fn_ret_angle_bracket_left: Option<&'any Token![<]>,
  fn_ret_angle_bracket_right: Option<&'any Token![>]>,
  fn_ret_wrapper_last_segment_gen_args: &'any Punctuated<GenericArgument, Token![,]>,
  fn_ret_wrapper_segments: &'any Punctuated<PathSegment, Token![::]>,
  fn_ret_wrapper_variant_ident: Option<Ident>,
  fn_where_predicates: &'any Punctuated<WherePredicate, Token![,]>,
}
