mod attrs;
pub(crate) mod mode;

use crate::{
  client_api_framework::{owned_or_ref::OwnedOrRef, transport_group::TransportGroup},
  misc::{Args, create_ident, parts_from_generics},
};
use proc_macro2::{Ident, Span};
use quote::{ToTokens as _, quote};
use syn::{Item, Path, PathArguments, PathSegment, punctuated::Punctuated, spanned::Spanned as _};

pub(crate) fn api(
  attrs: proc_macro::TokenStream,
  token_stream: proc_macro::TokenStream,
) -> crate::Result<proc_macro::TokenStream> {
  let attr_args: Args = syn::parse(attrs)?;
  let mut item: Item = syn::parse(token_stream)?;

  let attrs::Attrs { error, pkgs_aux, transports, mode: ty } =
    attrs::Attrs::try_from(&attr_args.0)?;

  let pkgs_aux_path = pkgs_aux.as_ref().map_or_else(
    || {
      let mut segments = Punctuated::new();
      segments.push(PathSegment {
        ident: Ident::new("PkgsAux", Span::mixed_site()),
        arguments: PathArguments::None,
      });
      OwnedOrRef::Owned(Path { leading_colon: None, segments })
    },
    OwnedOrRef::Ref,
  );

  let (api_ident, api_generics) = match &mut item {
    Item::Enum(container) => (&mut container.ident, &container.generics),
    Item::Struct(container) => (&mut container.ident, &container.generics),
    Item::Type(container) => (&mut container.ident, &container.generics),
    _ => return Err(crate::Error::NoEnumStructOrType(item.span())),
  };
  let api_string = api_ident.to_string();
  let (api_params, _api_where_predicates) = parts_from_generics(api_generics);

  let api_id_gat_ty = if api_params.is_empty() { quote::quote!() } else { quote!(__ApiParams) };

  let mut buffer = String::new();
  buffer.push_str(&api_string);

  let id_ident = create_ident(&mut buffer, ["Id"]);
  let id_impl = &quote::quote_spanned!(api_ident.span() =>
    #[doc = concat!("Identifier of the `", stringify!(#api_ident), "` API")]
    pub struct #id_ident;

    impl wtx::client_api_framework::ApiId for #id_ident {
      type Api<__ApiParams> = #api_ident<#api_id_gat_ty>;
    }
  );

  let api_impl = if let mode::Mode::Auto = ty {
    Some(quote::quote_spanned!(api_ident.span() =>
      impl<#api_params> wtx::client_api_framework::Api for #api_ident<#api_params> {
        type Error = #error;
        type Id = #id_ident;
      }
    ))
  } else {
    None
  };
  let api_impl_iter = api_impl.into_iter();

  let generic_pair_ident = create_ident(&mut buffer, ["Pair"]);
  let generic_pair_tt = quote::quote_spanned!(api_ident.span() =>
    #[allow(unused_qualifications)]
    #[doc = concat!("[wtx::client_api_framework::misc::Pair] with [", stringify!(#api_ident), "] as the API.")]
    pub type #generic_pair_ident<DRSR, T, TP, #api_params> = wtx::client_api_framework::misc::Pair<
      #pkgs_aux_path<#api_ident<#api_params>, DRSR, TP>,
      T
    >;
  );

  let generic_pkgs_aux_ident = create_ident(&mut buffer, ["PkgsAux"]);
  let generic_pkgs_aux_tt = quote::quote_spanned!(api_ident.span() =>
    #[doc = concat!("[", stringify!(#pkgs_aux_path), "] with an owned [", stringify!(#api_ident), "] as the API.")]
    pub type #generic_pkgs_aux_ident<DRSR, TP, #api_params> = #pkgs_aux_path<#api_ident<#api_params>, DRSR, TP>;
  );

  let generic_mut_pkgs_aux_ident = create_ident(&mut buffer, ["MutPkgsAux"]);
  let generic_mut_pkgs_aux_tt = quote::quote_spanned!(api_ident.span() =>
    #[doc = concat!("[", stringify!(#pkgs_aux_path), "] with a mutable reference of [", stringify!(#api_ident), "] as the API.")]
    pub type #generic_mut_pkgs_aux_ident<'api, DRSR, TP, #api_params> = #pkgs_aux_path<&'api mut #api_ident<#api_params>, DRSR, TP>;
  );

  let mut tys = Vec::new();
  let mut custom_placeholder;

  for transport in transports {
    let [camel_abbr, params] = match transport {
      TransportGroup::Custom(tt) => {
        custom_placeholder = tt.to_string();
        [custom_placeholder.as_str(), custom_placeholder.as_str()]
      }
      TransportGroup::Http => ["Http", "HttpParams"],
      TransportGroup::Stub => ["Stub", "()"],
      TransportGroup::WebSocket => ["Ws", "WsParams"],
    };
    let local_tp_ident = Ident::new(params, Span::mixed_site());
    let local_ty_ident_api_tp = create_ident(&mut buffer, [camel_abbr, "PkgsAux"]);
    let local_ty_ident_api_mut_tp = create_ident(&mut buffer, ["Mut", camel_abbr, "PkgsAux"]);
    buffer.clear();
    let local_ty_ident_tp = create_ident(&mut buffer, [camel_abbr, "PkgsAux"]);
    buffer.push_str(&api_string);
    tys.push(quote::quote!(
      #[allow(unused_qualifications)]
      #[doc = concat!(
        "[", stringify!(#pkgs_aux_path), "] with an owned [",
        stringify!(#api_ident),
        "] as the API and [wtx::client_api_framework::network::",
        stringify!(#local_tp_ident),
        "] as the transport parameter."
      )]
      pub type #local_ty_ident_api_tp<DRSR, #api_params> = #pkgs_aux_path<#api_ident<#api_params>, DRSR, wtx::client_api_framework::network::#local_tp_ident>;

      #[allow(unused_qualifications)]
      #[doc = concat!(
        "[", stringify!(#pkgs_aux_path), "] with a mutable reference of [",
        stringify!(#api_ident),
        "] as the API and [wtx::client_api_framework::network::",
        stringify!(#local_tp_ident),
        "] as the transport parameter."
      )]
      pub type #local_ty_ident_api_mut_tp<'api, DRSR, #api_params> = #pkgs_aux_path<&'api mut #api_ident<#api_params>, DRSR, wtx::client_api_framework::network::#local_tp_ident>;

      #[allow(unused_qualifications)]
      #[doc = concat!(
        "[", stringify!(#pkgs_aux_path), "] with [wtx::client_api_framework::network::",
        stringify!(#local_tp_ident),
        "] as the transport parameter."
      )]
      pub type #local_ty_ident_tp<A, DRSR> = #pkgs_aux_path<A, DRSR, wtx::client_api_framework::network::#local_tp_ident>;
    ));
  }

  let sts_ty = if api_params.is_empty() { quote::quote!(()) } else { api_params.to_token_stream() };

  Ok(
    quote::quote!(
      impl<#api_params> wtx::misc::Lease<#api_ident<#api_params>> for #api_ident<#api_params> {
        #[inline]
        fn lease(&self) -> &#api_ident<#api_params> {
          self
        }
      }

      impl<#api_params> wtx::misc::LeaseMut<#api_ident<#api_params>> for #api_ident<#api_params> {
        #[inline]
        fn lease_mut(&mut self) -> &mut #api_ident<#api_params> {
          self
        }
      }

      impl<#api_params> wtx::misc::SingleTypeStorage for #api_ident<#api_params> {
        type Item = #sts_ty;
      }

      #id_impl
      #(#api_impl_iter)*
      #item
      #generic_pair_tt
      #generic_pkgs_aux_tt
      #generic_mut_pkgs_aux_tt
      #(#tys)*
    )
    .to_token_stream()
    .into(),
  )
}
