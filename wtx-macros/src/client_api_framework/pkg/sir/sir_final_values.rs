use crate::client_api_framework::{
  pkg::{
    data_format_elems::DataFormatElems,
    fir::{
      fir_after_sending_item_values::FirAfterSendingItemValues,
      fir_aux_item_values::FirAuxItemValues,
      fir_before_sending_item_values::FirBeforeSendingItemValues,
      fir_params_items_values::FirParamsItemValues, fir_req_item_values::FirReqItemValues,
      fir_res_item_values::FirResItemValues,
    },
    misc::split_params,
    sir::{sir_aux_item_values::SirAuxItemValues, sir_pkg_attr::SirPkaAttr},
  },
  transport_group::TransportGroup,
};
use proc_macro2::{Ident, Span, TokenStream};
use syn::GenericParam;

pub(crate) struct SirFinalValues {
  pub(crate) auxs: Vec<TokenStream>,
  pub(crate) package: TokenStream,
  pub(crate) package_impls: Vec<TokenStream>,
}

impl SirFinalValues {
  fn pkg_params<'any>(
    freqdiv: &'any FirReqItemValues<'any>,
    fpiv: &'any FirParamsItemValues<'any>,
  ) -> (impl Iterator<Item = &'any GenericParam>, impl Iterator<Item = &'any GenericParam>) {
    let (a_lts, a_tys) = split_params(fpiv.fpiv_params);
    let (b_lts, b_tys) = split_params(freqdiv.freqdiv_params);
    (a_lts.chain(b_lts), a_tys.chain(b_tys))
  }

  fn transport_params(transport_group: &TransportGroup) -> TokenStream {
    match transport_group {
      TransportGroup::Custom(tt) => tt.clone(),
      TransportGroup::Http => quote::quote!(wtx::client_api_framework::network::HttpParams),
      TransportGroup::Stub => quote::quote!(()),
      TransportGroup::WebSocket => quote::quote!(wtx::client_api_framework::network::WsParams),
    }
  }
}

