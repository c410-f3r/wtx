#![allow(clippy::too_many_lines, reason = "it is the enum's nature to have too many variants")]

use crate::time::TimeError;

/// Day of the year
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DayOfYear {
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
  /// Sixty
  N60,
  /// Sixty-one
  N61,
  /// Sixty-two
  N62,
  /// Sixty-three
  N63,
  /// Sixty-four
  N64,
  /// Sixty-five
  N65,
  /// Sixty-six
  N66,
  /// Sixty-seven
  N67,
  /// Sixty-eight
  N68,
  /// Sixty-nine
  N69,
  /// Seventy
  N70,
  /// Seventy-one
  N71,
  /// Seventy-two
  N72,
  /// Seventy-three
  N73,
  /// Seventy-four
  N74,
  /// Seventy-five
  N75,
  /// Seventy-six
  N76,
  /// Seventy-seven
  N77,
  /// Seventy-eight
  N78,
  /// Seventy-nine
  N79,
  /// Eighty
  N80,
  /// Eighty-one
  N81,
  /// Eighty-two
  N82,
  /// Eighty-three
  N83,
  /// Eighty-four
  N84,
  /// Eighty-five
  N85,
  /// Eighty-six
  N86,
  /// Eighty-seven
  N87,
  /// Eighty-eight
  N88,
  /// Eighty-nine
  N89,
  /// Ninety
  N90,
  /// Ninety-one
  N91,
  /// Ninety-two
  N92,
  /// Ninety-three
  N93,
  /// Ninety-four
  N94,
  /// Ninety-five
  N95,
  /// Ninety-six
  N96,
  /// Ninety-seven
  N97,
  /// Ninety-eight
  N98,
  /// Ninety-nine
  N99,
  /// One Hundred
  N100,
  /// One Hundred One
  N101,
  /// One Hundred Two
  N102,
  /// One Hundred Three
  N103,
  /// One Hundred Four
  N104,
  /// One Hundred Five
  N105,
  /// One Hundred Six
  N106,
  /// One Hundred Seven
  N107,
  /// One Hundred Eight
  N108,
  /// One Hundred Nine
  N109,
  /// One Hundred Ten
  N110,
  /// One Hundred Eleven
  N111,
  /// One Hundred Twelve
  N112,
  /// One Hundred Thirteen
  N113,
  /// One Hundred Fourteen
  N114,
  /// One Hundred Fifteen
  N115,
  /// One Hundred Sixteen
  N116,
  /// One Hundred Seventeen
  N117,
  /// One Hundred Eighteen
  N118,
  /// One Hundred Nineteen
  N119,
  /// One Hundred Twenty
  N120,
  /// One Hundred Twenty-one
  N121,
  /// One Hundred Twenty-two
  N122,
  /// One Hundred Twenty-three
  N123,
  /// One Hundred Twenty-four
  N124,
  /// One Hundred Twenty-five
  N125,
  /// One Hundred Twenty-six
  N126,
  /// One Hundred Twenty-seven
  N127,
  /// One Hundred Twenty-eight
  N128,
  /// One Hundred Twenty-nine
  N129,
  /// One Hundred Thirty
  N130,
  /// One Hundred Thirty-one
  N131,
  /// One Hundred Thirty-two
  N132,
  /// One Hundred Thirty-three
  N133,
  /// One Hundred Thirty-four
  N134,
  /// One Hundred Thirty-five
  N135,
  /// One Hundred Thirty-six
  N136,
  /// One Hundred Thirty-seven
  N137,
  /// One Hundred Thirty-eight
  N138,
  /// One Hundred Thirty-nine
  N139,
  /// One Hundred Forty
  N140,
  /// One Hundred Forty-one
  N141,
  /// One Hundred Forty-two
  N142,
  /// One Hundred Forty-three
  N143,
  /// One Hundred Forty-four
  N144,
  /// One Hundred Forty-five
  N145,
  /// One Hundred Forty-six
  N146,
  /// One Hundred Forty-seven
  N147,
  /// One Hundred Forty-eight
  N148,
  /// One Hundred Forty-nine
  N149,
  /// One Hundred Fifty
  N150,
  /// One Hundred Fifty-one
  N151,
  /// One Hundred Fifty-two
  N152,
  /// One Hundred Fifty-three
  N153,
  /// One Hundred Fifty-four
  N154,
  /// One Hundred Fifty-five
  N155,
  /// One Hundred Fifty-six
  N156,
  /// One Hundred Fifty-seven
  N157,
  /// One Hundred Fifty-eight
  N158,
  /// One Hundred Fifty-nine
  N159,
  /// One Hundred Sixty
  N160,
  /// One Hundred Sixty-one
  N161,
  /// One Hundred Sixty-two
  N162,
  /// One Hundred Sixty-three
  N163,
  /// One Hundred Sixty-four
  N164,
  /// One Hundred Sixty-five
  N165,
  /// One Hundred Sixty-six
  N166,
  /// One Hundred Sixty-seven
  N167,
  /// One Hundred Sixty-eight
  N168,
  /// One Hundred Sixty-nine
  N169,
  /// One Hundred Seventy
  N170,
  /// One Hundred Seventy-one
  N171,
  /// One Hundred Seventy-two
  N172,
  /// One Hundred Seventy-three
  N173,
  /// One Hundred Seventy-four
  N174,
  /// One Hundred Seventy-five
  N175,
  /// One Hundred Seventy-six
  N176,
  /// One Hundred Seventy-seven
  N177,
  /// One Hundred Seventy-eight
  N178,
  /// One Hundred Seventy-nine
  N179,
  /// One Hundred Eighty
  N180,
  /// One Hundred Eighty-one
  N181,
  /// One Hundred Eighty-two
  N182,
  /// One Hundred Eighty-three
  N183,
  /// One Hundred Eighty-four
  N184,
  /// One Hundred Eighty-five
  N185,
  /// One Hundred Eighty-six
  N186,
  /// One Hundred Eighty-seven
  N187,
  /// One Hundred Eighty-eight
  N188,
  /// One Hundred Eighty-nine
  N189,
  /// One Hundred Ninety
  N190,
  /// One Hundred Ninety-one
  N191,
  /// One Hundred Ninety-two
  N192,
  /// One Hundred Ninety-three
  N193,
  /// One Hundred Ninety-four
  N194,
  /// One Hundred Ninety-five
  N195,
  /// One Hundred Ninety-six
  N196,
  /// One Hundred Ninety-seven
  N197,
  /// One Hundred Ninety-eight
  N198,
  /// One Hundred Ninety-nine
  N199,
  /// Two Hundred
  N200,
  /// Two Hundred One
  N201,
  /// Two Hundred Two
  N202,
  /// Two Hundred Three
  N203,
  /// Two Hundred Four
  N204,
  /// Two Hundred Five
  N205,
  /// Two Hundred Six
  N206,
  /// Two Hundred Seven
  N207,
  /// Two Hundred Eight
  N208,
  /// Two Hundred Nine
  N209,
  /// Two Hundred Ten
  N210,
  /// Two Hundred Eleven
  N211,
  /// Two Hundred Twelve
  N212,
  /// Two Hundred Thirteen
  N213,
  /// Two Hundred Fourteen
  N214,
  /// Two Hundred Fifteen
  N215,
  /// Two Hundred Sixteen
  N216,
  /// Two Hundred Seventeen
  N217,
  /// Two Hundred Eighteen
  N218,
  /// Two Hundred Nineteen
  N219,
  /// Two Hundred Twenty
  N220,
  /// Two Hundred Twenty-one
  N221,
  /// Two Hundred Twenty-two
  N222,
  /// Two Hundred Twenty-three
  N223,
  /// Two Hundred Twenty-four
  N224,
  /// Two Hundred Twenty-five
  N225,
  /// Two Hundred Twenty-six
  N226,
  /// Two Hundred Twenty-seven
  N227,
  /// Two Hundred Twenty-eight
  N228,
  /// Two Hundred Twenty-nine
  N229,
  /// Two Hundred Thirty
  N230,
  /// Two Hundred Thirty-one
  N231,
  /// Two Hundred Thirty-two
  N232,
  /// Two Hundred Thirty-three
  N233,
  /// Two Hundred Thirty-four
  N234,
  /// Two Hundred Thirty-five
  N235,
  /// Two Hundred Thirty-six
  N236,
  /// Two Hundred Thirty-seven
  N237,
  /// Two Hundred Thirty-eight
  N238,
  /// Two Hundred Thirty-nine
  N239,
  /// Two Hundred Forty
  N240,
  /// Two Hundred Forty-one
  N241,
  /// Two Hundred Forty-two
  N242,
  /// Two Hundred Forty-three
  N243,
  /// Two Hundred Forty-four
  N244,
  /// Two Hundred Forty-five
  N245,
  /// Two Hundred Forty-six
  N246,
  /// Two Hundred Forty-seven
  N247,
  /// Two Hundred Forty-eight
  N248,
  /// Two Hundred Forty-nine
  N249,
  /// Two Hundred Fifty
  N250,
  /// Two Hundred Fifty-one
  N251,
  /// Two Hundred Fifty-two
  N252,
  /// Two Hundred Fifty-three
  N253,
  /// Two Hundred Fifty-four
  N254,
  /// Two Hundred Fifty-five
  N255,
  /// Two Hundred Fifty-six
  N256,
  /// Two Hundred Fifty-seven
  N257,
  /// Two Hundred Fifty-eight
  N258,
  /// Two Hundred Fifty-nine
  N259,
  /// Two Hundred Sixty
  N260,
  /// Two Hundred Sixty-one
  N261,
  /// Two Hundred Sixty-two
  N262,
  /// Two Hundred Sixty-three
  N263,
  /// Two Hundred Sixty-four
  N264,
  /// Two Hundred Sixty-five
  N265,
  /// Two Hundred Sixty-six
  N266,
  /// Two Hundred Sixty-seven
  N267,
  /// Two Hundred Sixty-eight
  N268,
  /// Two Hundred Sixty-nine
  N269,
  /// Two Hundred Seventy
  N270,
  /// Two Hundred Seventy-one
  N271,
  /// Two Hundred Seventy-two
  N272,
  /// Two Hundred Seventy-three
  N273,
  /// Two Hundred Seventy-four
  N274,
  /// Two Hundred Seventy-five
  N275,
  /// Two Hundred Seventy-six
  N276,
  /// Two Hundred Seventy-seven
  N277,
  /// Two Hundred Seventy-eight
  N278,
  /// Two Hundred Seventy-nine
  N279,
  /// Two Hundred Eighty
  N280,
  /// Two Hundred Eighty-one
  N281,
  /// Two Hundred Eighty-two
  N282,
  /// Two Hundred Eighty-three
  N283,
  /// Two Hundred Eighty-four
  N284,
  /// Two Hundred Eighty-five
  N285,
  /// Two Hundred Eighty-six
  N286,
  /// Two Hundred Eighty-seven
  N287,
  /// Two Hundred Eighty-eight
  N288,
  /// Two Hundred Eighty-nine
  N289,
  /// Two Hundred Ninety
  N290,
  /// Two Hundred Ninety-one
  N291,
  /// Two Hundred Ninety-two
  N292,
  /// Two Hundred Ninety-three
  N293,
  /// Two Hundred Ninety-four
  N294,
  /// Two Hundred Ninety-five
  N295,
  /// Two Hundred Ninety-six
  N296,
  /// Two Hundred Ninety-seven
  N297,
  /// Two Hundred Ninety-eight
  N298,
  /// Two Hundred Ninety-nine
  N299,
  /// Three Hundred
  N300,
  /// Three Hundred One
  N301,
  /// Three Hundred Two
  N302,
  /// Three Hundred Three
  N303,
  /// Three Hundred Four
  N304,
  /// Three Hundred Five
  N305,
  /// Three Hundred Six
  N306,
  /// Three Hundred Seven
  N307,
  /// Three Hundred Eight
  N308,
  /// Three Hundred Nine
  N309,
  /// Three Hundred Ten
  N310,
  /// Three Hundred Eleven
  N311,
  /// Three Hundred Twelve
  N312,
  /// Three Hundred Thirteen
  N313,
  /// Three Hundred Fourteen
  N314,
  /// Three Hundred Fifteen
  N315,
  /// Three Hundred Sixteen
  N316,
  /// Three Hundred Seventeen
  N317,
  /// Three Hundred Eighteen
  N318,
  /// Three Hundred Nineteen
  N319,
  /// Three Hundred Twenty
  N320,
  /// Three Hundred Twenty-one
  N321,
  /// Three Hundred Twenty-two
  N322,
  /// Three Hundred Twenty-three
  N323,
  /// Three Hundred Twenty-four
  N324,
  /// Three Hundred Twenty-five
  N325,
  /// Three Hundred Twenty-six
  N326,
  /// Three Hundred Twenty-seven
  N327,
  /// Three Hundred Twenty-eight
  N328,
  /// Three Hundred Twenty-nine
  N329,
  /// Three Hundred Thirty
  N330,
  /// Three Hundred Thirty-one
  N331,
  /// Three Hundred Thirty-two
  N332,
  /// Three Hundred Thirty-three
  N333,
  /// Three Hundred Thirty-four
  N334,
  /// Three Hundred Thirty-five
  N335,
  /// Three Hundred Thirty-six
  N336,
  /// Three Hundred Thirty-seven
  N337,
  /// Three Hundred Thirty-eight
  N338,
  /// Three Hundred Thirty-nine
  N339,
  /// Three Hundred Forty
  N340,
  /// Three Hundred Forty-one
  N341,
  /// Three Hundred Forty-two
  N342,
  /// Three Hundred Forty-three
  N343,
  /// Three Hundred Forty-four
  N344,
  /// Three Hundred Forty-five
  N345,
  /// Three Hundred Forty-six
  N346,
  /// Three Hundred Forty-seven
  N347,
  /// Three Hundred Forty-eight
  N348,
  /// Three Hundred Forty-nine
  N349,
  /// Three Hundred Fifty
  N350,
  /// Three Hundred Fifty-one
  N351,
  /// Three Hundred Fifty-two
  N352,
  /// Three Hundred Fifty-three
  N353,
  /// Three Hundred Fifty-four
  N354,
  /// Three Hundred Fifty-five
  N355,
  /// Three Hundred Fifty-six
  N356,
  /// Three Hundred Fifty-seven
  N357,
  /// Three Hundred Fifty-eight
  N358,
  /// Three Hundred Fifty-nine
  N359,
  /// Three Hundred Sixty
  N360,
  /// Three Hundred Sixty-one
  N361,
  /// Three Hundred Sixty-two
  N362,
  /// Three Hundred Sixty-three
  N363,
  /// Three Hundred Sixty-four
  N364,
  /// Three Hundred Sixty-five
  N365,
  /// Three Hundred Sixty-six (Leap Day)
  N366,
}

