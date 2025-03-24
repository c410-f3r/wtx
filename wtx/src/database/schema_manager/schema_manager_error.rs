use crate::database::schema_manager::VersionTy;

/// Schema Manager Error
#[derive(Debug)]
pub enum SchemaManagerError {
  /// The `seeds` parameter must be provided through the CLI or the configuration file.
  ChecksumMustBeANumber,
  /// Databases must be sorted and unique
  DatabasesMustBeSortedAndUnique,
  /// Different rollback versions
  DifferentRollbackVersions,
  /// Divergent migrations
  DivergentMigration(VersionTy),
  /// Validation - Migrations number
  DivergentMigrationsNum {
    /// Expected
    expected: u32,
    /// Received
    received: u32,
  },
  /// Migration file has invalid syntax,
  InvalidMigration,
  /// TOML parser only supports a subset of the official TOML specification
  TomlParserOnlySupportsStringsAndArraysOfStrings,
  /// TOML parser only supports a subset of the official TOML specification
  TomlValueIsTooLarge,
  /// Migration file has an empty attribute
  IncompleteSqlFile,
}
