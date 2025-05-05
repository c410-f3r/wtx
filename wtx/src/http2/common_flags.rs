const ACK: u8 = 0b0000_0001;
const EOH: u8 = 0b0000_0100;
const EOS: u8 = 0b0000_0001;
const PAD: u8 = 0b0000_1000;
const PRI: u8 = 0b0010_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CommonFlags(u8);

impl CommonFlags {
  pub(crate) const fn ack() -> Self {
    Self(ACK)
  }

  pub(crate) const fn empty() -> Self {
    Self(0)
  }

  pub(crate) const fn new(byte: u8) -> Self {
    Self(byte)
  }

  pub(crate) const fn byte(self) -> u8 {
    self.0
  }

  pub(crate) const fn has_ack(self) -> bool {
    self.0 & ACK == ACK
  }

  pub(crate) const fn has_eoh(self) -> bool {
    self.0 & EOH == EOH
  }

  pub(crate) const fn has_eos(self) -> bool {
    self.0 & EOS == EOS
  }

  pub(crate) const fn has_pad(self) -> bool {
    self.0 & PAD == PAD
  }

  pub(crate) const fn has_pri(self) -> bool {
    self.0 & PRI == PRI
  }

  pub(crate) fn only_ack(&mut self) {
    self.0 &= ACK;
  }

  pub(crate) fn only_eoh_eos_pad_pri(&mut self) {
    self.0 &= EOH | EOS | PAD | PRI;
  }

  pub(crate) fn only_eos_pad(&mut self) {
    self.0 &= EOS | PAD;
  }

  pub(crate) fn set_ack(&mut self) {
    self.0 |= ACK;
  }

  pub(crate) fn set_eoh(&mut self) {
    self.0 |= EOH;
  }

  pub(crate) fn set_eos(&mut self) {
    self.0 |= EOS;
  }
}
