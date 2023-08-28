use crate::web_socket::DFLT_READ_BUFFER_LEN;
use alloc::{vec, vec::Vec};

/// Internal buffer used to read external data.
//
// This structure isn't strictly necessary but it tries to optimize two things:
//
// 1. Avoid syscalls by reading the maximum possible number of bytes at once.
// 2. The transposition of **payloads** of frames that compose a message into the `FrameBuffer`
//    of the same message. Frames are composed by headers and payloads as such it is necessary to
//    have some transfer strategy.
#[derive(Debug)]
pub struct ReadBuffer {
    antecedent_end_idx: usize,
    buffer: Vec<u8>,
    current_end_idx: usize,
    following_end_idx: usize,
}

impl ReadBuffer {
    pub(crate) fn with_capacity(len: usize) -> Self {
        Self {
            antecedent_end_idx: 0,
            buffer: vec![0; len],
            current_end_idx: 0,
            following_end_idx: 0,
        }
    }

    pub(crate) fn antecedent_end_idx(&self) -> usize {
        self.antecedent_end_idx
    }

    pub(crate) fn after_current_mut(&mut self) -> &mut [u8] {
        self.buffer
            .get_mut(self.current_end_idx..)
            .unwrap_or_default()
    }

    pub(crate) fn clear_if_following_is_empty(&mut self) {
        if !self.has_following() {
            self.antecedent_end_idx = 0;
            self.current_end_idx = 0;
            self.following_end_idx = 0;
        }
    }

    pub(crate) fn current(&self) -> &[u8] {
        self.buffer
            .get(self.antecedent_end_idx..self.current_end_idx)
            .unwrap_or_default()
    }

    pub(crate) fn current_mut(&mut self) -> &mut [u8] {
        self.buffer
            .get_mut(self.antecedent_end_idx..self.current_end_idx)
            .unwrap_or_default()
    }

    pub(crate) fn expand_after_current(&mut self, mut new_len: usize) {
        new_len = self.current_end_idx.wrapping_add(new_len);
        if new_len > self.buffer.len() {
            self.buffer.resize(new_len, 0);
        }
    }

    pub(crate) fn expand_buffer(&mut self, new_len: usize) {
        if new_len > self.buffer.len() {
            self.buffer.resize(new_len, 0);
        }
    }

    pub(crate) fn following_len(&self) -> usize {
        self.following_end_idx.wrapping_sub(self.current_end_idx)
    }

    pub(crate) fn has_following(&self) -> bool {
        self.following_end_idx > self.current_end_idx
    }

    pub(crate) fn merge_current_with_antecedent(&mut self) {
        self.antecedent_end_idx = self.current_end_idx;
    }

    pub(crate) fn set_indices_through_expansion(
        &mut self,
        antecedent_end_idx: usize,
        current_end_idx: usize,
        following_end_idx: usize,
    ) {
        self.antecedent_end_idx = antecedent_end_idx;
        self.current_end_idx = self.antecedent_end_idx.max(current_end_idx);
        self.following_end_idx = self.current_end_idx.max(following_end_idx);
        self.expand_buffer(self.following_end_idx);
    }
}

impl Default for ReadBuffer {
    #[inline]
    fn default() -> Self {
        Self::with_capacity(DFLT_READ_BUFFER_LEN)
    }
}
