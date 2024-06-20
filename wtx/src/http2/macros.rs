macro_rules! loop_until_some {
  ($opt:expr) => {{
    let resource = 'resource: {
      for _ in 0.._max_frames_mismatches!() {
        match $opt {
          None => continue,
          Some(elem) => break 'resource elem,
        }
      }
      return Err(crate::http2::misc::protocol_err(Http2Error::VeryLargeAmountOfFrameMismatches));
    };
    resource
  }};
}

macro_rules! process_higher_operation {
  (
    $hd:expr,
    $(@|$first_guard:ident| $first_cb:expr,)?
    |$guard:ident| $cb:expr
  ) => {
    'outer_fetch: loop {
      let mut $guard = $hd.lock().await;
      let err = 'err: {
        if let Err(err) = $guard.process_receipt().await {
          break 'err err;
        }
        match $cb {
          Err(err) => break 'err err,
          Ok(Some(elem)) => return Ok(elem),
          Ok(None) => continue 'outer_fetch,
        }
      };
      drop($guard);
      let now = crate::misc::GenericTime::now();
      let mut idx: u8 = 0;
      let mut has_reset_err = false;
      loop {
        let mut guard = $hd.lock().await;
        if idx >= crate::http2::MAX_FINAL_FETCHES {
          return crate::http2::misc::maybe_send_based_on_error(Err(err), guard.parts_mut()).await;
        }
        has_reset_err |= matches!(&err, crate::Error::Http2ErrorReset(..));
        let local_rslt = guard.process_receipt().await;
        if has_reset_err {
          if let Err(local_err) = local_rslt {
            return crate::http2::misc::maybe_send_based_on_error(
              Err(local_err),
              guard.parts_mut(),
            )
            .await;
          }
        }
        if now.elapsed().ok().map_or(true, |el| el >= crate::http2::MAX_FINAL_DURATION) {
          return crate::http2::misc::maybe_send_based_on_error(Err(err), guard.parts_mut()).await;
        }
        idx = idx.wrapping_add(1);
      }
    }
  };
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
macro_rules! max_hpack_len {
  () => {
    4_096
  };
}
macro_rules! max_concurrent_streams_num {
  () => {
    32
  };
}
macro_rules! max_headers_len {
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
macro_rules! max_recv_streams_num {
  () => {
    32
  };
}
macro_rules! read_buffer_len {
  () => {
    131_070
  };
}
