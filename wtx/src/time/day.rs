use crate::time::TimeError;

/// Day of the month.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Day {
  /// One
  N1,
  /// Two
  N2,
  /// Three
  N3,
  /// Four
  N4,
  /// Five
  N5,
  /// Six
  N6,
  /// Seven
  N7,
  /// Eight
  N8,
  /// Nine
  N9,
  /// Ten
  N10,
  /// Eleven
  N11,
  /// Twelve
  N12,
  /// Thirteen
  N13,
  /// Fourteen
  N14,
  /// Fifteen
  N15,
  /// Sixteen
  N16,
  /// Seventeen
  N17,
  /// Eighteen
  N18,
  /// Nineteen
  N19,
  /// Twenty
  N20,
  /// Twenty-one
  N21,
  /// Twenty-two
  N22,
  /// Twenty-three
  N23,
  /// Twenty-four
  N24,
  /// Twenty-five
  N25,
  /// Twenty-six
  N26,
  /// Twenty-seven
  N27,
  /// Twenty-eight
  N28,
  /// Twenty-nine
  N29,
  /// Thirty
  N30,
  /// Thirty-one
  N31,
}

impl Day {
  /// Creates a new instance from a valid `num` number.
  #[inline]
  pub const fn from_num(num: u8) -> Result<Self, TimeError> {
    Ok(match num {
      1 => Self::N1,
      2 => Self::N2,
      3 => Self::N3,
      4 => Self::N4,
      5 => Self::N5,
      6 => Self::N6,
      7 => Self::N7,
      8 => Self::N8,
      9 => Self::N9,
      10 => Self::N10,
      11 => Self::N11,
      12 => Self::N12,
      13 => Self::N13,
      14 => Self::N14,
      15 => Self::N15,
      16 => Self::N16,
      17 => Self::N17,
      18 => Self::N18,
      19 => Self::N19,
      20 => Self::N20,
      21 => Self::N21,
      22 => Self::N22,
      23 => Self::N23,
      24 => Self::N24,
      25 => Self::N25,
      26 => Self::N26,
      27 => Self::N27,
      28 => Self::N28,
      29 => Self::N29,
      30 => Self::N30,
      31 => Self::N31,
      _ => return Err(TimeError::InvalidMonthDay { received: num }),
    })
  }

  /// Integer representation
  #[inline]
  pub const fn num(&self) -> u8 {
    match self {
      Self::N1 => 1,
      Self::N2 => 2,
      Self::N3 => 3,
      Self::N4 => 4,
      Self::N5 => 5,
      Self::N6 => 6,
      Self::N7 => 7,
      Self::N8 => 8,
      Self::N9 => 9,
      Self::N10 => 10,
      Self::N11 => 11,
      Self::N12 => 12,
      Self::N13 => 13,
      Self::N14 => 14,
      Self::N15 => 15,
      Self::N16 => 16,
      Self::N17 => 17,
      Self::N18 => 18,
      Self::N19 => 19,
      Self::N20 => 20,
      Self::N21 => 21,
      Self::N22 => 22,
      Self::N23 => 23,
      Self::N24 => 24,
      Self::N25 => 25,
      Self::N26 => 26,
      Self::N27 => 27,
      Self::N28 => 28,
      Self::N29 => 29,
      Self::N30 => 30,
      Self::N31 => 31,
    }
  }

  /// String representation
  #[inline]
  pub const fn num_str(&self) -> &'static str {
    match self {
      Self::N1 => "01",
      Self::N2 => "02",
      Self::N3 => "03",
      Self::N4 => "04",
      Self::N5 => "05",
      Self::N6 => "06",
      Self::N7 => "07",
      Self::N8 => "08",
      Self::N9 => "09",
      Self::N10 => "10",
      Self::N11 => "11",
      Self::N12 => "12",
      Self::N13 => "13",
      Self::N14 => "14",
      Self::N15 => "15",
      Self::N16 => "16",
      Self::N17 => "17",
      Self::N18 => "18",
      Self::N19 => "19",
      Self::N20 => "20",
      Self::N21 => "21",
      Self::N22 => "22",
      Self::N23 => "23",
      Self::N24 => "24",
      Self::N25 => "25",
      Self::N26 => "26",
      Self::N27 => "27",
      Self::N28 => "28",
      Self::N29 => "29",
      Self::N30 => "30",
      Self::N31 => "31",
    }
  }
}

impl TryFrom<u8> for Day {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: u8) -> Result<Self, Self::Error> {
    Ok(Self::from_num(from)?)
  }
}
