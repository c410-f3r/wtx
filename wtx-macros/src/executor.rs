use syn::{ItemFn, parse_macro_input};

pub(crate) fn main(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input_fn = parse_macro_input!(item as ItemFn);

  let attrs = &input_fn.attrs;
  let block = &input_fn.block;
  let sig = &input_fn.sig;

  let asyncness = &sig.asyncness;
  let inputs = &sig.inputs;
  let name = &sig.ident;
  let output = &sig.output;

  let priv_fn_args = if inputs.is_empty() { None } else { Some(quote::quote!(_runtime_clone)) };
  let priv_fn_name = &syn::Ident::new(&format!("__{name}"), name.span());

  let tokens = quote::quote!(
    #(#attrs)*
    fn main() #output {
      #asyncness fn #priv_fn_name(#inputs) #output {
        #block
      }

      let runtime = wtx::sync::Arc::new(wtx::executor::Runtime::new());
      let _runtime_clone = runtime.clone();
      runtime
        .block_on(async move {
          #priv_fn_name(#priv_fn_args).await
        })
    }
  );
  tokens.into()
}

pub(crate) fn test(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input_fn = parse_macro_input!(item as ItemFn);

  let attrs = &input_fn.attrs;
  let block = &input_fn.block;
  let sig = &input_fn.sig;

  let asyncness = &sig.asyncness;
  let inputs = &sig.inputs;
  let name = &sig.ident;
  let output = &sig.output;

  let priv_fn_args = if inputs.is_empty() { None } else { Some(quote::quote!(&*_runtime_clone)) };
  let priv_fn_name = &syn::Ident::new(&format!("__{name}"), name.span());

  let tokens = quote::quote!(
    #[test]
    #(#attrs)*
    fn #name() {
      #asyncness fn #priv_fn_name(#inputs) #output {
        #block
      }

      let runtime = wtx::sync::Arc::new(wtx::executor::Runtime::new());
      let _runtime_clone = runtime.clone();
      runtime
        .block_on(async move {
          #priv_fn_name(#priv_fn_args).await
        })
        .unwrap();
    }
  );
  tokens.into()
}
