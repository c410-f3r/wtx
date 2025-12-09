#[derive(wtx_macros::FromVars)]
pub struct Foo {
  bar0: String,
  bar1: Option<String>,
  #[from_vars(parse_bar2)]
  bar2: u16,
  #[from_vars(parse_bar3)]
  bar3: Option<u32>,
}

fn parse_bar2(value: String) -> wtx::Result<u16> {
    Ok(value.parse()?)
}

fn parse_bar3(value: String) -> wtx::Result<u32> {
    Ok(value.parse()?)
}

fn main() {}