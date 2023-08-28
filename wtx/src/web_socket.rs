//! A computer communications protocol, providing full-duplex communication channels over a single
//! TCP connection.

mod close_code;
mod frame;
mod frame_buffer;
#[cfg(feature = "web-socket-handshake")]
pub mod handshake;
mod mask;
mod op_code;
mod web_socket_error;

use crate::{
    misc::{from_utf8_ext_rslt, from_utf8_opt, CompleteErr, ExtUtf8Error, Rng},
    web_socket::close_code::CloseCode,
    ReadBuffer, Stream,
};
use alloc::vec::Vec;
use core::borrow::BorrowMut;
pub use frame::{
    Frame, FrameControlArray, FrameControlArrayMut, FrameMut, FrameMutControlArray,
    FrameMutControlArrayMut, FrameMutMut, FrameMutVec, FrameMutVecMut, FrameVec, FrameVecMut,
};
pub use frame_buffer::{
    FrameBuffer, FrameBufferControlArray, FrameBufferControlArrayMut, FrameBufferMut,
    FrameBufferVec, FrameBufferVecMut,
};
pub use mask::unmask;
pub use op_code::OpCode;
pub use web_socket_error::WebSocketError;

pub(crate) const DFLT_FRAME_BUFFER_VEC_LEN: usize = 16 * 1024;
pub(crate) const DFLT_READ_BUFFER_LEN: usize = 2 * DFLT_FRAME_BUFFER_VEC_LEN;
pub(crate) const MAX_CONTROL_FRAME_LEN: usize = MAX_HDR_LEN_USIZE + MAX_CONTROL_FRAME_PAYLOAD_LEN;
pub(crate) const MAX_CONTROL_FRAME_PAYLOAD_LEN: usize = 125;
pub(crate) const MAX_HDR_LEN_U8: u8 = 14;
pub(crate) const MAX_HDR_LEN_USIZE: usize = 14;
pub(crate) const MAX_PAYLOAD_LEN: usize = 64 * 1024 * 1024;
pub(crate) const MIN_HEADER_LEN_USIZE: usize = 2;

/// Always masks the payload before sending.
pub type WebSocketClient<RB, S> = WebSocket<RB, S, true>;
/// [WebSocketClient] with a mutable reference of [ReadBuffer].
pub type WebSocketClientMut<'rb, S> = WebSocketClient<&'rb mut ReadBuffer, S>;
/// [WebSocketClient] with an owned [ReadBuffer].
pub type WebSocketClientOwned<S> = WebSocketClient<ReadBuffer, S>;
/// Always decode the payload after receiving.
pub type WebSocketServer<RB, S> = WebSocket<RB, S, false>;
/// [WebSocketServer] with a mutable reference of [ReadBuffer].
pub type WebSocketServerMut<'rb, S> = WebSocketServer<&'rb mut ReadBuffer, S>;
/// [WebSocketServer] with an owned [ReadBuffer].
pub type WebSocketServerOwned<S> = WebSocketServer<ReadBuffer, S>;

/// WebSocket protocol implementation over an asynchronous stream.
#[derive(Debug)]
pub struct WebSocket<RB, S, const IS_CLIENT: bool> {
    auto_close: bool,
    auto_pong: bool,
    is_stream_closed: bool,
    max_payload_len: usize,
    rb: RB,
    rng: Rng,
    stream: S,
}

impl<RB, S, const IS_CLIENT: bool> WebSocket<RB, S, IS_CLIENT> {
    /// Sets whether to automatically close the connection when a close frame is received. Defaults
    /// to `true`.
    #[inline]
    pub fn set_auto_close(&mut self, auto_close: bool) {
        self.auto_close = auto_close;
    }

    /// Sets whether to automatically send a pong frame when a ping frame is received. Defaults
    /// to `true`.
    #[inline]
    pub fn set_auto_pong(&mut self, auto_pong: bool) {
        self.auto_pong = auto_pong;
    }

    /// Sets whether to automatically close the connection when a received frame payload length
    /// exceeds `max_payload_len`. Defaults to `64 * 1024 * 1024` bytes (64 MiB).
    #[inline]
    pub fn set_max_payload_len(&mut self, max_payload_len: usize) {
        self.max_payload_len = max_payload_len;
    }
}