impl DayOfYear {
  /// Creates a new instance from valid a `num` number.
  #[inline]
  pub const fn from_num(num: u16) -> Result<Self, TimeError> {
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
      60 => Self::N60,
      61 => Self::N61,
      62 => Self::N62,
      63 => Self::N63,
      64 => Self::N64,
      65 => Self::N65,
      66 => Self::N66,
      67 => Self::N67,
      68 => Self::N68,
      69 => Self::N69,
      70 => Self::N70,
      71 => Self::N71,
      72 => Self::N72,
      73 => Self::N73,
      74 => Self::N74,
      75 => Self::N75,
      76 => Self::N76,
      77 => Self::N77,
      78 => Self::N78,
      79 => Self::N79,
      80 => Self::N80,
      81 => Self::N81,
      82 => Self::N82,
      83 => Self::N83,
      84 => Self::N84,
      85 => Self::N85,
      86 => Self::N86,
      87 => Self::N87,
      88 => Self::N88,
      89 => Self::N89,
      90 => Self::N90,
      91 => Self::N91,
      92 => Self::N92,
      93 => Self::N93,
      94 => Self::N94,
      95 => Self::N95,
      96 => Self::N96,
      97 => Self::N97,
      98 => Self::N98,
      99 => Self::N99,
      100 => Self::N100,
      101 => Self::N101,
      102 => Self::N102,
      103 => Self::N103,
      104 => Self::N104,
      105 => Self::N105,
      106 => Self::N106,
      107 => Self::N107,
      108 => Self::N108,
      109 => Self::N109,
      110 => Self::N110,
      111 => Self::N111,
      112 => Self::N112,
      113 => Self::N113,
      114 => Self::N114,
      115 => Self::N115,
      116 => Self::N116,
      117 => Self::N117,
      118 => Self::N118,
      119 => Self::N119,
      120 => Self::N120,
      121 => Self::N121,
      122 => Self::N122,
      123 => Self::N123,
      124 => Self::N124,
      125 => Self::N125,
      126 => Self::N126,
      127 => Self::N127,
      128 => Self::N128,
      129 => Self::N129,
      130 => Self::N130,
      131 => Self::N131,
      132 => Self::N132,
      133 => Self::N133,
      134 => Self::N134,
      135 => Self::N135,
      136 => Self::N136,
      137 => Self::N137,
      138 => Self::N138,
      139 => Self::N139,
      140 => Self::N140,
      141 => Self::N141,
      142 => Self::N142,
      143 => Self::N143,
      144 => Self::N144,
      145 => Self::N145,
      146 => Self::N146,
      147 => Self::N147,
      148 => Self::N148,
      149 => Self::N149,
      150 => Self::N150,
      151 => Self::N151,
      152 => Self::N152,
      153 => Self::N153,
      154 => Self::N154,
      155 => Self::N155,
      156 => Self::N156,
      157 => Self::N157,
      158 => Self::N158,
      159 => Self::N159,
      160 => Self::N160,
      161 => Self::N161,
      162 => Self::N162,
      163 => Self::N163,
      164 => Self::N164,
      165 => Self::N165,
      166 => Self::N166,
      167 => Self::N167,
      168 => Self::N168,
      169 => Self::N169,
      170 => Self::N170,
      171 => Self::N171,
      172 => Self::N172,
      173 => Self::N173,
      174 => Self::N174,
      175 => Self::N175,
      176 => Self::N176,
      177 => Self::N177,
      178 => Self::N178,
      179 => Self::N179,
      180 => Self::N180,
      181 => Self::N181,
      182 => Self::N182,
      183 => Self::N183,
      184 => Self::N184,
      185 => Self::N185,
      186 => Self::N186,
      187 => Self::N187,
      188 => Self::N188,
      189 => Self::N189,
      190 => Self::N190,
      191 => Self::N191,
      192 => Self::N192,
      193 => Self::N193,
      194 => Self::N194,
      195 => Self::N195,
      196 => Self::N196,
      197 => Self::N197,
      198 => Self::N198,
      199 => Self::N199,
      200 => Self::N200,
      201 => Self::N201,
      202 => Self::N202,
      203 => Self::N203,
      204 => Self::N204,
      205 => Self::N205,
      206 => Self::N206,
      207 => Self::N207,
      208 => Self::N208,
      209 => Self::N209,
      210 => Self::N210,
      211 => Self::N211,
      212 => Self::N212,
      213 => Self::N213,
      214 => Self::N214,
      215 => Self::N215,
      216 => Self::N216,
      217 => Self::N217,
      218 => Self::N218,
      219 => Self::N219,
      220 => Self::N220,
      221 => Self::N221,
      222 => Self::N222,
      223 => Self::N223,
      224 => Self::N224,
      225 => Self::N225,
      226 => Self::N226,
      227 => Self::N227,
      228 => Self::N228,
      229 => Self::N229,
      230 => Self::N230,
      231 => Self::N231,
      232 => Self::N232,
      233 => Self::N233,
      234 => Self::N234,
      235 => Self::N235,
      236 => Self::N236,
      237 => Self::N237,
      238 => Self::N238,
      239 => Self::N239,
      240 => Self::N240,
      241 => Self::N241,
      242 => Self::N242,
      243 => Self::N243,
      244 => Self::N244,
      245 => Self::N245,
      246 => Self::N246,
      247 => Self::N247,
      248 => Self::N248,
      249 => Self::N249,
      250 => Self::N250,
      251 => Self::N251,
      252 => Self::N252,
      253 => Self::N253,
      254 => Self::N254,
      255 => Self::N255,
      256 => Self::N256,
      257 => Self::N257,
      258 => Self::N258,
      259 => Self::N259,
      260 => Self::N260,
      261 => Self::N261,
      262 => Self::N262,
      263 => Self::N263,
      264 => Self::N264,
      265 => Self::N265,
      266 => Self::N266,
      267 => Self::N267,
      268 => Self::N268,
      269 => Self::N269,
      270 => Self::N270,
      271 => Self::N271,
      272 => Self::N272,
      273 => Self::N273,
      274 => Self::N274,
      275 => Self::N275,
      276 => Self::N276,
      277 => Self::N277,
      278 => Self::N278,
      279 => Self::N279,
      280 => Self::N280,
      281 => Self::N281,
      282 => Self::N282,
      283 => Self::N283,
      284 => Self::N284,
      285 => Self::N285,
      286 => Self::N286,
      287 => Self::N287,
      288 => Self::N288,
      289 => Self::N289,
      290 => Self::N290,
      291 => Self::N291,
      292 => Self::N292,
      293 => Self::N293,
      294 => Self::N294,
      295 => Self::N295,
      296 => Self::N296,
      297 => Self::N297,
      298 => Self::N298,
      299 => Self::N299,
      300 => Self::N300,
      301 => Self::N301,
      302 => Self::N302,
      303 => Self::N303,
      304 => Self::N304,
      305 => Self::N305,
      306 => Self::N306,
      307 => Self::N307,
      308 => Self::N308,
      309 => Self::N309,
      310 => Self::N310,
      311 => Self::N311,
      312 => Self::N312,
      313 => Self::N313,
      314 => Self::N314,
      315 => Self::N315,
      316 => Self::N316,
      317 => Self::N317,
      318 => Self::N318,
      319 => Self::N319,
      320 => Self::N320,
      321 => Self::N321,
      322 => Self::N322,
      323 => Self::N323,
      324 => Self::N324,
      325 => Self::N325,
      326 => Self::N326,
      327 => Self::N327,
      328 => Self::N328,
      329 => Self::N329,
      330 => Self::N330,
      331 => Self::N331,
      332 => Self::N332,
      333 => Self::N333,
      334 => Self::N334,
      335 => Self::N335,
      336 => Self::N336,
      337 => Self::N337,
      338 => Self::N338,
      339 => Self::N339,
      340 => Self::N340,
      341 => Self::N341,
      342 => Self::N342,
      343 => Self::N343,
      344 => Self::N344,
      345 => Self::N345,
      346 => Self::N346,
      347 => Self::N347,
      348 => Self::N348,
      349 => Self::N349,
      350 => Self::N350,
      351 => Self::N351,
      352 => Self::N352,
      353 => Self::N353,
      354 => Self::N354,
      355 => Self::N355,
      356 => Self::N356,
      357 => Self::N357,
      358 => Self::N358,
      359 => Self::N359,
      360 => Self::N360,
      361 => Self::N361,
      362 => Self::N362,
      363 => Self::N363,
      364 => Self::N364,
      365 => Self::N365,
      366 => Self::N366,
      _ => return Err(TimeError::InvalidDayOfTheYear { received: num }),
    })
  }

  /// Integer representation
  #[inline]
  pub const fn num(&self) -> u16 {
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
      Self::N60 => 60,
      Self::N61 => 61,
      Self::N62 => 62,
      Self::N63 => 63,
      Self::N64 => 64,
      Self::N65 => 65,
      Self::N66 => 66,
      Self::N67 => 67,
      Self::N68 => 68,
      Self::N69 => 69,
      Self::N70 => 70,
      Self::N71 => 71,
      Self::N72 => 72,
      Self::N73 => 73,
      Self::N74 => 74,
      Self::N75 => 75,
      Self::N76 => 76,
      Self::N77 => 77,
      Self::N78 => 78,
      Self::N79 => 79,
      Self::N80 => 80,
      Self::N81 => 81,
      Self::N82 => 82,
      Self::N83 => 83,
      Self::N84 => 84,
      Self::N85 => 85,
      Self::N86 => 86,
      Self::N87 => 87,
      Self::N88 => 88,
      Self::N89 => 89,
      Self::N90 => 90,
      Self::N91 => 91,
      Self::N92 => 92,
      Self::N93 => 93,
      Self::N94 => 94,
      Self::N95 => 95,
      Self::N96 => 96,
      Self::N97 => 97,
      Self::N98 => 98,
      Self::N99 => 99,
      Self::N100 => 100,
      Self::N101 => 101,
      Self::N102 => 102,
      Self::N103 => 103,
      Self::N104 => 104,
      Self::N105 => 105,
      Self::N106 => 106,
      Self::N107 => 107,
      Self::N108 => 108,
      Self::N109 => 109,
      Self::N110 => 110,
      Self::N111 => 111,
      Self::N112 => 112,
      Self::N113 => 113,
      Self::N114 => 114,
      Self::N115 => 115,
      Self::N116 => 116,
      Self::N117 => 117,
      Self::N118 => 118,
      Self::N119 => 119,
      Self::N120 => 120,
      Self::N121 => 121,
      Self::N122 => 122,
      Self::N123 => 123,
      Self::N124 => 124,
      Self::N125 => 125,
      Self::N126 => 126,
      Self::N127 => 127,
      Self::N128 => 128,
      Self::N129 => 129,
      Self::N130 => 130,
      Self::N131 => 131,
      Self::N132 => 132,
      Self::N133 => 133,
      Self::N134 => 134,
      Self::N135 => 135,
      Self::N136 => 136,
      Self::N137 => 137,
      Self::N138 => 138,
      Self::N139 => 139,
      Self::N140 => 140,
      Self::N141 => 141,
      Self::N142 => 142,
      Self::N143 => 143,
      Self::N144 => 144,
      Self::N145 => 145,
      Self::N146 => 146,
      Self::N147 => 147,
      Self::N148 => 148,
      Self::N149 => 149,
      Self::N150 => 150,
      Self::N151 => 151,
      Self::N152 => 152,
      Self::N153 => 153,
      Self::N154 => 154,
      Self::N155 => 155,
      Self::N156 => 156,
      Self::N157 => 157,
      Self::N158 => 158,
      Self::N159 => 159,
      Self::N160 => 160,
      Self::N161 => 161,
      Self::N162 => 162,
      Self::N163 => 163,
      Self::N164 => 164,
      Self::N165 => 165,
      Self::N166 => 166,
      Self::N167 => 167,
      Self::N168 => 168,
      Self::N169 => 169,
      Self::N170 => 170,
      Self::N171 => 171,
      Self::N172 => 172,
      Self::N173 => 173,
      Self::N174 => 174,
      Self::N175 => 175,
      Self::N176 => 176,
      Self::N177 => 177,
      Self::N178 => 178,
      Self::N179 => 179,
      Self::N180 => 180,
      Self::N181 => 181,
      Self::N182 => 182,
      Self::N183 => 183,
      Self::N184 => 184,
      Self::N185 => 185,
      Self::N186 => 186,
      Self::N187 => 187,
      Self::N188 => 188,
      Self::N189 => 189,
      Self::N190 => 190,
      Self::N191 => 191,
      Self::N192 => 192,
      Self::N193 => 193,
      Self::N194 => 194,
      Self::N195 => 195,
      Self::N196 => 196,
      Self::N197 => 197,
      Self::N198 => 198,
      Self::N199 => 199,
      Self::N200 => 200,
      Self::N201 => 201,
      Self::N202 => 202,
      Self::N203 => 203,
      Self::N204 => 204,
      Self::N205 => 205,
      Self::N206 => 206,
      Self::N207 => 207,
      Self::N208 => 208,
      Self::N209 => 209,
      Self::N210 => 210,
      Self::N211 => 211,
      Self::N212 => 212,
      Self::N213 => 213,
      Self::N214 => 214,
      Self::N215 => 215,
      Self::N216 => 216,
      Self::N217 => 217,
      Self::N218 => 218,
      Self::N219 => 219,
      Self::N220 => 220,
      Self::N221 => 221,
      Self::N222 => 222,
      Self::N223 => 223,
      Self::N224 => 224,
      Self::N225 => 225,
      Self::N226 => 226,
      Self::N227 => 227,
      Self::N228 => 228,
      Self::N229 => 229,
      Self::N230 => 230,
      Self::N231 => 231,
      Self::N232 => 232,
      Self::N233 => 233,
      Self::N234 => 234,
      Self::N235 => 235,
      Self::N236 => 236,
      Self::N237 => 237,
      Self::N238 => 238,
      Self::N239 => 239,
      Self::N240 => 240,
      Self::N241 => 241,
      Self::N242 => 242,
      Self::N243 => 243,
      Self::N244 => 244,
      Self::N245 => 245,
      Self::N246 => 246,
      Self::N247 => 247,
      Self::N248 => 248,
      Self::N249 => 249,
      Self::N250 => 250,
      Self::N251 => 251,
      Self::N252 => 252,
      Self::N253 => 253,
      Self::N254 => 254,
      Self::N255 => 255,
      Self::N256 => 256,
      Self::N257 => 257,
      Self::N258 => 258,
      Self::N259 => 259,
      Self::N260 => 260,
      Self::N261 => 261,
      Self::N262 => 262,
      Self::N263 => 263,
      Self::N264 => 264,
      Self::N265 => 265,
      Self::N266 => 266,
      Self::N267 => 267,
      Self::N268 => 268,
      Self::N269 => 269,
      Self::N270 => 270,
      Self::N271 => 271,
      Self::N272 => 272,
      Self::N273 => 273,
      Self::N274 => 274,
      Self::N275 => 275,
      Self::N276 => 276,
      Self::N277 => 277,
      Self::N278 => 278,
      Self::N279 => 279,
      Self::N280 => 280,
      Self::N281 => 281,
      Self::N282 => 282,
      Self::N283 => 283,
      Self::N284 => 284,
      Self::N285 => 285,
      Self::N286 => 286,
      Self::N287 => 287,
      Self::N288 => 288,
      Self::N289 => 289,
      Self::N290 => 290,
      Self::N291 => 291,
      Self::N292 => 292,
      Self::N293 => 293,
      Self::N294 => 294,
      Self::N295 => 295,
      Self::N296 => 296,
      Self::N297 => 297,
      Self::N298 => 298,
      Self::N299 => 299,
      Self::N300 => 300,
      Self::N301 => 301,
      Self::N302 => 302,
      Self::N303 => 303,
      Self::N304 => 304,
      Self::N305 => 305,
      Self::N306 => 306,
      Self::N307 => 307,
      Self::N308 => 308,
      Self::N309 => 309,
      Self::N310 => 310,
      Self::N311 => 311,
      Self::N312 => 312,
      Self::N313 => 313,
      Self::N314 => 314,
      Self::N315 => 315,
      Self::N316 => 316,
      Self::N317 => 317,
      Self::N318 => 318,
      Self::N319 => 319,
      Self::N320 => 320,
      Self::N321 => 321,
      Self::N322 => 322,
      Self::N323 => 323,
      Self::N324 => 324,
      Self::N325 => 325,
      Self::N326 => 326,
      Self::N327 => 327,
      Self::N328 => 328,
      Self::N329 => 329,
      Self::N330 => 330,
      Self::N331 => 331,
      Self::N332 => 332,
      Self::N333 => 333,
      Self::N334 => 334,
      Self::N335 => 335,
      Self::N336 => 336,
      Self::N337 => 337,
      Self::N338 => 338,
      Self::N339 => 339,
      Self::N340 => 340,
      Self::N341 => 341,
      Self::N342 => 342,
      Self::N343 => 343,
      Self::N344 => 344,
      Self::N345 => 345,
      Self::N346 => 346,
      Self::N347 => 347,
      Self::N348 => 348,
      Self::N349 => 349,
      Self::N350 => 350,
      Self::N351 => 351,
      Self::N352 => 352,
      Self::N353 => 353,
      Self::N354 => 354,
      Self::N355 => 355,
      Self::N356 => 356,
      Self::N357 => 357,
      Self::N358 => 358,
      Self::N359 => 359,
      Self::N360 => 360,
      Self::N361 => 361,
      Self::N362 => 362,
      Self::N363 => 363,
      Self::N364 => 364,
      Self::N365 => 365,
      Self::N366 => 366,
    }
  }
}
