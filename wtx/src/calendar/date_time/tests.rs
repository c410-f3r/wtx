use crate::calendar::{
  Date, DateTime, DayOfYear, Duration, Hour, Minute, Nanosecond, Second, Time, Year,
};

fn _2025_04_20_14_20_30_1234() -> DateTime {
  DateTime::new(
    Date::new(Year::from_num(2025).unwrap(), DayOfYear::from_num(110).unwrap()).unwrap(),
    Time::from_hms_ns(Hour::N14, Minute::N20, Second::N30, Nanosecond::from_num(1234).unwrap()),
  )
}

#[test]
fn add_and_sub() {
  macro_rules! test {
    ($lhs:expr, $rhs:expr, $rslt:expr) => {
      assert_eq!($lhs.add($rhs).unwrap(), $rslt);
      assert_eq!($lhs.sub($rhs.neg()).unwrap(), $rslt);
    };
  }

  test!(
    instance(2014, 5, 6, 7, 8, 9, 0),
    Duration::from_seconds(3600 + 60 + 1).unwrap(),
    instance(2014, 5, 6, 8, 9, 10, 0)
  );
  test!(
    instance(2014, 5, 6, 7, 8, 9, 0),
    Duration::from_seconds(-(3600 + 60 + 1)).unwrap(),
    instance(2014, 5, 6, 6, 7, 8, 0)
  );
  test!(
    instance(2014, 5, 6, 7, 8, 9, 0),
    Duration::from_seconds(86399).unwrap(),
    instance(2014, 5, 7, 7, 8, 8, 0)
  );
  test!(
    instance(2014, 5, 6, 7, 8, 9, 0),
    Duration::from_seconds(86_400 * 10).unwrap(),
    instance(2014, 5, 16, 7, 8, 9, 0)
  );
  test!(
    instance(2014, 5, 6, 7, 8, 9, 0),
    Duration::from_seconds(-86_400 * 10).unwrap(),
    instance(2014, 4, 26, 7, 8, 9, 0)
  );
  test!(
    instance(2014, 5, 6, 7, 8, 9, 0),
    Duration::from_seconds(86_400 * 10).unwrap(),
    instance(2014, 5, 16, 7, 8, 9, 0)
  );
}

#[test]
fn from_timestamp_secs() {
  let elements = [
    (1662921288, "2022-09-11T18:34:48"),
    (1662921287, "2022-09-11T18:34:47"),
    (-2208936075, "1900-01-01T14:38:45"),
    (-5337182663, "1800-11-15T01:15:37"),
    (0000000000, "1970-01-01T00:00:00"),
    (119731017, "1973-10-17T18:36:57"),
    (1234567890, "2009-02-13T23:31:30"),
    (2034061609, "2034-06-16T09:06:49"),
  ];
  for (timestamp, str) in elements {
    let instance = DateTime::from_timestamp_secs(timestamp).unwrap();
    assert_eq!(instance.iso_8601().as_str(), str);
    assert_eq!(instance.timestamp().0, timestamp);
  }
}

#[test]
fn timestamp() {
  assert_eq!(DateTime::MIN.timestamp().0, -1096193779200);
  assert_eq!(DateTime::MAX.timestamp().0, 971859427199);
  assert_eq!(_2025_04_20_14_20_30_1234().timestamp().0, 1745158830);
}

#[test]
fn to_str() {
  assert_eq!(DateTime::MIN.iso_8601().as_str(), "-32767-01-01T00:00:00");
  assert_eq!(DateTime::MAX.iso_8601().as_str(), "32766-12-31T23:59:59.999999999");
  assert_eq!(_2025_04_20_14_20_30_1234().iso_8601().as_str(), "2025-04-20T14:20:30.1234");
}

fn instance(y: i16, mon: u8, d: u8, h: u8, min: u8, s: u8, ms: u16) -> DateTime {
  DateTime::new(
    Date::from_ymd(y.try_into().unwrap(), mon.try_into().unwrap(), d.try_into().unwrap()).unwrap(),
    Time::from_hms_ms(
      h.try_into().unwrap(),
      min.try_into().unwrap(),
      s.try_into().unwrap(),
      ms.try_into().unwrap(),
    ),
  )
}