impl<RB, S, const IS_CLIENT: bool> WebSocket<RB, S, IS_CLIENT>
where
    RB: BorrowMut<ReadBuffer>,
    S: Stream,
{
    /// Creates a new instance from a stream that supposedly has already completed the WebSocket
    /// handshake.
    #[inline]
    pub fn new(mut rb: RB, stream: S) -> Self {
        rb.borrow_mut().clear_if_following_is_empty();
        Self {
            auto_close: true,
            auto_pong: true,
            is_stream_closed: false,
            max_payload_len: MAX_PAYLOAD_LEN,
            rb,
            rng: Rng::default(),
            stream,
        }
    }

    /// Reads a frame from the stream unmasking and validating its payload.
    #[inline]
    pub async fn read_frame<'fb, B>(
        &mut self,
        fb: &'fb mut FrameBuffer<B>,
    ) -> crate::Result<Frame<&'fb mut FrameBuffer<B>, IS_CLIENT>>
    where
        B: AsMut<Vec<u8>> + AsRef<[u8]>,
    {
        let rbfi = self.do_read_frame::<true>().await?;
        Self::copy_from_rb_to_fb(CopyType::Normal, fb, self.rb.borrow(), &rbfi);
        self.rb.borrow_mut().clear_if_following_is_empty();
        Frame::from_fb(fb)
    }

    /// Collects frames and returns the completed message once all fragments have been received.
    #[inline]
    pub async fn read_msg<'fb, B>(
        &mut self,
        fb: &'fb mut FrameBuffer<B>,
    ) -> crate::Result<Frame<&'fb mut FrameBuffer<B>, IS_CLIENT>>
    where
        B: AsMut<[u8]> + AsMut<Vec<u8>> + AsRef<[u8]>,
    {
        let mut iuc_opt = None;
        let mut is_binary = true;
        let rbfi = self.do_read_frame::<false>().await?;
        if rbfi.op_code.is_continuation() {
            return Err(WebSocketError::UnexpectedMessageFrame.into());
        }
        let should_stop_at_the_first_frame = match rbfi.op_code {
            OpCode::Binary => rbfi.fin,
            OpCode::Text => {
                let range = rbfi.header_end_idx..;
                let curr_payload = self.rb.borrow().current().get(range).unwrap_or_default();
                if rbfi.fin {
                    if from_utf8_opt(curr_payload).is_none() {
                        return Err(crate::Error::InvalidUTF8);
                    }
                    true
                } else {
                    is_binary = false;
                    match from_utf8_ext_rslt(curr_payload) {
                        Err(ExtUtf8Error::Incomplete {
                            incomplete_ending_char,
                            ..
                        }) => {
                            iuc_opt = Some(incomplete_ending_char);
                            false
                        }
                        Err(ExtUtf8Error::Invalid { .. }) => {
                            return Err(crate::Error::InvalidUTF8);
                        }
                        Ok(_) => false,
                    }
                }
            }
            OpCode::Continuation | OpCode::Close | OpCode::Ping | OpCode::Pong => true,
        };
        if should_stop_at_the_first_frame {
            Self::copy_from_rb_to_fb(CopyType::Normal, fb, self.rb.borrow(), &rbfi);
            self.rb.borrow_mut().clear_if_following_is_empty();
            return Frame::from_fb(fb);
        }
        let mut total_frame_len = msg_header_placeholder::<IS_CLIENT>().into();
        Self::copy_from_rb_to_fb(
            CopyType::Msg(&mut total_frame_len),
            fb,
            self.rb.borrow(),
            &rbfi,
        );
        if is_binary {
            self.manage_read_msg_loop(fb, rbfi.op_code, &mut total_frame_len, |_| Ok(()))
                .await?;
        } else {
            self.manage_read_msg_loop(fb, rbfi.op_code, &mut total_frame_len, |payload| {
                let tail = if let Some(mut incomplete) = iuc_opt.take() {
                    let (rslt, remaining) = incomplete.complete(payload);
                    match rslt {
                        Err(CompleteErr::HasInvalidBytes) => {
                            return Err(crate::Error::InvalidUTF8);
                        }
                        Err(CompleteErr::InsufficientInput) => {
                            let _ = iuc_opt.replace(incomplete);
                            &[]
                        }
                        Ok(_) => remaining,
                    }
                } else {
                    payload
                };
                match from_utf8_ext_rslt(tail) {
                    Err(ExtUtf8Error::Incomplete {
                        incomplete_ending_char,
                        ..
                    }) => {
                        iuc_opt = Some(incomplete_ending_char);
                    }
                    Err(ExtUtf8Error::Invalid { .. }) => {
                        return Err(crate::Error::InvalidUTF8);
                    }
                    Ok(_) => {}
                }
                Ok(())
            })
            .await?;
        };
        Frame::from_fb(fb)
    }

    /// Writes a frame to the stream without masking its payload.
    #[inline]
    pub async fn write_frame<B, FB>(
        &mut self,
        frame: &mut Frame<FB, IS_CLIENT>,
    ) -> crate::Result<()>
    where
        B: AsMut<[u8]> + AsRef<[u8]>,
        FB: BorrowMut<FrameBuffer<B>>,
    {
        Self::do_write_frame(
            frame,
            &mut self.is_stream_closed,
            &mut self.rng,
            &mut self.stream,
        )
        .await
    }

    fn copy_from_rb_to_fb<B>(
        ct: CopyType<'_>,
        fb: &mut FrameBuffer<B>,
        rb: &ReadBuffer,
        rbfi: &ReadBufferFrameInfo,
    ) where
        B: AsMut<Vec<u8>>,
    {
        let current_frame = rb.current();
        let range = match ct {
            CopyType::Msg(total_frame_len) => {
                let prev = *total_frame_len;
                *total_frame_len = total_frame_len.wrapping_add(rbfi.payload_len);
                fb.set_params_through_expansion(
                    0,
                    msg_header_placeholder::<IS_CLIENT>(),
                    *total_frame_len,
                );
                prev..*total_frame_len
            }
            CopyType::Normal => {
                let mask_placeholder = if IS_CLIENT { 4 } else { 0 };
                let header_len_total = rbfi.header_len.wrapping_add(mask_placeholder);
                let header_len_total_usize = rbfi.header_len.wrapping_add(mask_placeholder).into();
                fb.set_params_through_expansion(
                    0,
                    header_len_total,
                    rbfi.payload_len.wrapping_add(header_len_total_usize),
                );
                fb.buffer_mut()
                    .as_mut()
                    .get_mut(..rbfi.header_len.into())
                    .unwrap_or_default()
                    .copy_from_slice(
                        current_frame
                            .get(rbfi.header_begin_idx..rbfi.header_end_idx)
                            .unwrap_or_default(),
                    );
                let start = header_len_total_usize;
                let end = current_frame
                    .len()
                    .wrapping_sub(rbfi.header_begin_idx)
                    .wrapping_add(mask_placeholder.into());
                start..end
            }
        };
        fb.buffer_mut()
            .as_mut()
            .get_mut(range)
            .unwrap_or_default()
            .copy_from_slice(current_frame.get(rbfi.header_end_idx..).unwrap_or_default());
    }

    #[inline]
    async fn do_read_frame<const CHECK_TEXT_UTF8: bool>(
        &mut self,
    ) -> crate::Result<ReadBufferFrameInfo> {
        loop {
            let mut rbfi = self.fill_rb_from_stream().await?;
            let curr_frame = self.rb.borrow_mut().current_mut();
            if !IS_CLIENT {
                unmask(
                    curr_frame
                        .get_mut(rbfi.header_end_idx..)
                        .unwrap_or_default(),
                    rbfi.mask.ok_or(WebSocketError::MissingFrameMask)?,
                );
                let n = remove_mask(
                    curr_frame
                        .get_mut(rbfi.header_begin_idx..rbfi.header_end_idx)
                        .unwrap_or_default(),
                );
                let n_usize = n.into();
                rbfi.frame_len = rbfi.frame_len.wrapping_sub(n_usize);
                rbfi.header_begin_idx = rbfi.header_begin_idx.wrapping_add(n_usize);
                rbfi.header_len = rbfi.header_len.wrapping_sub(n);
            }
            let payload: &[u8] = curr_frame.get(rbfi.header_end_idx..).unwrap_or_default();
            match rbfi.op_code {
                OpCode::Close if self.auto_close && !self.is_stream_closed => {
                    match payload {
                        [] => {}
                        [_] => return Err(WebSocketError::InvalidCloseFrame.into()),
                        [a, b, rest @ ..] => {
                            if from_utf8_opt(rest).is_none() {
                                return Err(crate::Error::InvalidUTF8);
                            };
                            let is_not_allowed =
                                !CloseCode::from(u16::from_be_bytes([*a, *b])).is_allowed();
                            if is_not_allowed || rest.len() > MAX_CONTROL_FRAME_PAYLOAD_LEN - 2 {
                                Self::write_control_frame(
                                    &mut FrameControlArray::close_from_params(
                                        1002,
                                        <_>::default(),
                                        rest,
                                    )?,
                                    &mut self.is_stream_closed,
                                    &mut self.rng,
                                    &mut self.stream,
                                )
                                .await?;
                                return Err(WebSocketError::InvalidCloseFrame.into());
                            }
                        }
                    }
                    Self::write_control_frame(
                        &mut FrameControlArray::new_fin(<_>::default(), OpCode::Close, payload)?,
                        &mut self.is_stream_closed,
                        &mut self.rng,
                        &mut self.stream,
                    )
                    .await?;
                    break Ok(rbfi);
                }
                OpCode::Ping if self.auto_pong => {
                    Self::write_control_frame(
                        &mut FrameControlArray::new_fin(<_>::default(), OpCode::Pong, payload)?,
                        &mut self.is_stream_closed,
                        &mut self.rng,
                        &mut self.stream,
                    )
                    .await?;
                }
                OpCode::Text => {
                    if CHECK_TEXT_UTF8 && from_utf8_opt(payload).is_none() {
                        return Err(crate::Error::InvalidUTF8);
                    }
                    break Ok(rbfi);
                }
                OpCode::Continuation
                | OpCode::Binary
                | OpCode::Close
                | OpCode::Ping
                | OpCode::Pong => {
                    break Ok(rbfi);
                }
            }
        }
    }

    async fn do_write_frame<B, FB>(
        frame: &mut Frame<FB, IS_CLIENT>,
        is_stream_closed: &mut bool,
        rng: &mut Rng,
        stream: &mut S,
    ) -> crate::Result<()>
    where
        B: AsMut<[u8]> + AsRef<[u8]>,
        FB: BorrowMut<FrameBuffer<B>>,
    {
        if IS_CLIENT {
            let mut mask_opt = None;
            if let [_, second_byte, .., a, b, c, d] = frame.fb_mut().borrow_mut().header_mut() {
                if !has_masked_frame(*second_byte) {
                    *second_byte |= 0b1000_0000;
                    let mask = rng.random_u8_4();
                    *a = mask[0];
                    *b = mask[1];
                    *c = mask[2];
                    *d = mask[3];
                    mask_opt = Some(mask);
                }
            }
            if let Some(mask) = mask_opt {
                unmask(frame.fb_mut().borrow_mut().payload_mut(), mask);
            }
        }
        if frame.op_code() == OpCode::Close {
            *is_stream_closed = true;
        }
        stream.write_all(frame.fb().borrow().frame()).await?;
        Ok(())
    }

    async fn fill_initial_rb_from_stream(
        buffer: &mut [u8],
        max_payload_len: usize,
        read: &mut usize,
        stream: &mut S,
    ) -> crate::Result<ReadBufferFrameInfo>
    where
        S: Stream,
    {
        async fn read_until<S, const LEN: usize>(
            buffer: &mut [u8],
            read: &mut usize,
            start: usize,
            stream: &mut S,
        ) -> crate::Result<[u8; LEN]>
        where
            [u8; LEN]: Default,
            S: Stream,
        {
            let until = start.wrapping_add(LEN);
            while *read < until {
                let actual_buffer = buffer.get_mut(*read..).unwrap_or_default();
                let local_read = stream.read(actual_buffer).await?;
                if local_read == 0 {
                    return Err(crate::Error::UnexpectedEOF);
                }
                *read = read.wrapping_add(local_read);
            }
            Ok(buffer
                .get(start..until)
                .and_then(|el| el.try_into().ok())
                .unwrap_or_default())
        }

        let first_two = read_until::<_, 2>(buffer, read, 0, stream).await?;

        let fin = first_two[0] & 0b1000_0000 != 0;
        let rsv1 = first_two[0] & 0b0100_0000 != 0;
        let rsv2 = first_two[0] & 0b0010_0000 != 0;
        let rsv3 = first_two[0] & 0b0001_0000 != 0;

        if rsv1 || rsv2 || rsv3 {
            return Err(WebSocketError::ReservedBitsAreNotZero.into());
        }

        let is_masked = has_masked_frame(first_two[1]);
        let length_code = first_two[1] & 0b0111_1111;
        let op_code = op_code(first_two[0])?;

        let (mut header_len, payload_len) = match length_code {
            126 => (
                4,
                u16::from_be_bytes(read_until::<_, 2>(buffer, read, 2, stream).await?).into(),
            ),
            127 => {
                let payload_len = read_until::<_, 8>(buffer, read, 2, stream).await?;
                (10, u64::from_be_bytes(payload_len).try_into()?)
            }
            _ => (2, length_code.into()),
        };

        let mut mask = None;
        if is_masked {
            mask = Some(read_until::<_, 4>(buffer, read, header_len, stream).await?);
            header_len = header_len.wrapping_add(4);
        }

        if op_code.is_control() && !fin {
            return Err(WebSocketError::UnexpectedFragmentedControlFrame.into());
        }
        if op_code == OpCode::Ping && payload_len > MAX_CONTROL_FRAME_PAYLOAD_LEN {
            return Err(WebSocketError::VeryLargeControlFrame.into());
        }
        if payload_len >= max_payload_len {
            return Err(WebSocketError::VeryLargePayload.into());
        }

        Ok(ReadBufferFrameInfo {
            fin,
            frame_len: header_len.wrapping_add(payload_len),
            header_begin_idx: 0,
            header_end_idx: header_len,
            header_len: header_len.try_into().unwrap_or_default(),
            mask,
            op_code,
            payload_len,
        })
    }

    async fn fill_rb_from_stream(&mut self) -> crate::Result<ReadBufferFrameInfo> {
        let mut read = self.rb.borrow().following_len();
        self.rb.borrow_mut().merge_current_with_antecedent();
        self.rb.borrow_mut().expand_after_current(MAX_HDR_LEN_USIZE);
        let rbfi = Self::fill_initial_rb_from_stream(
            self.rb.borrow_mut().after_current_mut(),
            self.max_payload_len,
            &mut read,
            &mut self.stream,
        )
        .await?;
        if self.is_stream_closed && rbfi.op_code != OpCode::Close {
            return Err(WebSocketError::ConnectionClosed.into());
        }
        loop {
            if read >= rbfi.frame_len {
                break;
            }
            self.rb.borrow_mut().expand_after_current(rbfi.frame_len);
            let local_read = self
                .stream
                .read(
                    self.rb
                        .borrow_mut()
                        .after_current_mut()
                        .get_mut(read..)
                        .unwrap_or_default(),
                )
                .await?;
            read = read.wrapping_add(local_read);
        }
        let rb = self.rb.borrow_mut();
        rb.set_indices_through_expansion(
            rb.antecedent_end_idx(),
            rb.antecedent_end_idx().wrapping_add(rbfi.frame_len),
            rb.antecedent_end_idx().wrapping_add(read),
        );
        Ok(rbfi)
    }

    async fn manage_read_msg_loop<B>(
        &mut self,
        fb: &mut FrameBuffer<B>,
        first_frame_op_code: OpCode,
        total_frame_len: &mut usize,
        mut cb: impl FnMut(&[u8]) -> crate::Result<()>,
    ) -> crate::Result<()>
    where
        B: AsMut<[u8]> + AsMut<Vec<u8>> + AsRef<[u8]>,
        S: Stream,
    {
        loop {
            let rbfi = self.do_read_frame::<false>().await?;
            Self::copy_from_rb_to_fb(CopyType::Msg(total_frame_len), fb, self.rb.borrow(), &rbfi);
            match rbfi.op_code {
                OpCode::Continuation => {
                    cb(self
                        .rb
                        .borrow()
                        .current()
                        .get(rbfi.header_end_idx..)
                        .unwrap_or_default())?;
                    if rbfi.fin {
                        let mut buffer = [0; MAX_HDR_LEN_USIZE];
                        let header_len = copy_header_params_to_buffer::<IS_CLIENT>(
                            &mut buffer,
                            true,
                            first_frame_op_code,
                            fb.payload().len(),
                        )?;
                        let start_idx =
                            msg_header_placeholder::<IS_CLIENT>().wrapping_sub(header_len);
                        fb.header_mut()
                            .get_mut(start_idx.into()..)
                            .unwrap_or_default()
                            .copy_from_slice(buffer.get(..header_len.into()).unwrap_or_default());
                        fb.set_params_through_expansion(start_idx, header_len, *total_frame_len);
                        self.rb.borrow_mut().clear_if_following_is_empty();
                        break;
                    }
                }
                OpCode::Binary | OpCode::Close | OpCode::Ping | OpCode::Pong | OpCode::Text => {
                    return Err(WebSocketError::UnexpectedMessageFrame.into());
                }
            }
        }
        Ok(())
    }

    async fn write_control_frame(
        frame: &mut FrameControlArray<IS_CLIENT>,
        is_stream_closed: &mut bool,
        rng: &mut Rng,
        stream: &mut S,
    ) -> crate::Result<()> {
        Self::do_write_frame(frame, is_stream_closed, rng, stream).await?;
        Ok(())
    }
}

