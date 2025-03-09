use crate::misc::Vector;

#[test]
fn compiles() {
  create_packages_aux_wrapper!();
  let _pkg = PkgsAux::from_minimum((), (), ());
  let _pkg = PkgsAux::new((), Vector::new(), (), ());
}
