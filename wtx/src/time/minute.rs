use crate::time::TimeError;

/// Minutes of an hour.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Minute {
  /// Zero
  N0,
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
  /// Thirty-two
  N32,
  /// Thirty-three
  N33,
  /// Thirty-four
  N34,
  /// Thirty-five
  N35,
  /// Thirty-six
  N36,
  /// Thirty-seven
  N37,
  /// Thirty-eight
  N38,
  /// Thirty-nine
  N39,
  /// Forty
  N40,
  /// Forty-one
  N41,
  /// Forty-two
  N42,
  /// Forty-three
  N43,
  /// Forty-four
  N44,
  /// Forty-five
  N45,
  /// Forty-six
  N46,
  /// Forty-seven
  N47,
  /// Forty-eight
  N48,
  /// Forty-nine
  N49,
  /// Fifty
  N50,
  /// Fifty-one
  N51,
  /// Fifty-two
  N52,
  /// Fifty-three
  N53,
  /// Fifty-four
  N54,
  /// Fifty-five
  N55,
  /// Fifty-six
  N56,
  /// Fifty-seven
  N57,
  /// Fifty-eight
  N58,
  /// Fifty-nine
  N59,
}

impl Minute {
  /// Creates a new instance from a valid `num` number.
  #[inline]
  pub const fn from_num(num: u8) -> Result<Self, TimeError> {
    Ok(match num {
      0 => Self::N0,
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
      32 => Self::N32,
      33 => Self::N33,
      34 => Self::N34,
      35 => Self::N35,
      36 => Self::N36,
      37 => Self::N37,
      38 => Self::N38,
      39 => Self::N39,
      40 => Self::N40,
      41 => Self::N41,
      42 => Self::N42,
      43 => Self::N43,
      44 => Self::N44,
      45 => Self::N45,
      46 => Self::N46,
      47 => Self::N47,
      48 => Self::N48,
      49 => Self::N49,
      50 => Self::N50,
      51 => Self::N51,
      52 => Self::N52,
      53 => Self::N53,
      54 => Self::N54,
      55 => Self::N55,
      56 => Self::N56,
      57 => Self::N57,
      58 => Self::N58,
      59 => Self::N59,
      _ => return Err(TimeError::InvalidMinute { received: num }),
    })
  }

  /// Integer representation
  #[inline]
  pub const fn num(&self) -> u8 {
    match self {
      Self::N0 => 0,
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
      Self::N32 => 32,
      Self::N33 => 33,
      Self::N34 => 34,
      Self::N35 => 35,
      Self::N36 => 36,
      Self::N37 => 37,
      Self::N38 => 38,
      Self::N39 => 39,
      Self::N40 => 40,
      Self::N41 => 41,
      Self::N42 => 42,
      Self::N43 => 43,
      Self::N44 => 44,
      Self::N45 => 45,
      Self::N46 => 46,
      Self::N47 => 47,
      Self::N48 => 48,
      Self::N49 => 49,
      Self::N50 => 50,
      Self::N51 => 51,
      Self::N52 => 52,
      Self::N53 => 53,
      Self::N54 => 54,
      Self::N55 => 55,
      Self::N56 => 56,
      Self::N57 => 57,
      Self::N58 => 58,
      Self::N59 => 59,
    }
  }

  /// String representation
  #[inline]
  pub const fn num_str(&self) -> &'static str {
    match self {
      Self::N0 => "00",
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
      Self::N32 => "32",
      Self::N33 => "33",
      Self::N34 => "34",
      Self::N35 => "35",
      Self::N36 => "36",
      Self::N37 => "37",
      Self::N38 => "38",
      Self::N39 => "39",
      Self::N40 => "40",
      Self::N41 => "41",
      Self::N42 => "42",
      Self::N43 => "43",
      Self::N44 => "44",
      Self::N45 => "45",
      Self::N46 => "46",
      Self::N47 => "47",
      Self::N48 => "48",
      Self::N49 => "49",
      Self::N50 => "50",
      Self::N51 => "51",
      Self::N52 => "52",
      Self::N53 => "53",
      Self::N54 => "54",
      Self::N55 => "55",
      Self::N56 => "56",
      Self::N57 => "57",
      Self::N58 => "58",
      Self::N59 => "59",
    }
  }
}
