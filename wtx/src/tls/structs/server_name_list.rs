use crate::tls::structs::server_name::ServerName;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerNameList<'any> {
  pub names: &'any [ServerName<'any>],
}
