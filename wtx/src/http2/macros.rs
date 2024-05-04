macro_rules! rfr_resource_or_return {
  ($rfr:expr) => {
    match $rfr {
      ReadFrameRslt::ClosedConnection => return Ok(ReadFrameRslt::ClosedConnection),
      ReadFrameRslt::ClosedStream => return Ok(ReadFrameRslt::ClosedStream),
      ReadFrameRslt::IdleConnection => return Ok(ReadFrameRslt::IdleConnection),
      ReadFrameRslt::Resource(elem) => elem,
    }
  };
}

macro_rules! rfr_until_resource {
  ($rfr:expr) => {{
    let rfr_resource = 'rfr_resource: {
      for _ in 0.._max_frames_mismatches!() {
        match $rfr {
          ReadFrameRslt::ClosedConnection => return Ok(ReadFrameRslt::ClosedConnection),
          ReadFrameRslt::ClosedStream => return Ok(ReadFrameRslt::ClosedStream),
          ReadFrameRslt::IdleConnection => continue,
          ReadFrameRslt::Resource(elem) => break 'rfr_resource elem,
        }
      }
      return Err(crate::Error::VeryLargeAmountOfFrameMismatches);
    };
    rfr_resource
  }};
}

macro_rules! rfr_until_resource_with_guard {
  ($lock:expr, |$guard:ident| $cb:expr $(, |$another_guard:ident, $rslt:ident| $rest:expr)?) => {{
    let rfr_resource = 'rfr_resource: {
      for _ in 0.._max_frames_mismatches!() {
        let mut $guard = $lock.lock().await;
        match $cb {
          ReadFrameRslt::ClosedConnection => return Ok(ReadFrameRslt::ClosedConnection),
          ReadFrameRslt::ClosedStream => return Ok(ReadFrameRslt::ClosedStream),
          ReadFrameRslt::IdleConnection => continue,
          ReadFrameRslt::Resource(elem) => {
            $(
              let mut $another_guard = $guard;
              let $rslt = elem;
              $rest;
              let elem = $rslt;
            )?
            break 'rfr_resource elem;
          }
        }
      }
      return Err(crate::Error::VeryLargeAmountOfFrameMismatches);
    };
    rfr_resource
  }};
}
