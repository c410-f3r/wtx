macro_rules! read_continuation_frames {
  (
    $first_rfi:expr,
    $max_payload_len:expr,
    ($nc_is_noop:expr, $nc_rsv1:expr),
    $network_buffer:expr,
    $no_masking:expr,
    $reader_buffer_first:expr,
    $reader_buffer_second:expr,
    $stream:expr,
    ($stream_reader_expr:expr, $stream_writer_expr:expr),
    ($first_text_cb:expr, $recurrent_text_cb:expr),
    $reader_buffer_first_cb:expr
  ) => {
    'rcf_block: {
      use crate::web_socket::web_socket_reader;
      web_socket_reader::copy_from_arbitrary_nb_to_rb1::<IS_CLIENT>(
        $network_buffer,
        $no_masking,
        $reader_buffer_first,
        $first_rfi,
      )?;
      let mut iuc = web_socket_reader::manage_op_code_of_first_continuation_frame(
        $first_rfi.op_code,
        $reader_buffer_first,
        $first_text_cb,
      )?;
      loop {
        let mut rfi = web_socket_reader::fetch_frame_from_stream::<_, IS_CLIENT>(
          $max_payload_len,
          ($nc_is_noop, $nc_rsv1),
          $network_buffer,
          $no_masking,
          $stream_reader_expr,
        )
        .await?;
        let begin = $reader_buffer_first.len();
        rfi.should_decompress = $first_rfi.should_decompress;
        web_socket_reader::copy_from_arbitrary_nb_to_rb1::<IS_CLIENT>(
          $network_buffer,
          $no_masking,
          $reader_buffer_first,
          &rfi,
        )?;
        let payload = $reader_buffer_first.get_mut(begin..).unwrap_or_default();
        let WebSocketCommonPart { connection_state, nc, rng, stream } = $stream_writer_expr;
        if !web_socket_reader::manage_auto_reply::<_, _, IS_CLIENT>(
          stream,
          connection_state.lease_mut(),
          $no_masking,
          rfi.op_code,
          payload,
          rng,
          &mut web_socket_reader::write_control_frame_cb,
        )
        .await?
        {
          $reader_buffer_first.truncate(begin);
          continue;
        }
        if web_socket_reader::manage_op_code_of_continuation_frames(
          rfi.fin,
          $first_rfi.op_code,
          &mut iuc,
          rfi.op_code,
          payload,
          $recurrent_text_cb,
        )? {
          let cb: fn(_, _, _, _) -> crate::Result<()> = $reader_buffer_first_cb;
          cb($first_rfi, nc, $reader_buffer_first, $reader_buffer_second)?;
          break 'rcf_block;
        }
      }
    }
  };
}

macro_rules! read_frame {
  (
    $max_payload_len:expr,
    ($nc_is_noop:expr, $nc_rsv1:expr),
    $network_buffer:expr,
    $no_masking:expr,
    $reader_buffer_first:expr,
    $reader_buffer_second:expr,
    $stream:ident,
    ($stream_reader_expr:expr, $stream_writer_expr:expr)
  ) => {
    'rffs_block: {
      use crate::web_socket::web_socket_reader;
      let first_rfi = loop {
        $reader_buffer_first.clear();
        let rfi = web_socket_reader::fetch_frame_from_stream::<_, IS_CLIENT>(
          $max_payload_len,
          ($nc_is_noop, $nc_rsv1),
          $network_buffer,
          $no_masking,
          $stream_reader_expr,
        )
        .await?;
        if !rfi.fin {
          break rfi;
        }
        let WebSocketCommonPart { connection_state, nc, rng, stream } = $stream_writer_expr;
        let payload = if rfi.should_decompress {
          web_socket_reader::copy_from_compressed_nb_to_rb1::<NC, IS_CLIENT>(
            nc,
            $network_buffer,
            $no_masking,
            $reader_buffer_first,
            &rfi,
          )?;
          $reader_buffer_first.as_slice_mut()
        } else {
          let current_mut = $network_buffer.current_mut();
          web_socket_reader::unmask_nb::<IS_CLIENT>(current_mut, $no_masking, &rfi)?;
          current_mut
        };
        if web_socket_reader::manage_auto_reply::<_, _, IS_CLIENT>(
          stream,
          connection_state.lease_mut(),
          $no_masking,
          rfi.op_code,
          payload,
          rng,
          &mut web_socket_reader::write_control_frame_cb,
        )
        .await?
        {
          web_socket_reader::manage_op_code_of_first_final_frame(rfi.op_code, payload)?;
          // FIXME(STABLE): Use `payload` with polonius
          let borrow_checker = if rfi.should_decompress {
            $reader_buffer_first.as_slice_mut()
          } else {
            $network_buffer.current_mut()
          };
          break 'rffs_block Frame::new(true, rfi.op_code, borrow_checker, $nc_rsv1);
        }
      };
      $reader_buffer_second.clear();
      if first_rfi.should_decompress {
        read_continuation_frames!(
          &first_rfi,
          $max_payload_len,
          ($nc_is_noop, $nc_rsv1),
          $network_buffer,
          $no_masking,
          $reader_buffer_first,
          $reader_buffer_second,
          $stream,
          ($stream_reader_expr, $stream_writer_expr),
          (|_| Ok(None), |_, _| Ok(())),
          |local_first_rfi, local_nc, local_rbf, local_rbs| {
            web_socket_reader::copy_from_compressed_rb1_to_rb2(
              local_first_rfi,
              local_nc,
              local_rbf,
              local_rbs,
            )
          }
        );
        Frame::new(true, first_rfi.op_code, $reader_buffer_second, $nc_rsv1)
      } else {
        read_continuation_frames!(
          &first_rfi,
          $max_payload_len,
          ($nc_is_noop, $nc_rsv1),
          $network_buffer,
          $no_masking,
          $reader_buffer_first,
          $reader_buffer_second,
          $stream,
          ($stream_reader_expr, $stream_writer_expr),
          (
            web_socket_reader::manage_text_of_first_continuation_frame,
            web_socket_reader::manage_text_of_recurrent_continuation_frames
          ),
          |_, _, _, _| Ok(())
        );
        Frame::new(true, first_rfi.op_code, $reader_buffer_first, $nc_rsv1)
      }
    }
  };
}
