use crate::client_api_framework::pkg::{
  data_format::DataFormat,
  fir::{
    fir_aux_item_values::FirAuxItemValues, fir_params_items_values::FirParamsItemValues,
    fir_req_item_values::FirReqItemValues,
  },
  misc::{EMPTY_GEN_ARGS, EMPTY_PATH_SEGS, EMPTY_WHERE_PREDS},
  sir::sir_aux_item_values::{
    BuilderCommonValues, BuilderExtendedValues, CreateMethodReturningBuilderParams, FnCommonValues,
    SirAuxItemValues,
  },
};
use proc_macro2::{Ident, Span, TokenStream};
use syn::{punctuated::Punctuated, GenericParam, Lifetime, LifetimeDef};

impl SirAuxItemValues {
  pub(super) fn push_builder_method_returning_builder(
    fn_name_ident: &Ident,
    fn_common_values: &FnCommonValues<'_>,
    saiv_tts: &mut Vec<TokenStream>,
    impl_values: BuilderCommonValues<'_>,
    ret_values: BuilderExtendedValues<'_>,
  ) {
    let method = Self::create_method_returning_builder(&CreateMethodReturningBuilderParams {
      bev: ret_values,
      builder_aux_field_constr: &quote::quote!(self.aux),
      fn_common_values,
      fn_name_ident,
      fn_this: &quote::quote!(self),
    });
    let impl_ident = impl_values.ident;
    let (lts0, tys0) = Self::builder_params(impl_values);
    let (lts1, tys1) = Self::builder_params(impl_values);
    let where_predicates = Self::builder_where_predicates(impl_values);
    saiv_tts.push(quote::quote!(
      impl<'aux, #(#lts0,)* #(#tys0,)*> #impl_ident<'aux, #(#lts1,)* #(#tys1,)*>
      where
        #(#where_predicates,)*
      {
        #method
      }
    ));
    saiv_tts.push(Self::create_builder_struct(ret_values.bcv));
  }

  pub(super) fn push_dt_methods_returning_pkg(
    data_formats: &[DataFormat],
    fpiv: &FirParamsItemValues<'_>,
    freqdiv: &FirReqItemValues<'_>,
    pkg_ident: &Ident,
    saiv_tts: &mut Vec<TokenStream>,
    impl_values: BuilderCommonValues<'_>,
  ) {
    let mut do_push = |aux_call: &TokenStream, fn_ident: &Ident, wrapper_ident: &Ident| {
      let FirParamsItemValues { fpiv_params, .. } = *fpiv;
      let FirReqItemValues { freqdiv_ident, freqdiv_params, .. } = *freqdiv;
      let fpiv_params_iter = fpiv_params.iter();
      let method = quote::quote!(
        /// Final building method that creates a package with all the necessary values.
        pub fn #fn_ident(self) -> #pkg_ident<
          #(#fpiv_params_iter,)*
          wtx::data_transformation::format::#wrapper_ident<#freqdiv_ident<#freqdiv_params>>
        > {
          let data = self.data;
          let content = self.aux.#aux_call;
          let params = self.params;
          #pkg_ident { content, params }
        }
      );
      let impl_ident = impl_values.ident;
      let (lts0, tys0) = Self::builder_params(impl_values);
      let (lts1, tys1) = Self::builder_params(impl_values);
      let where_predicates = Self::builder_where_predicates(impl_values);
      saiv_tts.push(quote::quote!(
        impl<'aux, #(#lts0,)* #(#tys0,)*> #impl_ident<'aux, #(#lts1,)* #(#tys1,)*>
        where
          #(#where_predicates,)*
        {
          #method
        }
      ));
    };
    if data_formats.len() > 1 {
      for data_format in data_formats {
        let dfe = data_format.elems();
        do_push(
          &dfe.dfe_pkgs_aux_call,
          &dfe.dfe_data_format_builder_fn,
          &dfe.dfe_ext_req_ctnt_wrapper,
        );
      }
    } else {
      for data_format in data_formats {
        let dfe = data_format.elems();
        do_push(
          &dfe.dfe_pkgs_aux_call,
          &Ident::new("build", Span::mixed_site()),
          &dfe.dfe_ext_req_ctnt_wrapper,
        );
      }
    }
  }

  pub(super) fn push_pkgs_aux_method_returning_builder(
    pkgs_aux_fn_name_ident: &Ident,
    saiv_tts: &mut Vec<TokenStream>,
    ret_values: BuilderExtendedValues<'_>,
  ) {
    let method = Self::create_method_returning_builder(&CreateMethodReturningBuilderParams {
      bev: ret_values,
      builder_aux_field_constr: &quote::quote!(self),
      fn_common_values: &FnCommonValues {
        fn_args: TokenStream::new(),
        fn_params: &{
          let mut generic_params = Punctuated::new();
          generic_params.push(GenericParam::Lifetime(LifetimeDef {
            attrs: Vec::new(),
            lifetime: Lifetime {
              apostrophe: Span::mixed_site(),
              ident: Ident::new("aux", Span::mixed_site()),
            },
            colon_token: None,
            bounds: Punctuated::new(),
          }));
          generic_params
        },
        fn_ret_angle_bracket_left: None,
        fn_ret_angle_bracket_right: None,
        fn_ret_wrapper_last_segment_gen_args: EMPTY_GEN_ARGS,
        fn_ret_wrapper_segments: EMPTY_PATH_SEGS,
        fn_ret_wrapper_variant_ident: None,
        fn_where_predicates: EMPTY_WHERE_PREDS,
      },
      fn_name_ident: pkgs_aux_fn_name_ident,
      fn_this: &quote::quote!(&'aux mut self),
    });
    let FirAuxItemValues { faiv_params, faiv_ty, faiv_where_predicates, .. } = *ret_values.bcv.faiv;
    saiv_tts.push(quote::quote!(
      impl<#faiv_params> #faiv_ty
      where
        #faiv_where_predicates
      {
        #method
      }
    ));
    saiv_tts.push(Self::create_builder_struct(ret_values.bcv));
  }
}
