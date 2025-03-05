use crate::{
  client_api_framework::pkg::{
    enum_struct_or_type::EnumStructOrType,
    fir::{fir_aux_item_values::FirAuxItemValues, fir_custom_item_values::FirCustomItemValuesRef},
    misc::{
      EMPTY_GEN_ARGS, EMPTY_PATH_SEGS, inner_angle_bracketed_values, is_unit_type, split_params,
    },
    sir::sir_aux_item_values::{
      BuilderCommonValues, BuilderExtendedValues, CreateMethodReturningBuilderParams,
      FnCommonValues, SirAuxItemValues,
    },
  },
  misc::parts_from_generics,
};
use proc_macro2::{Ident, Span, TokenStream};
use syn::{Fields, FnArg, ImplItemMethod, ReturnType, Type};

impl SirAuxItemValues {
  pub(super) fn create_automatic_fn_params<'any>(
    fcivr: FirCustomItemValuesRef<'_, 'any>,
  ) -> crate::Result<(FnCommonValues<'any>, TokenStream, bool)> {
    let FirCustomItemValuesRef { fields_attrs, ident, item, params, ty, where_predicates } = fcivr;
    let single_elem = |is_trivial| (quote::quote!(elem: #ty), quote::quote!(elem), is_trivial);
    let (fn_args, fn_ret_constr, is_trivial) = match item {
      EnumStructOrType::Enum => single_elem(false),
      EnumStructOrType::Struct(item_struct) => match &item_struct.fields {
        Fields::Named(fields_named) => {
          if fields_attrs.is_empty() {
            (quote::quote!(), quote::quote!(#ident {}), true)
          } else {
            let field_idents = fields_named
              .named
              .iter()
              .zip(fields_attrs)
              .map(|(struct_field, attr_field)| {
                Some(
                  attr_field
                    .as_ref()
                    .map_or_else(|| struct_field.ident.as_ref(), |el| Some(&el.name)),
                )
              })
              .collect::<Vec<_>>();
            let tys = fields_named.named.iter().map(|struct_field| &struct_field.ty);
            (
              quote::quote!(#(#field_idents: #tys,)*),
              quote::quote!(#ident {#(#field_idents,)*}),
              false,
            )
          }
        }
        Fields::Unnamed(fields_unnamed) => {
          if fields_attrs.is_empty() {
            (quote::quote!(), quote::quote!(#ident ()), true)
          } else {
            let field_idents = fields_attrs
              .iter()
              .map(|attr_field| {
                attr_field
                  .as_ref()
                  .map(|el| &el.name)
                  .ok_or_else(|| crate::Error::AbsentFieldInUnnamedStruct(item_struct.ident.span()))
              })
              .collect::<crate::Result<Vec<_>>>()?;
            let tys = fields_unnamed.unnamed.iter().map(|struct_field| &struct_field.ty);
            (
              quote::quote!(#(#field_idents: #tys,)*),
              quote::quote!(#ident (#(#field_idents,)*)),
              false,
            )
          }
        }
        Fields::Unit => (quote::quote!(), quote::quote!(#ident), true),
      },
      EnumStructOrType::Type(item_type) => {
        if let Type::Tuple(type_tuple) = &*item_type.ty {
          if is_unit_type(type_tuple) {
            (quote::quote!(), quote::quote!(()), true)
          } else {
            single_elem(false)
          }
        } else {
          single_elem(false)
        }
      }
    };
    Ok((
      FnCommonValues {
        fn_args,
        fn_params: params,
        fn_ret_angle_bracket_left: None,
        fn_ret_angle_bracket_right: None,
        fn_ret_wrapper_last_segment_gen_args: EMPTY_GEN_ARGS,
        fn_ret_wrapper_segments: EMPTY_PATH_SEGS,
        fn_ret_wrapper_variant_ident: None,
        fn_where_predicates: where_predicates,
      },
      fn_ret_constr,
      is_trivial,
    ))
  }

  pub(super) fn create_builder_struct(bcv: BuilderCommonValues<'_>) -> TokenStream {
    let (lts, tys) = Self::builder_params(bcv);
    let where_predicates = Self::builder_where_predicates(bcv);
    let FirAuxItemValues { faiv_ty, .. } = *bcv.faiv;
    let data_ty = bcv.freqdiv.map(|el| &el.freqdiv_ty).into_iter();
    let params_ty = bcv.fpiv.map(|el| &el.fpiv_ty).into_iter();
    let ident = bcv.ident;
    quote::quote!(
      /// Temporary building structure intended to statically create packages using a fluent
      /// interface.
      #[derive(Debug)]
      pub struct #ident<'aux, #(#lts,)* #(#tys,)*>
      where
        #(#where_predicates,)*
      {
        aux: &'aux mut #faiv_ty,
        #(data: #data_ty,)*
        #(params: #params_ty,)*
      }
    )
  }

  pub(super) fn create_manual_fn_params<'iim>(
    iim: &'iim ImplItemMethod,
    fn_name: &str,
  ) -> crate::Result<(FnCommonValues<'iim>, TokenStream)> {
    if iim.sig.ident != fn_name {
      return Err(crate::Error::BadAuxData(iim.sig.ident.span(), fn_name.to_owned()));
    }

    let (fn_params, fn_where_predicates) = parts_from_generics(&iim.sig.generics);
    let fn_args_iter_fn = || {
      iim.sig.inputs.iter().filter_map(|fn_arg| {
        if let FnArg::Typed(pat_type) = fn_arg { Some(pat_type) } else { None }
      })
    };

    let mut fn_ret_angle_bracket_left = None;
    let mut fn_ret_angle_bracket_right = None;
    let mut fn_ret_wrapper_last_segment_gen_args = EMPTY_GEN_ARGS;
    let mut fn_ret_wrapper_segments = EMPTY_PATH_SEGS;
    let mut fn_ret_wrapper_variant_ident = None;
    let mut has_short_circuit = false;
    if let ReturnType::Type(_, ret_ty) = &iim.sig.output {
      if let Some((tp, ps, abga)) = inner_angle_bracketed_values(ret_ty) {
        match ps.ident.to_string().as_str() {
          "Option" => {
            has_short_circuit = true;
            fn_ret_wrapper_variant_ident = Some(Ident::new("Some", Span::mixed_site()));
          }
          "Result" => {
            has_short_circuit = true;
            fn_ret_wrapper_variant_ident = Some(Ident::new("Ok", Span::mixed_site()));
          }
          _ => {}
        }
        fn_ret_angle_bracket_left = Some(&abga.lt_token);
        fn_ret_angle_bracket_right = Some(&abga.gt_token);
        fn_ret_wrapper_last_segment_gen_args = &abga.args;
        fn_ret_wrapper_segments = &tp.path.segments;
      }
    }

    let fn_args_iter = fn_args_iter_fn();
    let fn_args = quote::quote!(#(#fn_args_iter,)*);
    let fn_args_idents = fn_args_iter_fn().map(|el| &*el.pat);
    let method_ident = &iim.sig.ident;
    let fn_ret_constr = if has_short_circuit {
      quote::quote!(self.aux.#method_ident(#(#fn_args_idents,)*)?)
    } else {
      quote::quote!(self.aux.#method_ident(#(#fn_args_idents,)*))
    };

    Ok((
      FnCommonValues {
        fn_args,
        fn_params,
        fn_ret_angle_bracket_left,
        fn_ret_angle_bracket_right,
        fn_ret_wrapper_last_segment_gen_args,
        fn_ret_wrapper_segments,
        fn_ret_wrapper_variant_ident,
        fn_where_predicates,
      },
      fn_ret_constr,
    ))
  }

  pub(super) fn create_method_returning_builder(
    cmrbp: &CreateMethodReturningBuilderParams<'_>,
  ) -> TokenStream {
    let CreateMethodReturningBuilderParams {
      bev:
        BuilderExtendedValues {
          bcv: BuilderCommonValues { ident: builder_ident, .. },
          data_field_constr: builder_data_field_constr,
          fn_stmts: builder_fn_stmts,
          params_field_constr: builder_params_field_constr,
        },
      builder_aux_field_constr,
      fn_common_values:
        FnCommonValues {
          fn_args,
          fn_params,
          fn_ret_angle_bracket_left,
          fn_ret_angle_bracket_right,
          fn_ret_wrapper_last_segment_gen_args,
          fn_ret_wrapper_segments,
          fn_ret_wrapper_variant_ident,
          fn_where_predicates,
        },
      fn_name_ident,
      fn_this,
    } = cmrbp;

    let (fn_lfs, fn_tys) = split_params(fn_params);
    let (bdr_lts, bdr_tys) = Self::builder_params(cmrbp.bev.bcv);

    let mut fn_ret_wrapper_segments_idents = fn_ret_wrapper_segments.iter().map(|el| &el.ident);
    let fn_ret_wrapper_segments_idents_first = fn_ret_wrapper_segments_idents.next();

    let builder_data_field_constr_iter = builder_data_field_constr.iter();
    let builder_params_field_constr_iter = builder_params_field_constr.iter();
    let builder_fn_stmts_iter = builder_fn_stmts.iter();
    let fn_ret_wrapper_last_segment_gen_args_iter =
      fn_ret_wrapper_last_segment_gen_args.iter().skip(1);

    quote::quote!(
      /// Delegates to a new builder structure.
      ///
      /// This function must be called in order to continue constructing a package.
      pub fn #fn_name_ident<#(#fn_lfs,)* #(#fn_tys,)*>(
        #fn_this,
        #fn_args
      ) -> #fn_ret_wrapper_segments_idents_first #(::#fn_ret_wrapper_segments_idents)* #fn_ret_angle_bracket_left
        #builder_ident<'aux, #(#bdr_lts,)* #(#bdr_tys,)*>
        #(, #fn_ret_wrapper_last_segment_gen_args_iter)*
      #fn_ret_angle_bracket_right
      where
        #fn_where_predicates
      {
        #(#builder_fn_stmts_iter)*
        #fn_ret_wrapper_variant_ident (#builder_ident {
          #(params: #builder_params_field_constr_iter,)*
          #(data: #builder_data_field_constr_iter,)*
          aux: #builder_aux_field_constr,
        })
      }
    )
  }
}
