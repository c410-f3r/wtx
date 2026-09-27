use rust_decimal::Decimal;

use crate::{
  calendar::{Datetime, Time, Utc},
  fixed_point::IPpm32,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Data<S> {
  /// Boolean
  Boolean(bool),
  /// Currency,
  Currency(Decimal),
  /// Date or Time
  Datetime(Datetime<Utc>),
  /// Empty cell
  #[default]
  Empty,
  /// Float
  Float(f64),
  /// Percentage
  Percentage(IPpm32),
  /// String
  String(S),
  /// Time
  Time(Time),
  /// Unknown
  Unknown,
}
