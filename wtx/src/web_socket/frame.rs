use crate::{
    misc::{from_utf8_opt, Expand, SingleTypeStorage},
    web_socket::{
        copy_header_params_to_buffer,
        frame_buffer::{
            FrameBufferControlArray, FrameBufferControlArrayMut, FrameBufferMut, FrameBufferVecMut,
        },
        op_code, FrameBuffer, FrameBufferVec, OpCode, WebSocketError,
        MAX_CONTROL_FRAME_PAYLOAD_LEN, MAX_HDR_LEN_USIZE, MIN_HEADER_LEN_USIZE,
    },
};
use core::{
    borrow::{Borrow, BorrowMut},
    str,
};

/// Composed by a [FrameBufferControlArray].
pub type FrameControlArray<const IS_CLIENT: bool> = Frame<FrameBufferControlArray, IS_CLIENT>;
/// Composed by a [FrameBufferControlArrayMut].
pub type FrameControlArrayMut<'bytes, const IS_CLIENT: bool> =
    Frame<FrameBufferControlArrayMut<'bytes>, IS_CLIENT>;
/// Composed by a [FrameBufferMut].
pub type FrameMut<'bytes, const IS_CLIENT: bool> = Frame<FrameBufferMut<'bytes>, IS_CLIENT>;
/// Composed by a [FrameBufferVec].
pub type FrameVec<const IS_CLIENT: bool> = Frame<FrameBufferVec, IS_CLIENT>;
/// Composed by an mutable [FrameBufferVecMut] reference.
pub type FrameVecMut<'bytes, const IS_CLIENT: bool> = Frame<FrameBufferVecMut<'bytes>, IS_CLIENT>;

/// Composed by an mutable [FrameBufferControlArray] reference.
pub type FrameMutControlArray<'fb, const IS_CLIENT: bool> =
    Frame<&'fb mut FrameBufferControlArray, IS_CLIENT>;
/// Composed by an mutable [FrameBufferControlArrayMut] reference.
pub type FrameMutControlArrayMut<'fb, const IS_CLIENT: bool> =
    Frame<&'fb mut FrameBufferControlArray, IS_CLIENT>;
/// Composed by an mutable [FrameBufferMut] reference.
pub type FrameMutMut<'bytes, 'fb, const IS_CLIENT: bool> =
    Frame<&'fb mut FrameBufferMut<'bytes>, IS_CLIENT>;
/// Composed by an mutable [FrameBufferVec] reference.
pub type FrameMutVec<'fb, const IS_CLIENT: bool> = Frame<&'fb mut FrameBufferVec, IS_CLIENT>;
/// Composed by an mutable [FrameBufferVecMut] reference.
pub type FrameMutVecMut<'bytes, 'fb, const IS_CLIENT: bool> =
    Frame<&'fb mut FrameBufferVecMut<'bytes>, IS_CLIENT>;

/// Represents a WebSocket frame
#[derive(Debug)]
pub struct Frame<FB, const IS_CLIENT: bool> {
    fin: bool,
    op_code: OpCode,
    fb: FB,
}

impl<FB, const IS_CLIENT: bool> Frame<FB, IS_CLIENT> {
    /// Contains the raw bytes that compose this frame.
    #[inline]
    pub fn fb(&self) -> &FB {
        &self.fb
    }

    pub(crate) fn fb_mut(&mut self) -> &mut FB {
        &mut self.fb
    }

    /// Indicates if this is the final frame in a message.
    #[inline]
    pub fn fin(&self) -> bool {
        self.fin
    }

    /// See [OpCode].
    #[inline]
    pub fn op_code(&self) -> OpCode {
        self.op_code
    }
}

