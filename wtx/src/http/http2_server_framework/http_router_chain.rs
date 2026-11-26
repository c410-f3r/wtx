use crate::{
  collections::{ArrayVectorCopy, DynVectorU7},
  http::http2_server_framework::http_router_node::HttpRouterNode,
};

/// A chain that leads to an endpoint
#[derive(Clone, Debug)]
pub struct HttpRouterChain {
  leafs: DynVectorU7<HttpRouterNode, 1>,
  steps: ArrayVectorCopy<HttpRouterNode, 4>,
}

impl HttpRouterChain {
  pub(crate) fn new(
    leafs: DynVectorU7<HttpRouterNode, 1>,
    steps: ArrayVectorCopy<HttpRouterNode, 4>,
  ) -> Self {
    Self { leafs, steps }
  }

  pub(crate) fn leafs(&self) -> &[HttpRouterNode] {
    &self.leafs
  }

  pub(crate) fn leafs_mut(&mut self) -> &mut DynVectorU7<HttpRouterNode, 1> {
    &mut self.leafs
  }

  pub(crate) fn steps(&self) -> &ArrayVectorCopy<HttpRouterNode, 4> {
    &self.steps
  }
}
