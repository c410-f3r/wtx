_create_enum! {
  /// Migration repeatability
  #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
  pub enum Repeatability<u8> {
    /// Always runs when executing a migration, regardless of the checksum
    Always = (0, "always"),
    /// When executing a migration, runs if the checksum has been changed
    OnChecksumChange = (1, "on-checksum-change")
  }
}