impl<B, FB, const IS_CLIENT: bool> Frame<FB, IS_CLIENT>
where
    B: AsRef<[u8]>,
    FB: Borrow<FrameBuffer<B>> + SingleTypeStorage<Item = B>,
{
    /// Creates a new instance based on the contained bytes of `fb`.
    #[inline]
    pub fn from_fb(fb: FB) -> crate::Result<Self> {
        let header = fb.borrow().header();
        let len = header.len();
        let has_valid_header = (MIN_HEADER_LEN_USIZE..=MAX_HDR_LEN_USIZE).contains(&len);
        let (true, Some(first_header_byte)) = (has_valid_header, header.first().copied()) else {
            return Err(WebSocketError::InvalidFrameHeaderBounds.into());
        };
        Ok(Self {
            fb,
            fin: first_header_byte & 0b1000_0000 != 0,
            op_code: op_code(first_header_byte)?,
        })
    }

    /// Checks if the frame payload is valid UTF-8, regardless of its type.
    #[inline]
    pub fn is_utf8(&self) -> bool {
        self.op_code.is_text() || from_utf8_opt(self.fb.borrow().payload()).is_some()
    }

    /// If the frame is of type [OpCode::Text], returns its payload interpreted as a string.
    #[inline]
    pub fn text_payload<'this>(&'this self) -> Option<&'this str>
    where
        B: 'this,
    {
        self.op_code.is_text().then(|| {
            #[allow(unsafe_code)]
            // SAFETY: All text frames have valid UTF-8 contents when read.
            unsafe {
                str::from_utf8_unchecked(self.fb.borrow().payload())
            }
        })
    }
}

impl<B, FB, const IS_CLIENT: bool> Frame<FB, IS_CLIENT>
where
    B: AsMut<[u8]> + AsRef<[u8]> + Expand,
    FB: BorrowMut<FrameBuffer<B>> + SingleTypeStorage<Item = B>,
{
    /// Creates based on the individual parameters that compose a close frame.
    ///
    /// `reason` is capped based on the maximum allowed size of a control frame minus 2.
    #[inline]
    pub fn close_from_params(code: u16, fb: FB, reason: &[u8]) -> crate::Result<Self> {
        let reason_len = reason.len().min(MAX_CONTROL_FRAME_PAYLOAD_LEN - 2);
        let payload_len = reason_len.wrapping_add(2);
        Self::build_frame(fb, true, OpCode::Close, payload_len, |local_fb| {
            let payload = local_fb.borrow_mut().payload_mut();
            payload
                .get_mut(..2)
                .unwrap_or_default()
                .copy_from_slice(&code.to_be_bytes());
            payload
                .get_mut(2..)
                .unwrap_or_default()
                .copy_from_slice(reason.get(..reason_len).unwrap_or_default());
            Ok(())
        })
    }

    /// Creates a new instance that is considered final.
    #[inline]
    pub fn new_fin(fb: FB, op_code: OpCode, payload: &[u8]) -> crate::Result<Self> {
        Self::new(fb, true, op_code, payload)
    }

    /// Creates a new instance that is meant to be a continuation of previous frames.
    #[inline]
    pub fn new_unfin(fb: FB, op_code: OpCode, payload: &[u8]) -> crate::Result<Self> {
        Self::new(fb, false, op_code, payload)
    }

    fn build_frame(
        mut fb: FB,
        fin: bool,
        op_code: OpCode,
        payload_len: usize,
        cb: impl FnOnce(&mut FB) -> crate::Result<()>,
    ) -> crate::Result<Self> {
        fb.borrow_mut().clear();
        fb.borrow_mut()
            .buffer_mut()
            .expand(MAX_HDR_LEN_USIZE.saturating_add(payload_len));
        let n = copy_header_params_to_buffer::<IS_CLIENT>(
            fb.borrow_mut().buffer_mut().as_mut(),
            fin,
            op_code,
            payload_len,
        )?;
        fb.borrow_mut().set_header_indcs(0, n)?;
        fb.borrow_mut().set_payload_len(payload_len)?;
        cb(&mut fb)?;
        Ok(Self { fin, op_code, fb })
    }

    fn new(fb: FB, fin: bool, op_code: OpCode, payload: &[u8]) -> crate::Result<Self> {
        let payload_len = if op_code.is_control() {
            payload.len().min(MAX_CONTROL_FRAME_PAYLOAD_LEN)
        } else {
            payload.len()
        };
        Self::build_frame(fb, fin, op_code, payload_len, |local_fb| {
            local_fb
                .borrow_mut()
                .payload_mut()
                .copy_from_slice(payload.get(..payload_len).unwrap_or_default());
            Ok(())
        })
    }
}
