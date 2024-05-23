macro_rules! loop_until_some {
  ($opt:expr) => {{
    let resource = 'resource: {
      for _ in 0.._max_frames_mismatches!() {
        match $opt {
          None => continue,
          Some(elem) => break 'resource elem,
        }
      }
      return Err(crate::Error::http2_go_away_generic(
        Http2Error::VeryLargeAmountOfFrameMismatches,
      ));
    };
    resource
  }};
}

macro_rules! process_receipt_loop {
  ($hd:expr, |$guard:ident| $cb:expr) => {
    loop {
      let mut $guard = $hd.lock().await;
      let _opt = $guard.process_receipt().await?;
      $cb
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
