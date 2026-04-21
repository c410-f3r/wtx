/// Chain Validation - Evaluation Depth
///
/// The depth of the graph search
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CvEvaluationDepth {
  /// Search can continue up to a certain depth
  ///
  /// This variant is often more secure.
  Chain(u8),
  /// Only verifies the leaf certificate
  EndEntity,
}