impl<'module, 'others>
  TryFrom<(
    &'others mut String,
    FirParamsItemValues<'module>,
    FirReqItemValues<'module>,
    FirResItemValues<'others>,
    SirPkaAttr,
    Option<FirAfterSendingItemValues<'module>>,
    Option<FirAuxItemValues<'module>>,
    Option<FirBeforeSendingItemValues<'module>>,
  )> for SirFinalValues
{
  type Error = crate::Error;

  #[inline]
  fn try_from(
    (camel_case_id, fpiv, freqdiv, fresdiv, spa, fasiv_opt, faiv_opt, fbsiv_opt): (
      &'others mut String,
      FirParamsItemValues<'module>,
      FirReqItemValues<'module>,
      FirResItemValues<'others>,
      SirPkaAttr,
      Option<FirAfterSendingItemValues<'module>>,
      Option<FirAuxItemValues<'module>>,
      Option<FirBeforeSendingItemValues<'module>>,
    ),
  ) -> Result<Self, Self::Error> {
    let FirParamsItemValues { fpiv_ident, fpiv_params, fpiv_where_predicates, .. } = &fpiv;
    let FirReqItemValues { freqdiv_ident, freqdiv_params, freqdiv_where_predicates, .. } = freqdiv;
    let FirResItemValues { fresdiv_ident, fresdiv_params, .. } = fresdiv;
    let SirPkaAttr { data_formats, id, transport_groups } = &spa;

    let res_lf = {
      let mut iter = fresdiv_params.iter();
      if let Some(elem) = iter.next() {
        if matches!(elem, GenericParam::Lifetime(_)) && iter.next().is_none() {
          Some(quote::quote!('__de))
        } else {
          // FIXME(STABLE): non_lifetime_binders
          return Err(crate::Error::ResponsesCanHaveAtMostOneLt(fresdiv_ident.span()));
        }
      } else {
        None
      }
    };

    let camel_case_pkg_ident = &{
      let idx = camel_case_id.len();
      camel_case_id.push_str("Pkg");
      let ident = Ident::new(camel_case_id, Span::mixed_site());
      camel_case_id.truncate(idx);
      ident
    };

    let fasiv_fn_call_idents = fasiv_opt.as_ref().map(|el| &el.fasiv_fn_call_idents);
    let fasiv_fn_where_predicates = fasiv_opt.as_ref().map(|el| &el.fasiv_where_predicates);

    let fbsiv_fn_call_idents = fbsiv_opt.as_ref().map(|el| &el.fbsiv_fn_call_idents);
    let fbsiv_fn_where_predicates = fbsiv_opt.as_ref().map(|el| &el.fbsiv_where_predicates);

    let saiv_tts = faiv_opt
      .as_ref()
      .map(|elem| {
        SirAuxItemValues::try_from((
          camel_case_id,
          camel_case_pkg_ident,
          elem,
          &fpiv,
          &freqdiv,
          &spa,
        ))
      })
      .transpose()?
      .map(|elem| elem.saiv_tts)
      .unwrap_or_default();
    let mut package_impls = Vec::new();

    for data_format in data_formats {
      let DataFormatElems { dfe_ext_req_ctnt_wrapper, dfe_ext_res_ctnt_wrapper, .. } =
        data_format.elems();
      let iter = transport_groups
        .iter()
        .map(|el| (false, el))
        .chain(transport_groups.iter().map(|el| (true, el)));
      for (is_mut, transport_group) in iter {
        let res_lf_iter0 = res_lf.iter();
        let res_lf_iter1 = res_lf.iter();
        let before_sending_defaults = data_format.before_sending_defaults(transport_group);
        let fasiv_fn_name_ident_iter =
          fasiv_opt.as_ref().map(|el| &el.fasiv_item.sig.ident).into_iter();
        let fbsiv_fn_name_ident_iter =
          fbsiv_opt.as_ref().map(|el| &el.fbsiv_item.sig.ident).into_iter();
        let fpiv_params_iter0 = fpiv_params.iter();
        let fpiv_params_iter1 = fpiv_params.iter();
        let fpiv_where_predicates_iter = fpiv_where_predicates.iter();
        let freqdiv_where_predicates_iter = freqdiv_where_predicates.iter();
        let (lts, tys) = Self::pkg_params(&freqdiv, &fpiv);
        let is_mut_lf = is_mut.then(|| quote::quote! { '__is_mut }).into_iter();
        let tp = {
          let tt = Self::transport_params(transport_group);
          if is_mut { quote::quote!(&'__is_mut mut #tt) } else { tt }
        };
        package_impls.push(quote::quote!(
          impl<
            #(#is_mut_lf,)* #(#lts,)* #(#tys,)* __API, __API_PARAMS, __DRSR, __TRANSPORT
          > wtx::client_api_framework::pkg::Package<
            __API, __DRSR, __TRANSPORT, #tp
          > for #camel_case_pkg_ident<
            #(#fpiv_params_iter0,)*
            wtx::data_transformation::format::#dfe_ext_req_ctnt_wrapper<#freqdiv_ident<#freqdiv_params>>
          >
          where
            #fasiv_fn_where_predicates
            #fbsiv_fn_where_predicates
            #(#fpiv_where_predicates_iter,)*
            #(#freqdiv_where_predicates_iter,)*
            wtx::data_transformation::format::#dfe_ext_req_ctnt_wrapper<
              #freqdiv_ident<#freqdiv_params>
            >: wtx::misc::Encode<wtx::data_transformation::dnsn::De<__DRSR>>,
            for<'__de> wtx::data_transformation::format::#dfe_ext_res_ctnt_wrapper<
              #fresdiv_ident<#(#res_lf_iter0)*>
            >: wtx::misc::DecodeSeq<'__de, wtx::data_transformation::dnsn::De<__DRSR>>,
            __API: wtx::client_api_framework::Api<
                Error = <<#id as wtx::client_api_framework::ApiId>::Api<__API_PARAMS> as wtx::client_api_framework::Api>::Error,
                Id = #id
              >
              + wtx::misc::LeaseMut<<#id as wtx::client_api_framework::ApiId>::Api<__API_PARAMS>>
              + wtx::misc::SingleTypeStorage<Item = __API_PARAMS>
          {
            type ExternalRequestContent = wtx::data_transformation::format::#dfe_ext_req_ctnt_wrapper<
              #freqdiv_ident<#freqdiv_params>
            >;
            type ExternalResponseContent<'__de> = wtx::data_transformation::format::#dfe_ext_res_ctnt_wrapper<
              #fresdiv_ident<#(#res_lf_iter1)*>
            >;
            type PackageParams = #fpiv_ident< #(#fpiv_params_iter1)* >;

            #[inline]
            async fn after_sending(
              &mut self,
              (_api, _bytes, _drsr): (&mut __API, &mut wtx::collection::Vector<u8>, &mut __DRSR),
              (_trans, _trans_params): (&mut __TRANSPORT, &mut #tp),
            ) -> Result<(), __API::Error> {
              #( #fasiv_fn_name_ident_iter(#fasiv_fn_call_idents).await?; )*
              Ok(())
            }

            #[inline]
            async fn before_sending(
              &mut self,
              (_api, _bytes, _drsr): (&mut __API, &mut wtx::collection::Vector<u8>, &mut __DRSR),
              (_trans, _trans_params): (&mut __TRANSPORT, &mut #tp),
            ) -> Result<(), __API::Error> {
              #before_sending_defaults
              #( #fbsiv_fn_name_ident_iter(#fbsiv_fn_call_idents).await?; )*
              Ok(())
            }

            #[inline]
            fn ext_req_content(&self) -> &Self::ExternalRequestContent {
              &self.content
            }

            #[inline]
            fn ext_req_content_mut(&mut self) -> &mut Self::ExternalRequestContent {
              &mut self.content
            }

            #[inline]
            fn pkg_params(&self) -> &Self::PackageParams {
              &self.params
            }

            #[inline]
            fn pkg_params_mut(&mut self) -> &mut Self::PackageParams {
              &mut self.params
            }
          }
        ));
      }
    }

    let fpiv_params_iter0 = fpiv_params.iter();
    let fpiv_params_iter1 = fpiv_params.iter();
    Ok(Self {
      auxs: saiv_tts,
      package: quote::quote!(
        /// Package containing all the expected parameters and data necessary to manage and issue
        /// a request.
        ///
        /// For more information, please see the official API's documentation.
        #[derive(Debug)]
        pub struct #camel_case_pkg_ident<#(#fpiv_params_iter0,)* C>
        where
          #fpiv_where_predicates
        {
          /// Content. Data format containing request data.
          pub content: C,
          /// Parameters. Used across the package lifetime.
          pub params: #fpiv_ident<#(#fpiv_params_iter1)*>,
        }
      ),
      package_impls,
    })
  }
}
