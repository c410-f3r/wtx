use crate::misc::Args;
use syn::{FnArg, ItemFn, Meta, Pat};

pub(crate) fn db(
  attrs: proc_macro::TokenStream,
  item: proc_macro::TokenStream,
) -> crate::Result<proc_macro::TokenStream> {
  let [dir_ts, tls_config_ts] = attrs_ts(attrs)?;
  let input_fn: ItemFn = syn::parse(item)?;

  let mut has_conn = false;
  for input in &input_fn.sig.inputs {
    if let FnArg::Typed(pat_type) = input
      && let Pat::Ident(pat_ident) = &*pat_type.pat
    {
      let name = pat_ident.ident.to_string();
      if name == "client" {
        has_conn = true;
      }
    }
  }

  let fn_attrs = &input_fn.attrs;
  let fn_block = &input_fn.block;
  let fn_sig = &input_fn.sig;

  let fn_asyncness = &fn_sig.asyncness;
  let fn_inputs = &fn_sig.inputs;
  let fn_name = &fn_sig.ident;
  let fn_output = &fn_sig.output;

  let mut priv_fn_args = Vec::new();
  if has_conn {
    priv_fn_args.push(quote::quote!(client));
  }
  let priv_fn_name = &syn::Ident::new(&format!("__{fn_name}"), fn_name.span());

  let tokens = quote::quote!(
    #(#fn_attrs)*
    #[test]
    fn #fn_name() #fn_output {
      use wtx::executor::Runtime as _;

      #fn_asyncness fn #priv_fn_name(#fn_inputs) #fn_output {
        #fn_block
      }

      wtx::database::client::postgres::database_test(
        #dir_ts,
        #tls_config_ts,
        #priv_fn_name
      ).unwrap()
    }
  );
  Ok(tokens.into())
}

fn attrs_ts(attrs: proc_macro::TokenStream) -> crate::Result<[proc_macro2::TokenStream; 2]> {
  let attrs_args: Args = syn::parse(attrs)?;
  let mut dir = None;
  let mut tls_config_ts = None;
  for arg in attrs_args.0 {
    if let Meta::List(meta_list) = arg {
      if meta_list.path.is_ident("dir") {
        let value: syn::LitStr = meta_list.parse_args()?;
        dir = Some(value.value());
      } else if meta_list.path.is_ident("tls_config") {
        let value: syn::Expr = meta_list.parse_args()?;
        tls_config_ts = Some(value);
      }
    }
  }
  Ok([
    if let Some(elem) = dir { quote::quote!(Some(#elem)) } else { quote::quote!(None) },
    if let Some(elem) = tls_config_ts {
      quote::quote!(#elem)
    } else {
      quote::quote!(wtx::tls::TlsConfig::new(Default::default()).unwrap())
    },
  ])
}