#[derive(Debug)]
enum CopyType<'read> {
    Msg(&'read mut usize),
    Normal,
}

#[derive(Debug)]
struct ReadBufferFrameInfo {
    fin: bool,
    frame_len: usize,
    header_begin_idx: usize,
    header_end_idx: usize,
    header_len: u8,
    mask: Option<[u8; 4]>,
    op_code: OpCode,
    payload_len: usize,
}

pub(crate) fn copy_header_params_to_buffer<const IS_CLIENT: bool>(
    buffer: &mut [u8],
    fin: bool,
    op_code: OpCode,
    payload_len: usize,
) -> crate::Result<u8> {
    fn first_header_byte(fin: bool, op_code: OpCode) -> u8 {
        u8::from(fin) << 7 | u8::from(op_code)
    }

    fn manage_mask<const IS_CLIENT: bool, const N: u8>(
        rest: &mut [u8],
        second_byte: &mut u8,
    ) -> crate::Result<u8> {
        Ok(if IS_CLIENT {
            *second_byte &= 0b0111_1111;
            let [a, b, c, d, ..] = rest else {
                return Err(WebSocketError::InvalidFrameHeaderBounds.into());
            };
            *a = 0;
            *b = 0;
            *c = 0;
            *d = 0;
            N.wrapping_add(4)
        } else {
            N
        })
    }
    match payload_len {
        0..=125 => {
            if let ([a, b, rest @ ..], Ok(u8_len)) = (buffer, u8::try_from(payload_len)) {
                *a = first_header_byte(fin, op_code);
                *b = u8_len;
                return manage_mask::<IS_CLIENT, 2>(rest, b);
            }
        }
        126..=0xFFFF => {
            let rslt = u16::try_from(payload_len).map(u16::to_be_bytes);
            if let ([a, b, c, d, rest @ ..], Ok([len_c, len_d])) = (buffer, rslt) {
                *a = first_header_byte(fin, op_code);
                *b = 126;
                *c = len_c;
                *d = len_d;
                return manage_mask::<IS_CLIENT, 4>(rest, b);
            }
        }
        _ => {
            if let (
                [a, b, c, d, e, f, g, h, i, j, rest @ ..],
                Ok([len_c, len_d, len_e, len_f, len_g, len_h, len_i, len_j]),
            ) = (buffer, u64::try_from(payload_len).map(u64::to_be_bytes))
            {
                *a = first_header_byte(fin, op_code);
                *b = 127;
                *c = len_c;
                *d = len_d;
                *e = len_e;
                *f = len_f;
                *g = len_g;
                *h = len_h;
                *i = len_i;
                *j = len_j;
                return manage_mask::<IS_CLIENT, 10>(rest, b);
            }
        }
    }

    Err(WebSocketError::InvalidFrameHeaderBounds.into())
}

