#[derive(Clone, Copy, Debug)]
pub(crate) struct Metadata<M> {
  pub(crate) begin: usize,
  pub(crate) len: usize,
  pub(crate) misc: M,
}
