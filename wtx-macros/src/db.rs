use crate::misc::Args;
use syn::{FnArg, ItemFn, Meta, Pat};

pub(crate) fn db(
  attrs: proc_macro::TokenStream,
  item: proc_macro::TokenStream,
) -> crate::Result<proc_macro::TokenStream> {
  let dir_ts = dir_ts(attrs)?;
  let input_fn: ItemFn = syn::parse(item)?;

  let mut has_conn = false;
  let mut has_runtime = false;
  for input in &input_fn.sig.inputs {
    if let FnArg::Typed(pat_type) = input
      && let Pat::Ident(pat_ident) = &*pat_type.pat
    {
      let name = pat_ident.ident.to_string();
      if name == "conn" {
        has_conn = true;
      } else if name == "runtime" {
        has_runtime = true;
      } else {
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
    priv_fn_args.push(quote::quote!(conn));
  }
  if has_runtime {
    priv_fn_args.push(quote::quote!(&*_runtime_clone));
  }
  let priv_fn_name = &syn::Ident::new(&format!("__{fn_name}"), fn_name.span());

  let tokens = quote::quote!(
    #[test]
    #(#fn_attrs)*
    fn #fn_name() {
      #fn_asyncness fn #priv_fn_name(#fn_inputs) #fn_output {
        #fn_block
      }

      let runtime = wtx::sync::Arc::new(wtx::executor::Runtime::new());
      let runtime_clone = runtime.clone();
      runtime.block_on(async move {
        wtx::database::client::postgres::database_test(
          #dir_ts,
          runtime_clone,
          |conn, _runtime| async move { #priv_fn_name(#(#priv_fn_args),*).await }
        )
        .await
        .unwrap();
      });
    }
  );
  Ok(tokens.into())
}

fn dir_ts(attrs: proc_macro::TokenStream) -> crate::Result<proc_macro2::TokenStream> {
  let attrs_args: Args = syn::parse(attrs)?;
  let mut dir = None;
  for arg in attrs_args.0 {
    if let Meta::List(meta_list) = arg
      && meta_list.path.is_ident("dir")
    {
      let lit: syn::LitStr = meta_list.parse_args()?;
      dir = Some(lit.value());
    }
  }
  let dir_ts = if let Some(elem) = dir { quote::quote!(Some(#elem)) } else { quote::quote!(None) };
  Ok(dir_ts)
}
