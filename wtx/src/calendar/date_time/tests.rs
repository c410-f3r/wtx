use crate::calendar::{
  Date, DateTime, DayOfYear, Duration, DynTz, Hour, Minute, Nanosecond, Second, Time, TimeZone,
  Utc, Year,
};

fn _2025_04_20_14_20_30_1234() -> DateTime<Utc> {
  DateTime::new(
    Date::new(Year::from_num(2025).unwrap(), DayOfYear::from_num(110).unwrap()).unwrap(),
    Time::from_hms_ns(Hour::N14, Minute::N20, Second::N30, Nanosecond::from_num(1234).unwrap()),
    Utc,
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
    instance(2014, 5, 6, 7, 8, 9, 0, Utc),
    Duration::from_seconds(3600 + 60 + 1).unwrap(),
    instance(2014, 5, 6, 8, 9, 10, 0, Utc)
  );
  test!(
    instance(2014, 5, 6, 7, 8, 9, 0, Utc),
    Duration::from_seconds(-(3600 + 60 + 1)).unwrap(),
    instance(2014, 5, 6, 6, 7, 8, 0, Utc)
  );
  test!(
    instance(2014, 5, 6, 7, 8, 9, 0, Utc),
    Duration::from_seconds(86399).unwrap(),
    instance(2014, 5, 7, 7, 8, 8, 0, Utc)
  );
  test!(
    instance(2014, 5, 6, 7, 8, 9, 0, Utc),
    Duration::from_seconds(86_400 * 10).unwrap(),
    instance(2014, 5, 16, 7, 8, 9, 0, Utc)
  );
  test!(
    instance(2014, 5, 6, 7, 8, 9, 0, Utc),
    Duration::from_seconds(-86_400 * 10).unwrap(),
    instance(2014, 4, 26, 7, 8, 9, 0, Utc)
  );
  test!(
    instance(2014, 5, 6, 7, 8, 9, 0, Utc),
    Duration::from_seconds(86_400 * 10).unwrap(),
    instance(2014, 5, 16, 7, 8, 9, 0, Utc)
  );
}

#[test]
fn add_days_with_tzs() {
  fn base<TZ>(tz: TZ) -> DateTime<TZ>
  where
    TZ: TimeZone,
  {
    instance(2014, 5, 6, 7, 8, 9, 0, tz)
  }

  let east = DynTz::from_minutes(9 * 60).unwrap();
  let west = DynTz::from_minutes(-5 * 60).unwrap();

  assert_eq!(
    &base(east).add(Duration::from_days(5).unwrap()).unwrap().iso_8601(),
    "2014-05-11T07:08:09+09:00"
  );
  assert_eq!(
    &base(west).add(Duration::from_days(5).unwrap()).unwrap().iso_8601(),
    "2014-05-11T07:08:09-05:00"
  );

  assert_eq!(
    &base(east).add(Duration::from_days(35).unwrap()).unwrap().iso_8601(),
    "2014-06-10T07:08:09+09:00"
  );
  assert_eq!(
    &base(west).add(Duration::from_days(35).unwrap()).unwrap().iso_8601(),
    "2014-06-10T07:08:09-05:00"
  );
}

#[test]
fn from_iso_8601() {
  let _datetime = DateTime::<Utc>::from_iso_8601(b"2022-02-10T10:10:10").unwrap();
  let _datetime = DateTime::<Utc>::from_iso_8601(b"2022-02-10T10:10:10.000").unwrap();
  let _datetime = DateTime::<Utc>::from_iso_8601(b"2022-02-10T10:10:10.000Z").unwrap();
  let _datetime = DateTime::<DynTz>::from_iso_8601(b"2022-02-10T10:10:10.000-04").unwrap();
  let _datetime = DateTime::<DynTz>::from_iso_8601(b"2022-02-10T10:10:10.000+04:30").unwrap();
  let _datetime = DateTime::<DynTz>::from_iso_8601(b"2022-01-10T10:10:10.000-00:00").unwrap();
}

#[test]
fn from_timestamp_secs() {
  let elements = [
    (1662921288, "2022-09-11T18:34:48Z"),
    (1662921287, "2022-09-11T18:34:47Z"),
    (-2208936075, "1900-01-01T14:38:45Z"),
    (-5337182663, "1800-11-15T01:15:37Z"),
    (0000000000, "1970-01-01T00:00:00Z"),
    (119731017, "1973-10-17T18:36:57Z"),
    (1234567890, "2009-02-13T23:31:30Z"),
    (2034061609, "2034-06-16T09:06:49Z"),
  ];
  for (timestamp, str) in elements {
    let instance = DateTime::from_timestamp_secs(timestamp).unwrap();
    assert_eq!(instance.iso_8601().as_str(), str);
    assert_eq!(instance.timestamp_secs_and_ns().0, timestamp);
  }
}

