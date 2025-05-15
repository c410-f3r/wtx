use crate::calendar::{Duration, Hour, Minute, Second, Time, nanosecond::Nanosecond};

fn _8_48_05_234_445_009() -> Time {
  Time::from_hms_ns(Hour::N8, Minute::N48, Second::N5, Nanosecond::from_num(234_445_009).unwrap())
}

fn _14_20_30() -> Time {
  Time::from_hms(Hour::N14, Minute::N20, Second::N30)
}

#[test]
fn hour() {
  assert_eq!(Time::ZERO.hour().num(), 0);
  assert_eq!(Time::MAX.hour().num(), 23);
  assert_eq!(_8_48_05_234_445_009().hour().num(), 8);
  assert_eq!(_14_20_30().hour().num(), 14);
}

#[test]
fn iso_8601() {
  assert_eq!(Time::ZERO.iso_8601().as_str(), "00:00:00");
  assert_eq!(Time::MAX.iso_8601().as_str(), "23:59:59.999999999");
  assert_eq!(_8_48_05_234_445_009().iso_8601().as_str(), "08:48:05.234445009");
  assert_eq!(_14_20_30().iso_8601().as_str(), "14:20:30");

  let valid = [
    "09:08:07",
    "09:08:07.1",
    "09:08:07.12",
    "09:08:07.123",
    "09:08:07.123",
    "09:08:07.1234",
    "09:08:07.12345",
    "09:08:07.123456",
    "09:08:07.1234567",
    "09:08:07.12345678",
    "09:08:07.123456789",
  ];
  for str in valid {
    let time = Time::from_iso_8601(str.as_bytes()).unwrap();
    let time_str = time.iso_8601();
    assert_eq!(time, Time::from_iso_8601(time_str.as_bytes()).unwrap());
  }

  let invalid = [
    "",
    "x",
    "15",
    "15:8:",
    "15:8:x",
    "15:8:9x",
    "23:59:61",
    "23:54:35 GMT",
    "23:54:35 +0000",
    "1441497364.649",
    "+1441497364.649",
    "+1441497364",
    "001:02:03",
    "01:002:03",
    "01:02:003",
    "12:34:56.x",
    "12:34:56. 0",
    "09:08:00000000007",
  ];
  for str in invalid {
    assert!(Time::from_iso_8601(str.as_bytes()).is_err());
  }
}

#[test]
fn minute() {
  assert_eq!(Time::ZERO.minute().num(), 0);
  assert_eq!(Time::MAX.minute().num(), 59);
  assert_eq!(_8_48_05_234_445_009().minute().num(), 48);
  assert_eq!(_14_20_30().minute().num(), 20);
}

#[test]
fn nanosecond() {
  assert_eq!(Time::ZERO.nanosecond().num(), 0);
  assert_eq!(Time::MAX.nanosecond().num(), 999_999_999);
  assert_eq!(_8_48_05_234_445_009().nanosecond().num(), 234_445_009);
  assert_eq!(_14_20_30().nanosecond().num(), 0);
}

#[test]
fn overflowing_add_and_sub() {
  macro_rules! test {
    ($lhs:expr, $rhs:expr, $rslt:expr) => {{
      let (this, rem) = $rslt;
      assert_eq!($lhs.overflowing_add($rhs), (this, rem));
      assert_eq!($lhs.overflowing_sub($rhs.neg()), (this, -rem));
    }};
  }

  test!(
    instance(0, 0, 0, 0),
    Duration::from_milliseconds(-990),
    (instance(23, 59, 59, 10), -86_400)
  );
  test!(
    instance(0, 0, 0, 0),
    Duration::from_milliseconds(-9990),
    (instance(23, 59, 50, 10), -86_400)
  );
  test!(
    instance(3, 4, 5, 678),
    Duration::from_hours(-7).unwrap(),
    (instance(20, 4, 5, 678), -86_400)
  );
  test!(instance(3, 4, 5, 678), Duration::from_hours(11).unwrap(), (instance(14, 4, 5, 678), 0));
  test!(
    instance(3, 4, 5, 678),
    Duration::from_hours(23).unwrap(),
    (instance(2, 4, 5, 678), 86_400)
  );
  test!(
    instance(3, 5, 59, 900),
    Duration::from_days(12345).unwrap(),
    (instance(3, 5, 59, 900), 1_066_608_000)
  );
  test!(instance(3, 5, 59, 900), Duration::from_milliseconds(100), (instance(3, 6, 0, 0), 0));
  test!(
    instance(3, 5, 59, 900),
    Duration::from_seconds(-86399).unwrap(),
    (instance(3, 6, 0, 900), -86_400)
  );
  test!(
    instance(3, 5, 59, 900),
    Duration::from_seconds(86399).unwrap(),
    (instance(3, 5, 58, 900), 86_400)
  );
  test!(instance(3, 5, 59, 900), Duration::ZERO, (instance(3, 5, 59, 900), 0));
}

#[test]
fn second() {
  assert_eq!(Time::ZERO.second().num(), 0);
  assert_eq!(Time::MAX.second().num(), 59);
  assert_eq!(_8_48_05_234_445_009().second().num(), 5);
  assert_eq!(_14_20_30().second().num(), 30);
}

#[test]
fn seconds_from_mn() {
  assert_eq!(Time::ZERO.seconds_since_mn(), 0);
  assert_eq!(Time::MAX.seconds_since_mn(), 86_399);
  assert_eq!(_8_48_05_234_445_009().seconds_since_mn(), 28800 + 2880 + 5);
  assert_eq!(_14_20_30().seconds_since_mn(), 50400 + 1200 + 30);
}

fn instance(h: u8, m: u8, s: u8, ms: u16) -> Time {
  Time::from_hms_ms(
    h.try_into().unwrap(),
    m.try_into().unwrap(),
    s.try_into().unwrap(),
    ms.try_into().unwrap(),
  )
}
