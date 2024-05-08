macro_rules! hre_resource_or_return {
  ($rfr:expr) => {
    match $rfr {
      Http2RsltExt::ClosedConnection => return Ok(Http2RsltExt::ClosedConnection),
      Http2RsltExt::ClosedStream => return Ok(Http2RsltExt::ClosedStream),
      Http2RsltExt::Idle => return Ok(Http2RsltExt::Idle),
      Http2RsltExt::Resource(elem) => elem,
    }
  };
}

macro_rules! hre_to_hr {
  ($lock:expr, |$guard:ident| $cb:expr $(, |$another_guard:ident, $rslt:ident| $rest:expr)?) => {{
    let rslt = loop {
      let mut $guard = $lock.lock().await;
      match $cb {
        Http2RsltExt::ClosedConnection => return Ok(Http2Rslt::ClosedConnection),
        Http2RsltExt::ClosedStream => return Ok(Http2Rslt::ClosedStream),
        Http2RsltExt::Resource(elem) => {
          $(
            let mut $another_guard = $guard;
            let $rslt = elem;
            $rest;
            let elem = $rslt;
          )?
          break elem;
        },
        Http2RsltExt::Idle => continue,
      }
    };
    rslt
  }};
}

macro_rules! hre_until_resource {
  ($rfr:expr) => {{
    let rfr_resource = 'rfr_resource: {
      for _ in 0.._max_frames_mismatches!() {
        match $rfr {
          Http2RsltExt::ClosedConnection => return Ok(Http2RsltExt::ClosedConnection),
          Http2RsltExt::ClosedStream => return Ok(Http2RsltExt::ClosedStream),
          Http2RsltExt::Idle => continue,
          Http2RsltExt::Resource(elem) => break 'rfr_resource elem,
        }
      }
      return Err(crate::Error::VeryLargeAmountOfFrameMismatches);
    };
    rfr_resource
  }};
}

macro_rules! initial_window_len {
  () => {
    65_535
  };
}
macro_rules! max_body_len {
  () => {
    131_070
  };
}
macro_rules! max_buffered_frames_num {
  () => {
    16
  };
}
macro_rules! max_cached_headers_len {
  () => {
    4_096
  };
}
macro_rules! max_expanded_headers_len {
  () => {
    4_096
  };
}
macro_rules! max_frame_len {
  () => {
    16_384
  };
}
macro_rules! max_frame_len_lower_bound {
  () => {
    16_384
  };
}
macro_rules! max_frame_len_upper_bound {
  () => {
    16_777_215
  };
}
macro_rules! max_rapid_resets_num {
  () => {
    16
  };
}
macro_rules! max_streams_num {
  () => {
    16
  };
}
macro_rules! read_buffer_len {
  () => {
    131_070
  };
}
