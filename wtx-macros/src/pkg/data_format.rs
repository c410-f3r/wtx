use crate::{pkg::data_format_elems::DataFormatElems, transport_group::TransportGroup};
use proc_macro2::{Ident, Span, TokenStream};
use syn::{Lit, Meta, NestedMeta};

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
        quote::quote!(
          _ext_req_params.method = wtx::http::Method::$method;
          _ext_req_params.mime = Some(wtx::http::Mime::$mime);
        )
      };
    }
    macro_rules! http_mime_type {
      ($mime:ident) => {
        quote::quote!(
          _ext_req_params.mime = Some(wtx::http::Mime::$mime);
        )
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
      DataFormat::Json => rslt!(http_mime_type!(Json)),
      DataFormat::JsonRpc(_) => rslt!(http_method_and_mime_type!(Post, Json)),
      DataFormat::Protobuf => rslt!(http_mime_type!(Protobuf)),
      _ => TokenStream::new(),
    }
  }

  pub(crate) fn elems(&self) -> DataFormatElems {
    let ident_fn = |name| Ident::new(name, Span::mixed_site());
    match self {
      DataFormat::Borsh => DataFormatElems {
        dfe_data_format_builder_fn: ident_fn("build_borsh"),
        dfe_ext_req_ctnt_wrapper: ident_fn("VerbatimRequest"),
        dfe_ext_res_ctnt_wrapper: ident_fn("VerbatimResponse"),
        dfe_pkgs_aux_call: quote::quote!(verbatim_request(data)),
      },
      DataFormat::Json => DataFormatElems {
        dfe_data_format_builder_fn: ident_fn("build_json"),
        dfe_ext_req_ctnt_wrapper: ident_fn("VerbatimRequest"),
        dfe_ext_res_ctnt_wrapper: ident_fn("VerbatimResponse"),
        dfe_pkgs_aux_call: quote::quote!(verbatim_request(data)),
      },
      DataFormat::JsonRpc(method) => DataFormatElems {
        dfe_data_format_builder_fn: ident_fn("build_json_rpc"),
        dfe_ext_req_ctnt_wrapper: ident_fn("JsonRpcRequest"),
        dfe_ext_res_ctnt_wrapper: ident_fn("JsonRpcResponse"),
        dfe_pkgs_aux_call: quote::quote!(json_rpc_request(#method, data)),
      },
      DataFormat::Protobuf => DataFormatElems {
        dfe_data_format_builder_fn: ident_fn("build_protobuf"),
        dfe_ext_req_ctnt_wrapper: ident_fn("VerbatimRequest"),
        dfe_ext_res_ctnt_wrapper: ident_fn("VerbatimResponse"),
        dfe_pkgs_aux_call: quote::quote!(verbatim_request(data)),
      },
      DataFormat::Verbatim => DataFormatElems {
        dfe_data_format_builder_fn: ident_fn("build_verbatim"),
        dfe_ext_req_ctnt_wrapper: ident_fn("VerbatimRequest"),
        dfe_ext_res_ctnt_wrapper: ident_fn("VerbatimResponse"),
        dfe_pkgs_aux_call: quote::quote!(verbatim_request(data)),
      },
    }
  }
}

impl<'attrs> TryFrom<&'attrs NestedMeta> for DataFormat {
  type Error = crate::Error;

  fn try_from(from: &'attrs NestedMeta) -> Result<Self, Self::Error> {
    macro_rules! first_path_seg_ident {
      ($path:expr) => {
        if let Some(elem) = $path.segments.first() {
          &elem.ident
        } else {
          return Err(crate::Error::UnknownDataFormat);
        }
      };
    }
    let NestedMeta::Meta(meta) = from else {
      return Err(crate::Error::UnknownDataFormat);
    };
    if let Meta::List(meta_list) = meta {
      let first_path_seg_ident = first_path_seg_ident!(meta_list.path);
      if first_path_seg_ident == "json_rpc" {
        if let Some(NestedMeta::Lit(Lit::Str(elem))) = meta_list.nested.first() {
          Ok(Self::JsonRpc(elem.value()))
        } else {
          Err(crate::Error::IncorrectJsonRpcDataFormat)
        }
      } else {
        Err(crate::Error::UnknownDataFormat)
      }
    } else if let Meta::Path(elem) = meta {
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