pub(crate) fn has_masked_frame(second_header_byte: u8) -> bool {
    second_header_byte & 0b1000_0000 != 0
}

pub(crate) fn op_code(first_header_byte: u8) -> crate::Result<OpCode> {
    OpCode::try_from(first_header_byte & 0b0000_1111)
}
const fn msg_header_placeholder<const IS_CLIENT: bool>() -> u8 {
    if IS_CLIENT {
        MAX_HDR_LEN_U8
    } else {
        MAX_HDR_LEN_U8 - 4
    }
}

fn remove_mask(header: &mut [u8]) -> u8 {
    let Some(second_header_byte) = header.get_mut(1) else {
        return 0;
    };
    if !has_masked_frame(*second_header_byte) {
        return 0;
    }
    *second_header_byte &= 0b0111_1111;
    let prev_header_len = header.len();
    let until_mask = header
        .get_mut(..prev_header_len.wrapping_sub(4))
        .unwrap_or_default();
    let mut buffer = [0u8; MAX_HDR_LEN_USIZE - 4];
    let swap_bytes = buffer.get_mut(..until_mask.len()).unwrap_or_default();
    swap_bytes.copy_from_slice(until_mask);
    let new_header = header.get_mut(4..prev_header_len).unwrap_or_default();
    new_header.copy_from_slice(swap_bytes);
    4
}
