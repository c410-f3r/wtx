/// Client API Framework Error
#[derive(Debug)]
pub enum ClientApiFrameworkError {
  /// A slice-like batch of package is not sorted
  BatchPackagesAreNotSorted,
  /// The server closed the connection
  ClosedWsConnection,
  /// A server was not able to receive the full request data after several attempts.
  CouldNotSendTheFullRequestData,
  /// A given response id is not present in the set of sent packages.
  ResponseIdIsNotPresentInTheOfSentBatchPackages(usize),
  /// No stored test response to return a result from a request
  TestTransportNoResponse,
  /// It is not possible to convert a `u16` into a HTTP status code
  UnknownHttpStatusCode(u16),
}
