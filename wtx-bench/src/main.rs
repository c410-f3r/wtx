//! WebSocket benchmark

#![allow(
  // Does not matter
  clippy::arithmetic_side_effects,
  // Does not matter
  clippy::indexing_slicing,
  // Does not matter
  clippy::panic,
  // Does not matter
  clippy::print_stdout,
  // Does not matter
  clippy::unwrap_used
)]

mod misc;
mod postgres;
mod web_socket;

use wtx::misc::UriRef;

#[tokio::main]
async fn main() {
  let args: Vec<_> = std::env::args().skip(1).collect();
  match args.as_slice() {
    [first, second, rest @ ..] => match first.as_str() {
      "postgres" => {
        let uri = UriRef::new(second.as_str());
        let mut diesel_async = misc::Agent { name: "diesel-async".to_owned(), result: 0 };
        let mut sqlx_postgres = misc::Agent { name: "sqlx-postgres-tokio".to_owned(), result: 0 };
        let mut tokio_postgres = misc::Agent { name: "tokio-postgres".to_owned(), result: 0 };
        let mut wtx = misc::Agent { name: "wtx-tokio".to_owned(), result: 0 };
        postgres::bench(
          &uri,
          [&mut diesel_async, &mut sqlx_postgres, &mut tokio_postgres, &mut wtx],
        )
        .await;
        misc::plot(
          &[diesel_async, sqlx_postgres, tokio_postgres, wtx],
          &postgres::caption(),
          "/tmp/wtx-postgres.png",
        );
      }
      "web-socket" => {
        let mut agents = Vec::new();
        for uri_string in [second].into_iter().chain(rest) {
          let uri = UriRef::new(uri_string.as_str());
          let mut agent = misc::Agent { name: uri.href().to_owned(), result: 0 };
          web_socket::bench(&mut agent, &uri).await;
          agents.push(agent);
        }
        misc::plot(&agents, &web_socket::caption(), "/tmp/wtx-web-socket.png");
      }
      _ => {
        panic!("Unknown benchmark target");
      }
    },
    _ => {
      panic!("Unknown benchmark target");
    }
  }
  println!("Finished!");
}