#[test]
fn iso_8601() {
  fn base0<TZ>(tz: TZ) -> DateTime<TZ>
  where
    TZ: TimeZone,
  {
    instance(2014, 5, 6, 7, 8, 9, 0, tz)
  }
  fn base1<TZ>(tz: TZ) -> DateTime<TZ>
  where
    TZ: TimeZone,
  {
    instance(2014, 5, 6, 0, 0, 0, 0, tz)
  }
  fn base2<TZ>(tz: TZ) -> DateTime<TZ>
  where
    TZ: TimeZone,
  {
    instance(2014, 5, 6, 23, 59, 59, 0, tz)
  }

  let edt = DynTz::from_minutes(-4 * 60).unwrap();
  let kst = DynTz::from_minutes(9 * 60).unwrap();

  assert_eq!(&base0(Utc).iso_8601(), "2014-05-06T07:08:09Z");
  assert_eq!(&base0(edt).iso_8601(), "2014-05-06T07:08:09-04:00");
  assert_eq!(&base0(kst).iso_8601(), "2014-05-06T07:08:09+09:00");

  assert_eq!(&base1(Utc).iso_8601(), "2014-05-06T00:00:00Z");
  assert_eq!(&base1(edt).iso_8601(), "2014-05-06T00:00:00-04:00");
  assert_eq!(&base1(kst).iso_8601(), "2014-05-06T00:00:00+09:00");

  assert_eq!(&base2(Utc).iso_8601(), "2014-05-06T23:59:59Z");
  assert_eq!(&base2(edt).iso_8601(), "2014-05-06T23:59:59-04:00");
  assert_eq!(&base2(kst).iso_8601(), "2014-05-06T23:59:59+09:00");

  assert_eq!(DateTime::MIN.iso_8601().as_str(), "-32767-01-01T00:00:00Z");
  assert_eq!(DateTime::MAX.iso_8601().as_str(), "32766-12-31T23:59:59.999999999Z");
  assert_eq!(_2025_04_20_14_20_30_1234().iso_8601().as_str(), "2025-04-20T14:20:30.1234Z");
}

#[test]
fn timestamp() {
  assert_eq!(DateTime::MIN.timestamp_secs_and_ns().0, -1096193779200);
  assert_eq!(DateTime::MAX.timestamp_secs_and_ns().0, 971859427199);
  assert_eq!(_2025_04_20_14_20_30_1234().timestamp_secs_and_ns().0, 1745158830);
}

#[test]
fn times_zones() {
  assert_eq!(DateTime::MIN.timestamp_secs_and_ns().0, -1096193779200);
  assert_eq!(DateTime::MAX.timestamp_secs_and_ns().0, 971859427199);
  assert_eq!(_2025_04_20_14_20_30_1234().timestamp_secs_and_ns().0, 1745158830);
}

#[test]
fn to_tz() {
  assert_eq!(
    DateTime::<DynTz>::from_iso_8601(b"1234-10-15T14:33:10-04:00")
      .unwrap()
      .to_tz(DynTz::from_minutes(60).unwrap())
      .unwrap(),
    DateTime::<DynTz>::from_iso_8601(b"1234-10-15T19:33:10+01:00").unwrap()
  );
  assert_eq!(
    DateTime::<DynTz>::from_iso_8601(b"1234-10-15T14:33:10+04:30")
      .unwrap()
      .to_tz(DynTz::from_minutes(-60).unwrap())
      .unwrap(),
    DateTime::<DynTz>::from_iso_8601(b"1234-10-15T09:03:10-01:00").unwrap()
  );
}

#[test]
fn to_utc() {
  assert_eq!(
    DateTime::<DynTz>::from_iso_8601(b"0123-01-04T03:20:01-04").unwrap().to_utc().unwrap(),
    DateTime::<Utc>::from_iso_8601(b"0123-01-04T07:20:01Z").unwrap()
  );
  assert_eq!(
    DateTime::<DynTz>::from_iso_8601(b"3210-02-30T13:25:10+04:05").unwrap().to_utc().unwrap(),
    DateTime::<Utc>::from_iso_8601(b"3210-02-30T09:20:10Z").unwrap()
  );
}

fn instance<TZ>(y: i16, mon: u8, d: u8, h: u8, min: u8, s: u8, ms: u16, tz: TZ) -> DateTime<TZ>
where
  TZ: TimeZone,
{
  DateTime::new(
    Date::from_ymd(y.try_into().unwrap(), mon.try_into().unwrap(), d.try_into().unwrap()).unwrap(),
    Time::from_hms_ms(
      h.try_into().unwrap(),
      min.try_into().unwrap(),
      s.try_into().unwrap(),
      ms.try_into().unwrap(),
    ),
    tz,
  )
}
