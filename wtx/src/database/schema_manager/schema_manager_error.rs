use crate::database::schema_manager::Uid;

/// Schema Manager Error
#[derive(Debug)]
pub enum SchemaManagerError {
  /// The `seeds` parameter must be provided through the CLI or the configuration file.
  ChecksumMustBeANumber,
  /// Databases must be sorted and unique
  DatabasesMustBeSortedAndUnique,
  /// Different rollback user IDs
  DifferentRollbackUids,
  /// The group version of the migration group is older than the current supported version.
  DivergentGroupVersions(u32, u32),
  /// Divergent migrations
  DivergentMigration(Uid),
  /// Validation - Migrations number
  DivergentMigrationsNum {
    /// Expected
    expected: u32,
    /// Received
    received: u32,
  },
  /// Migration file has invalid syntax,
  InvalidMigration,
  /// No seed directory was specified in the configuration file
  NoSeedDir,
  /// TOML parser only supports a subset of the official TOML specification
  TomlParserOnlySupportsStringsAndArraysOfStrings,
  /// TOML parser only supports a subset of the official TOML specification
  TomlValueIsTooLarge,
  /// Migration file has an empty attribute
  IncompleteSqlFile,
}
