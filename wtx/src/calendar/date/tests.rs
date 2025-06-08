use crate::calendar::{CeDays, DAYS_PER_QUADCENTURY, Date, DayOfYear, Duration, Weekday, Year};

fn _0401_03_02() -> Date {
  Date::from_ce_days(CeDays::from_num(DAYS_PER_QUADCENTURY.cast_signed() + 59 + 2).unwrap())
    .unwrap()
}

fn _2025_04_20() -> Date {
  Date::new(Year::from_num(2025).unwrap(), DayOfYear::from_num(110).unwrap()).unwrap()
}

#[test]
fn ce_days() {
  assert_eq!(Date::CE.ce_days(), 1);
  assert_eq!(Date::MIN.ce_days(), -11968265);
  assert_eq!(Date::MAX.ce_days(), 11967535);
  assert_eq!(_0401_03_02().ce_days(), DAYS_PER_QUADCENTURY.cast_signed() + 59 + 2);
  assert_eq!(_2025_04_20().ce_days(), 739361);
}

#[test]
fn constructors_converge() {
  assert_eq!(
    Date::new(Year::from_num(500).unwrap(), DayOfYear::from_num(104).unwrap()).unwrap(),
    Date::from_ce_days(CeDays::from_num(182360).unwrap()).unwrap()
  );
}

#[test]
fn add_and_sub() {
  macro_rules! test {
    ($lhs:expr, $rhs:expr, $rslt:expr) => {
      assert_eq!($lhs.add($rhs).unwrap(), $rslt);
      assert_eq!($lhs.sub($rhs.neg()).unwrap(), $rslt);
    };
  }

  test!(instance(2014, 1, 1), Duration::ZERO, instance(2014, 1, 1));
  test!(instance(2014, 1, 1), Duration::from_seconds(86399).unwrap(), instance(2014, 1, 1));
  test!(instance(2014, 1, 1), Duration::from_seconds(-86399).unwrap(), instance(2014, 1, 1));
  test!(instance(2014, 1, 1), Duration::from_days(1).unwrap(), instance(2014, 1, 2));
  test!(instance(2014, 1, 1), Duration::from_days(-1).unwrap(), instance(2013, 12, 31));
  test!(instance(2014, 1, 1), Duration::from_days(364).unwrap(), instance(2014, 12, 31));
  test!(instance(2014, 1, 1), Duration::from_days(365 * 4 + 1).unwrap(), instance(2018, 1, 1));
  test!(instance(2014, 1, 1), Duration::from_days(365 * 400 + 97).unwrap(), instance(2414, 1, 1));
  test!(instance(-7, 1, 1), Duration::from_days(365 * 12 + 3).unwrap(), instance(5, 1, 1));
}

#[test]
fn day() {
  assert_eq!(Date::MIN.day().num(), 1);
  assert_eq!(Date::MAX.day().num(), 31);
  assert_eq!(_0401_03_02().day().num(), 2);
  assert_eq!(_2025_04_20().day().num(), 20);
}

#[test]
fn day_of_year() {
  assert_eq!(Date::MIN.day_of_year().num(), 1);
  assert_eq!(Date::MAX.day_of_year().num(), 365);
  assert_eq!(_0401_03_02().day_of_year().num(), 61);
  assert_eq!(_2025_04_20().day_of_year().num(), 110);
}

#[test]
fn iso_8601() {
  assert_eq!(Date::MIN.iso_8601().as_str(), "-32767-01-01");
  assert_eq!(Date::MAX.iso_8601().as_str(), "32766-12-31");
  assert_eq!(_0401_03_02().iso_8601().as_str(), "401-03-02");
  assert_eq!(_2025_04_20().iso_8601().as_str(), "2025-04-20");
}

#[test]
fn month() {
  assert_eq!(Date::MIN.month().num(), 1);
  assert_eq!(Date::MAX.month().num(), 12);
  assert_eq!(_0401_03_02().month().num(), 3);
  assert_eq!(_2025_04_20().month().num(), 4);
}

#[test]
fn weekday() {
  assert_eq!(Date::from_ce_days((-9).try_into().unwrap()).unwrap().weekday(), Weekday::Friday);
  assert_eq!(Date::from_ce_days((-8).try_into().unwrap()).unwrap().weekday(), Weekday::Saturday);
  assert_eq!(Date::from_ce_days((-7).try_into().unwrap()).unwrap().weekday(), Weekday::Sunday);
  assert_eq!(Date::from_ce_days((-6).try_into().unwrap()).unwrap().weekday(), Weekday::Monday);
  assert_eq!(Date::from_ce_days((-5).try_into().unwrap()).unwrap().weekday(), Weekday::Tuesday);
  assert_eq!(Date::from_ce_days((-4).try_into().unwrap()).unwrap().weekday(), Weekday::Wednesday);
  assert_eq!(Date::from_ce_days((-3).try_into().unwrap()).unwrap().weekday(), Weekday::Thursday);
  assert_eq!(Date::from_ce_days((-2).try_into().unwrap()).unwrap().weekday(), Weekday::Friday);
  assert_eq!(Date::from_ce_days((-1).try_into().unwrap()).unwrap().weekday(), Weekday::Saturday);
  assert_eq!(Date::from_ce_days(0.try_into().unwrap()).unwrap().weekday(), Weekday::Sunday);
  assert_eq!(Date::from_ce_days(1.try_into().unwrap()).unwrap().weekday(), Weekday::Monday);
  assert_eq!(Date::from_ce_days(2.try_into().unwrap()).unwrap().weekday(), Weekday::Tuesday);
  assert_eq!(Date::from_ce_days(3.try_into().unwrap()).unwrap().weekday(), Weekday::Wednesday);
  assert_eq!(Date::from_ce_days(4.try_into().unwrap()).unwrap().weekday(), Weekday::Thursday);
  assert_eq!(Date::from_ce_days(5.try_into().unwrap()).unwrap().weekday(), Weekday::Friday);
  assert_eq!(Date::from_ce_days(6.try_into().unwrap()).unwrap().weekday(), Weekday::Saturday);
  assert_eq!(Date::from_ce_days(7.try_into().unwrap()).unwrap().weekday(), Weekday::Sunday);
  assert_eq!(Date::from_ce_days(8.try_into().unwrap()).unwrap().weekday(), Weekday::Monday);
  assert_eq!(Date::from_ce_days(9.try_into().unwrap()).unwrap().weekday(), Weekday::Tuesday);
}

#[test]
fn year() {
  assert_eq!(Date::MIN.year().num(), -32767);
  assert_eq!(Date::MAX.year().num(), 32766);
  assert_eq!(_0401_03_02().year().num(), 401);
  assert_eq!(_2025_04_20().year().num(), 2025);
}

fn instance(y: i16, m: u8, d: u8) -> Date {
  Date::from_ymd(y.try_into().unwrap(), m.try_into().unwrap(), d.try_into().unwrap()).unwrap()
}
