#[wtx::pkg]
mod a {
}

#[wtx::pkg]
mod b {
  #[pkg::req_data]
  struct Req(
    i32
  );
}

#[wtx::pkg]
mod c {
  #[pkg::res_data]
  struct Res;
}

fn main() {
}
