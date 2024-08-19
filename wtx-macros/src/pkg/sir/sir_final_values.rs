use crate::{
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
      TransportGroup::Custom(tt) => {
        quote::quote!(<#tt as wtx::client_api_framework::network::transport::Transport<DRSR>>::Params)
      }
      TransportGroup::Http => quote::quote!(wtx::client_api_framework::network::HttpParams),
      TransportGroup::Stub => quote::quote!(()),
      TransportGroup::Tcp => quote::quote!(wtx::client_api_framework::network::TcpParams),
      TransportGroup::Udp => quote::quote!(wtx::client_api_framework::network::UdpParams),
      TransportGroup::WebSocket => quote::quote!(wtx::client_api_framework::network::WsParams),
    }
  }
}

impl<'attrs, 'module, 'others>
  TryFrom<(
    &'others mut String,
    FirParamsItemValues<'module>,
    FirReqItemValues<'module>,
    FirResItemValues<'others>,
    SirPkaAttr<'attrs>,
    Option<FirAfterSendingItemValues<'module>>,
    Option<FirAuxItemValues<'module>>,
    Option<FirBeforeSendingItemValues<'module>>,
  )> for SirFinalValues
{
  type Error = crate::Error;

  fn try_from(
    (camel_case_id, fpiv, freqdiv, fresdiv, spa, fasiv_opt, faiv_opt, fbsiv_opt): (
      &'others mut String,
      FirParamsItemValues<'module>,
      FirReqItemValues<'module>,
      FirResItemValues<'others>,
      SirPkaAttr<'attrs>,
      Option<FirAfterSendingItemValues<'module>>,
      Option<FirAuxItemValues<'module>>,
      Option<FirBeforeSendingItemValues<'module>>,
    ),
  ) -> Result<Self, Self::Error> {
    let FirParamsItemValues { fpiv_ty, fpiv_params, fpiv_where_predicates, .. } = &fpiv;
    let FirReqItemValues { freqdiv_ident, freqdiv_params, freqdiv_where_predicates, .. } = freqdiv;
    let FirResItemValues { res_ident } = fresdiv;
    let SirPkaAttr { api, data_formats, transport_groups } = &spa;
    let camel_case_pkg_ident = &{
      let idx = camel_case_id.len();
      camel_case_id.push_str("Pkg");
      let ident = Ident::new(camel_case_id, Span::mixed_site());
      camel_case_id.truncate(idx);
      ident
    };

    let fasiv_fn_call_idents = fasiv_opt.as_ref().map(|el| &el.fasiv_fn_call_idents);
    let fbsiv_fn_call_idents = fbsiv_opt.as_ref().map(|el| &el.fbsiv_fn_call_idents);
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
      for transport_group in transport_groups {
        let before_sending_defaults = data_format.before_sending_defaults(transport_group);
        let fasiv_fn_name_ident_iter =
          fasiv_opt.as_ref().map(|el| &el.fasiv_item.sig.ident).into_iter();
        let fbsiv_fn_name_ident_iter =
          fbsiv_opt.as_ref().map(|el| &el.fbsiv_item.sig.ident).into_iter();
        let fpiv_params_iter = fpiv_params.iter();
        let fpiv_where_predicates_iter = fpiv_where_predicates.iter();
        let freqdiv_where_predicates_iter = freqdiv_where_predicates.iter();
        let tp = Self::transport_params(transport_group);
        let (lts, tys) = Self::pkg_params(&freqdiv, &fpiv);
        package_impls.push(quote::quote!(
          impl<
            #(#lts,)*
            #(#tys,)*
            A,
            DRSR
          > wtx::client_api_framework::pkg::Package<A, DRSR, #tp> for #camel_case_pkg_ident<
            #(#fpiv_params_iter,)*
            wtx::data_transformation::format::#dfe_ext_req_ctnt_wrapper<#freqdiv_ident<#freqdiv_params>>
          >
          where
            #(#fpiv_where_predicates_iter,)*
            #(#freqdiv_where_predicates_iter,)*
            wtx::data_transformation::format::#dfe_ext_req_ctnt_wrapper<
              #freqdiv_ident<#freqdiv_params>
            >: wtx::data_transformation::dnsn::Serialize<DRSR>,
            for<'de> wtx::data_transformation::format::#dfe_ext_res_ctnt_wrapper<
              #res_ident
            >: wtx::data_transformation::dnsn::Deserialize<'de, DRSR>,
            A: wtx::client_api_framework::Api<Error = <#api as wtx::client_api_framework::Api>::Error> + wtx::misc::LeaseMut<#api>,
          {
            type ExternalRequestContent = wtx::data_transformation::format::#dfe_ext_req_ctnt_wrapper<
              #freqdiv_ident<#freqdiv_params>
            >;
            type ExternalResponseContent<'de> = wtx::data_transformation::format::#dfe_ext_res_ctnt_wrapper<
              #res_ident
            >;
            type PackageParams = #fpiv_ty;

            #[inline]
            async fn after_sending(
              &mut self,
              _api: &mut A,
              _ext_res_params: &mut <#tp as wtx::client_api_framework::network::transport::TransportParams>::ExternalResponseParams,
            ) -> Result<(), A::Error> {
              #( #fasiv_fn_name_ident_iter(#fasiv_fn_call_idents).await?; )*
              Ok(())
            }

            #[inline]
            async fn before_sending(
              &mut self,
              _api: &mut A,
              _ext_req_params: &mut <#tp as wtx::client_api_framework::network::transport::TransportParams>::ExternalRequestParams,
              _req_bytes: &[u8],
            ) -> Result<(), A::Error> {
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

    let fpiv_params_iter = fpiv_params.iter();
    Ok(Self {
      auxs: saiv_tts,
      package: quote::quote!(
        /// Package containing all the expected parameters and data necessary to manage and issue
        /// a request.
        ///
        /// For more information, please see the official API's documentation.
        #[derive(Debug)]
        pub struct #camel_case_pkg_ident<#(#fpiv_params_iter,)* C>
        where
          #fpiv_where_predicates
        {
          /// Content. Data format containing request data.
          pub content: C,
          /// Parameters. Used across the package lifetime.
          pub params: #fpiv_ty,
        }
      ),
      package_impls,
    })
  }
}
