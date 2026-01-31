use crate::client_api_framework::{
  pkg::data_format_elems::DataFormatElems, transport_group::TransportGroup,
};
use proc_macro2::{Ident, Span, TokenStream};
use syn::{
  LitStr, Meta,
  parse::{Parse, ParseStream},
};

#[derive(Debug)]
pub(crate) enum DataFormat {
  Borsh,
  Json,
  JsonRpc(String),
  Protobuf,
  Verbatim,
}

impl DataFormat {
  pub(crate) fn before_sending_defaults(&self, tg: &TransportGroup) -> TokenStream {
    macro_rules! http_method_and_mime_type {
      ($method:ident, $mime:ident) => {
        quote::quote!({
          use wtx::client_api_framework::network::transport::TransportParams;
          _trans_params.ext_req_params_mut().method = wtx::http::Method::$method;
          _trans_params.ext_req_params_mut().mime = Some(wtx::http::Mime::$mime);
        })
      };
    }
    macro_rules! http_mime_type {
      ($mime:ident) => {
        quote::quote!({
          use wtx::client_api_framework::network::transport::TransportParams;
          _trans_params.ext_req_params_mut().mime = Some(wtx::http::Mime::$mime);
        })
      };
    }
    macro_rules! rslt {
      ($http_tt:expr) => {
        match *tg {
          TransportGroup::Http => $http_tt,
          _ => TokenStream::new(),
        }
      };
    }
    match *self {
      DataFormat::Json => rslt!(http_mime_type!(ApplicationJson)),
      DataFormat::JsonRpc(_) => rslt!(http_method_and_mime_type!(Post, ApplicationJson)),
      DataFormat::Protobuf => rslt!(http_mime_type!(ApplicationVndGoogleProtobuf)),
      _ => TokenStream::new(),
    }
  }

  pub(crate) fn elems(&self) -> DataFormatElems {
    let ident_fn = |name| Ident::new(name, Span::mixed_site());
    match self {
      DataFormat::Borsh => DataFormatElems {
        dfe_data_format_builder_fn: ident_fn("build_borsh"),
        dfe_ext_req_ctnt_wrapper: ident_fn("VerbatimEncoder"),
        dfe_ext_res_ctnt_wrapper: ident_fn("VerbatimDecoder"),
        dfe_pkgs_aux_call: quote::quote!(verbatim_request(data)),
      },
      DataFormat::Json => DataFormatElems {
        dfe_data_format_builder_fn: ident_fn("build_json"),
        dfe_ext_req_ctnt_wrapper: ident_fn("VerbatimEncoder"),
        dfe_ext_res_ctnt_wrapper: ident_fn("VerbatimDecoder"),
        dfe_pkgs_aux_call: quote::quote!(verbatim_request(data)),
      },
      DataFormat::JsonRpc(method) => DataFormatElems {
        dfe_data_format_builder_fn: ident_fn("build_json_rpc"),
        dfe_ext_req_ctnt_wrapper: ident_fn("JsonRpcEncoder"),
        dfe_ext_res_ctnt_wrapper: ident_fn("JsonRpcDecoder"),
        dfe_pkgs_aux_call: quote::quote!(json_rpc_request(#method, data)),
      },
      DataFormat::Protobuf => DataFormatElems {
        dfe_data_format_builder_fn: ident_fn("build_protobuf"),
        dfe_ext_req_ctnt_wrapper: ident_fn("VerbatimEncoder"),
        dfe_ext_res_ctnt_wrapper: ident_fn("VerbatimDecoder"),
        dfe_pkgs_aux_call: quote::quote!(verbatim_request(data)),
      },
      DataFormat::Verbatim => DataFormatElems {
        dfe_data_format_builder_fn: ident_fn("build_verbatim"),
        dfe_ext_req_ctnt_wrapper: ident_fn("VerbatimEncoder"),
        dfe_ext_res_ctnt_wrapper: ident_fn("VerbatimDecoder"),
        dfe_pkgs_aux_call: quote::quote!(verbatim_request(data)),
      },
    }
  }
}

impl TryFrom<&Meta> for DataFormat {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &Meta) -> Result<Self, Self::Error> {
    macro_rules! first_path_seg_ident {
      ($path:expr) => {
        if let Some(elem) = $path.segments.first() {
          &elem.ident
        } else {
          return Err(crate::Error::UnknownDataFormat);
        }
      };
    }
    if let Meta::List(meta_list) = from {
      let first_path_seg_ident = first_path_seg_ident!(meta_list.path);
      if first_path_seg_ident == "json_rpc" {
        let arg = syn::parse2::<JsonRpcArg>(meta_list.tokens.clone())
          .map_err(|_err| crate::Error::IncorrectJsonRpcDataFormat)?
          .0;
        Ok(Self::JsonRpc(arg.value()))
      } else {
        Err(crate::Error::UnknownDataFormat)
      }
    } else if let Meta::Path(elem) = from {
      match first_path_seg_ident!(elem).to_string().as_str() {
        "borsh" => Ok(Self::Borsh),
        "json" => Ok(Self::Json),
        "protobuf" => Ok(Self::Protobuf),
        "verbatim" => Ok(Self::Verbatim),
        _ => Err(crate::Error::UnknownDataFormat),
      }
    } else {
      Err(crate::Error::MandatoryOuterAttrsAreNotPresent)
    }
  }
}

struct JsonRpcArg(LitStr);

impl Parse for JsonRpcArg {
  fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
    let elem: LitStr = input.parse()?;
    Ok(Self(elem))
  }
}
