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
      let _runtime_clone = runtime.clone();
      runtime
        .block_on(async move {
          use wtx::{database::Executor as _, rng::SeedableRng as _};

          #[derive(wtx::FromVars)]
          struct LocalVars {
            database_uri: String,
          }

          let local_vars: LocalVars = wtx::misc::EnvVars::from_available().unwrap().finish();
          let uri = local_vars.database_uri.as_str().into();

          let mut config = wtx::database::client::postgres::Config::from_uri(&uri).unwrap();
          let mut db_name = String::new();
          db_name.push('_');
          db_name.push_str(wtx::misc::timestamp_nanos_str().unwrap().1.as_str());
          let mut rng = wtx::rng::ChaCha20::from_seed(wtx::rng::simple_32_seed()).unwrap();

          let orig_db = String::from(config.db());

          {
            let mut conn = wtx::database::client::postgres::PostgresExecutor::<wtx::Error, _, _>::connect(
              &wtx::database::client::postgres::Config::from_uri(&uri).unwrap(),
              wtx::database::client::postgres::ExecutorBuffer::new(100, &mut rng),
              &mut rng,
              std::net::TcpStream::connect(uri.hostname_with_implied_port()).unwrap(),
            )
            .await
            .unwrap();
            let mut create_db_query = String::new();
            create_db_query.push_str("CREATE DATABASE ");
            create_db_query.push_str(&db_name);
            conn.execute_ignored(create_db_query.as_str()).await.unwrap();
          }

          {
            config.set_db(db_name.as_str());
            let mut conn = wtx::database::client::postgres::PostgresExecutor::connect(
              &config,
              wtx::database::client::postgres::ExecutorBuffer::new(100, &mut rng),
              &mut rng,
              std::net::TcpStream::connect(uri.hostname_with_implied_port()).unwrap(),
            )
            .await
            .unwrap();
            wtx::database::schema_manager::Commands::new(8, &mut conn)
              .clear_migrate_and_seed(#dir_ts)
              .await
              .unwrap();
            #priv_fn_name(#(#priv_fn_args),*).await
          }

          {
            config.set_db(orig_db.as_str());
            let mut conn = wtx::database::client::postgres::PostgresExecutor::<wtx::Error, _, _>::connect(
              &config,
              wtx::database::client::postgres::ExecutorBuffer::new(100, &mut rng),
              &mut rng,
              std::net::TcpStream::connect(uri.hostname_with_implied_port()).unwrap(),
            )
            .await
            .unwrap();
            let mut drop_db_query = String::new();
            drop_db_query.push_str("DROP DATABASE ");
            drop_db_query.push_str(&db_name);
            conn.execute_ignored(drop_db_query.as_str()).await.unwrap();
          }
        })
        .unwrap();
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
