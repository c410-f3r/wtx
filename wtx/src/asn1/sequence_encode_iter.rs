use crate::{
  asn1::{Asn1EncodeWrapper, Len, asn1_writer},
  codec::{Encode, EncodeWrapper, GenericCodec},
};

/// Helper that encodes elements yielded by `C`
#[derive(Debug, PartialEq)]
pub struct SequenceEncodeIter<I>(
  /// Callback
  pub I,
);

impl<E, I> SequenceEncodeIter<I>
where
  I: Iterator<Item = E>,
  E: Encode<GenericCodec<(), Asn1EncodeWrapper>>,
{
  /// The encoding of an collection object requires the injection of a tag and the guessing of
  /// its entire length for performance reasons.
  pub fn encode(
    &mut self,
    ew: &mut EncodeWrapper<'_, Asn1EncodeWrapper>,
    len_guess: Len,
    tag: u8,
  ) -> crate::Result<()> {
    ew.encode_aux.len_guess = len_guess;
    let rslt = asn1_writer(ew, ew.encode_aux.len_guess.clone(), tag, |local_ew| {
      for elem in &mut self.0 {
        elem.encode(local_ew)?;
      }
      Ok(())
    });
    ew.encode_aux.len_guess = Len::default();
    rslt
  }
}
