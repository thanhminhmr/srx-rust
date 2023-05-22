pub const STATE_TABLE: StateTable<255> = StateTable([
	StateElement::new( 0,  27,   0,   0,   0), //  0,  0,  0
	StateElement::new( 0,  28,   1,  27,   0), //  0,  0,  1
	StateElement::new( 0,  28,   1,  54,   0), //  0,  0,  2
	StateElement::new( 0,  29,   2,  74,   0), //  0,  0,  3
	StateElement::new( 0,  31,  27,   1,   1), //  0,  1,  0
	StateElement::new( 0,  32,  28,  28,   1), //  0,  1,  1
	StateElement::new( 0,  32,  28,  55,   1), //  0,  1,  2
	StateElement::new( 0,  33,  29,  75,   1), //  0,  1,  3
	StateElement::new( 0,  31,  54,   1,   1), //  0,  2,  0
	StateElement::new( 0,  32,  55,  28,   1), //  0,  2,  1
	StateElement::new( 0,  32,  55,  55,   1), //  0,  2,  2
	StateElement::new( 0,  33,  56,  75,   1), //  0,  2,  3
	StateElement::new( 0,  35,  74,   2,   2), //  0,  3,  0
	StateElement::new( 0,  36,  75,  29,   2), //  0,  3,  1
	StateElement::new( 0,  36,  75,  56,   2), //  0,  3,  2
	StateElement::new( 0,  37,  76,  76,   2), //  0,  3,  3
	StateElement::new( 0,  39,  90,   3,   3), //  0,  4,  0
	StateElement::new( 0,  40,  91,  30,   3), //  0,  4,  1
	StateElement::new( 0,  40,  91,  57,   3), //  0,  4,  2
	StateElement::new( 0,  41,  92,  77,   3), //  0,  4,  3
	StateElement::new( 0,  43, 102,   3,   3), //  0,  5,  0
	StateElement::new( 0,  44, 103,  30,   3), //  0,  5,  1
	StateElement::new( 0,  44, 103,  57,   3), //  0,  5,  2
	StateElement::new( 0,  47, 110,   3,   3), //  0,  6,  0
	StateElement::new( 0,  48, 111,  30,   3), //  0,  6,  1
	StateElement::new( 0,  50, 116,   3,   3), //  0,  7,  0
	StateElement::new( 0,  51, 117,  30,   3), //  0,  7,  1
	StateElement::new( 1,  54,   4,   4,   4), //  1,  0,  0
	StateElement::new( 1,  55,   5,  31,   4), //  1,  0,  1
	StateElement::new( 1,  55,   5,  58,   4), //  1,  0,  2
	StateElement::new( 1,  56,   6,  78,   4), //  1,  0,  3
	StateElement::new( 1,  58,  31,   5,   5), //  1,  1,  0
	StateElement::new( 1,  59,  32,  32,   5), //  1,  1,  1
	StateElement::new( 1,  59,  32,  59,   5), //  1,  1,  2
	StateElement::new( 1,  60,  33,  79,   5), //  1,  1,  3
	StateElement::new( 1,  58,  58,   5,   5), //  1,  2,  0
	StateElement::new( 1,  59,  59,  32,   5), //  1,  2,  1
	StateElement::new( 1,  59,  59,  59,   5), //  1,  2,  2
	StateElement::new( 1,  60,  60,  79,   5), //  1,  2,  3
	StateElement::new( 1,  62,  78,   6,   6), //  1,  3,  0
	StateElement::new( 1,  63,  79,  33,   6), //  1,  3,  1
	StateElement::new( 1,  63,  79,  60,   6), //  1,  3,  2
	StateElement::new( 1,  64,  80,  80,   6), //  1,  3,  3
	StateElement::new( 1,  65,  93,   7,   7), //  1,  4,  0
	StateElement::new( 1,  66,  94,  34,   7), //  1,  4,  1
	StateElement::new( 1,  66,  94,  61,   7), //  1,  4,  2
	StateElement::new( 1,  67,  95,  81,   7), //  1,  4,  3
	StateElement::new( 1,  68, 104,   7,   7), //  1,  5,  0
	StateElement::new( 1,  69, 105,  34,   7), //  1,  5,  1
	StateElement::new( 1,  69, 105,  61,   7), //  1,  5,  2
	StateElement::new( 1,  70, 112,   7,   7), //  1,  6,  0
	StateElement::new( 1,  71, 113,  34,   7), //  1,  6,  1
	StateElement::new( 1,  72, 118,   7,   7), //  1,  7,  0
	StateElement::new( 1,  73, 119,  34,   7), //  1,  7,  1
	StateElement::new( 2,  74,   8,   8,   8), //  2,  0,  0
	StateElement::new( 2,  75,   9,  35,   8), //  2,  0,  1
	StateElement::new( 2,  75,   9,  62,   8), //  2,  0,  2
	StateElement::new( 2,  76,  10,  82,   8), //  2,  0,  3
	StateElement::new( 2,  78,  35,   9,   9), //  2,  1,  0
	StateElement::new( 2,  79,  36,  36,   9), //  2,  1,  1
	StateElement::new( 2,  79,  36,  63,   9), //  2,  1,  2
	StateElement::new( 2,  80,  37,  83,   9), //  2,  1,  3
	StateElement::new( 2,  78,  62,   9,   9), //  2,  2,  0
	StateElement::new( 2,  79,  63,  36,   9), //  2,  2,  1
	StateElement::new( 2,  79,  63,  63,   9), //  2,  2,  2
	StateElement::new( 2,  82,  82,  10,  10), //  2,  3,  0
	StateElement::new( 2,  83,  83,  37,  10), //  2,  3,  1
	StateElement::new( 2,  83,  83,  64,  10), //  2,  3,  2
	StateElement::new( 2,  84,  96,  11,  11), //  2,  4,  0
	StateElement::new( 2,  85,  97,  38,  11), //  2,  4,  1
	StateElement::new( 2,  86, 106,  11,  11), //  2,  5,  0
	StateElement::new( 2,  87, 107,  38,  11), //  2,  5,  1
	StateElement::new( 2,  88, 114,  11,  11), //  2,  6,  0
	StateElement::new( 2,  89, 115,  38,  11), //  2,  6,  1
	StateElement::new( 3,  90,  12,  12,  12), //  3,  0,  0
	StateElement::new( 3,  91,  13,  39,  12), //  3,  0,  1
	StateElement::new( 3,  91,  13,  65,  12), //  3,  0,  2
	StateElement::new( 3,  92,  14,  84,  12), //  3,  0,  3
	StateElement::new( 3,  93,  39,  13,  13), //  3,  1,  0
	StateElement::new( 3,  94,  40,  40,  13), //  3,  1,  1
	StateElement::new( 3,  94,  40,  66,  13), //  3,  1,  2
	StateElement::new( 3,  95,  41,  85,  13), //  3,  1,  3
	StateElement::new( 3,  93,  65,  13,  13), //  3,  2,  0
	StateElement::new( 3,  94,  66,  40,  13), //  3,  2,  1
	StateElement::new( 3,  96,  84,  14,  14), //  3,  3,  0
	StateElement::new( 3,  97,  85,  41,  14), //  3,  3,  1
	StateElement::new( 3,  98,  98,  15,  15), //  3,  4,  0
	StateElement::new( 3,  99,  99,  42,  15), //  3,  4,  1
	StateElement::new( 3, 100, 108,  15,  15), //  3,  5,  0
	StateElement::new( 3, 101, 109,  42,  15), //  3,  5,  1
	StateElement::new( 4, 102,  16,  16,  16), //  4,  0,  0
	StateElement::new( 4, 103,  17,  43,  16), //  4,  0,  1
	StateElement::new( 4, 103,  17,  68,  16), //  4,  0,  2
	StateElement::new( 4, 104,  43,  17,  17), //  4,  1,  0
	StateElement::new( 4, 105,  44,  44,  17), //  4,  1,  1
	StateElement::new( 4, 105,  44,  69,  17), //  4,  1,  2
	StateElement::new( 4, 104,  68,  17,  17), //  4,  2,  0
	StateElement::new( 4, 105,  69,  44,  17), //  4,  2,  1
	StateElement::new( 4, 106,  86,  18,  18), //  4,  3,  0
	StateElement::new( 4, 107,  87,  45,  18), //  4,  3,  1
	StateElement::new( 4, 108, 100,  19,  19), //  4,  4,  0
	StateElement::new( 4, 109, 101,  46,  19), //  4,  4,  1
	StateElement::new( 5, 110,  20,  20,  20), //  5,  0,  0
	StateElement::new( 5, 111,  21,  47,  20), //  5,  0,  1
	StateElement::new( 5, 112,  47,  21,  21), //  5,  1,  0
	StateElement::new( 5, 113,  48,  48,  21), //  5,  1,  1
	StateElement::new( 5, 112,  70,  21,  21), //  5,  2,  0
	StateElement::new( 5, 113,  71,  48,  21), //  5,  2,  1
	StateElement::new( 5, 114,  88,  22,  22), //  5,  3,  0
	StateElement::new( 5, 115,  89,  49,  22), //  5,  3,  1
	StateElement::new( 6, 116,  23,  23,  23), //  6,  0,  0
	StateElement::new( 6, 117,  24,  50,  23), //  6,  0,  1
	StateElement::new( 6, 118,  50,  24,  24), //  6,  1,  0
	StateElement::new( 6, 119,  51,  51,  24), //  6,  1,  1
	StateElement::new( 6, 118,  72,  24,  24), //  6,  2,  0
	StateElement::new( 6, 119,  73,  51,  24), //  6,  2,  1
	StateElement::new( 7, 120,  25,  25,  25), //  7,  0,  0
	StateElement::new( 7, 121,  26,  52,  25), //  7,  0,  1
	StateElement::new( 7, 122,  52,  26,  26), //  7,  1,  0
	StateElement::new( 7, 123,  53,  53,  26), //  7,  1,  1
	StateElement::new( 8, 124,  25,  25,  25), //  8,  0,  0
	StateElement::new( 8, 125,  26,  52,  25), //  8,  0,  1
	StateElement::new( 8, 126,  52,  26,  26), //  8,  1,  0
	StateElement::new( 8, 127,  53,  53,  26), //  8,  1,  1
	StateElement::new( 9, 128,  25,  25,  25), //  9,  0,  0
	StateElement::new( 9, 129,  26,  52,  25), //  9,  0,  1
	StateElement::new( 9, 130,  52,  26,  26), //  9,  1,  0
	StateElement::new( 9, 131,  53,  53,  26), //  9,  1,  1
	StateElement::new(10, 132,  25,  25,  25), // 10,  0,  0
	StateElement::new(10, 133,  26,  52,  25), // 10,  0,  1
	StateElement::new(10, 134,  52,  26,  26), // 10,  1,  0
	StateElement::new(10, 135,  53,  53,  26), // 10,  1,  1
	StateElement::new(11, 136,  25,  25,  25), // 11,  0,  0
	StateElement::new(11, 137,  26,  52,  25), // 11,  0,  1
	StateElement::new(11, 138,  52,  26,  26), // 11,  1,  0
	StateElement::new(11, 139,  53,  53,  26), // 11,  1,  1
	StateElement::new(12, 140,  25,  25,  25), // 12,  0,  0
	StateElement::new(12, 141,  26,  52,  25), // 12,  0,  1
	StateElement::new(12, 142,  52,  26,  26), // 12,  1,  0
	StateElement::new(12, 143,  53,  53,  26), // 12,  1,  1
	StateElement::new(13, 144,  25,  25,  25), // 13,  0,  0
	StateElement::new(13, 145,  26,  52,  25), // 13,  0,  1
	StateElement::new(13, 146,  52,  26,  26), // 13,  1,  0
	StateElement::new(13, 147,  53,  53,  26), // 13,  1,  1
	StateElement::new(14, 148,  25,  25,  25), // 14,  0,  0
	StateElement::new(14, 149,  26,  52,  25), // 14,  0,  1
	StateElement::new(14, 150,  52,  26,  26), // 14,  1,  0
	StateElement::new(14, 151,  53,  53,  26), // 14,  1,  1
	StateElement::new(15, 152,  25,  25,  25), // 15,  0,  0
	StateElement::new(15, 153,  26,  52,  25), // 15,  0,  1
	StateElement::new(15, 154,  52,  26,  26), // 15,  1,  0
	StateElement::new(15, 155,  53,  53,  26), // 15,  1,  1
	StateElement::new(16, 156,  25,  25,  25), // 16,  0,  0
	StateElement::new(16, 157,  26,  52,  25), // 16,  0,  1
	StateElement::new(16, 158,  52,  26,  26), // 16,  1,  0
	StateElement::new(16, 159,  53,  53,  26), // 16,  1,  1
	StateElement::new(17, 160,  25,  25,  25), // 17,  0,  0
	StateElement::new(17, 161,  26,  52,  25), // 17,  0,  1
	StateElement::new(17, 162,  52,  26,  26), // 17,  1,  0
	StateElement::new(17, 163,  53,  53,  26), // 17,  1,  1
	StateElement::new(18, 164,  25,  25,  25), // 18,  0,  0
	StateElement::new(18, 165,  26,  52,  25), // 18,  0,  1
	StateElement::new(18, 166,  52,  26,  26), // 18,  1,  0
	StateElement::new(18, 167,  53,  53,  26), // 18,  1,  1
	StateElement::new(19, 168,  25,  25,  25), // 19,  0,  0
	StateElement::new(19, 169,  26,  52,  25), // 19,  0,  1
	StateElement::new(19, 170,  52,  26,  26), // 19,  1,  0
	StateElement::new(19, 171,  53,  53,  26), // 19,  1,  1
	StateElement::new(20, 172,  25,  25,  25), // 20,  0,  0
	StateElement::new(20, 173,  26,  52,  25), // 20,  0,  1
	StateElement::new(20, 174,  52,  26,  26), // 20,  1,  0
	StateElement::new(20, 175,  53,  53,  26), // 20,  1,  1
	StateElement::new(21, 176,  25,  25,  25), // 21,  0,  0
	StateElement::new(21, 177,  26,  52,  25), // 21,  0,  1
	StateElement::new(21, 178,  52,  26,  26), // 21,  1,  0
	StateElement::new(21, 179,  53,  53,  26), // 21,  1,  1
	StateElement::new(22, 180,  25,  25,  25), // 22,  0,  0
	StateElement::new(22, 181,  26,  52,  25), // 22,  0,  1
	StateElement::new(22, 182,  52,  26,  26), // 22,  1,  0
	StateElement::new(22, 183,  53,  53,  26), // 22,  1,  1
	StateElement::new(23, 184,  25,  25,  25), // 23,  0,  0
	StateElement::new(23, 185,  26,  52,  25), // 23,  0,  1
	StateElement::new(23, 186,  52,  26,  26), // 23,  1,  0
	StateElement::new(23, 187,  53,  53,  26), // 23,  1,  1
	StateElement::new(24, 188,  25,  25,  25), // 24,  0,  0
	StateElement::new(24, 189,  26,  52,  25), // 24,  0,  1
	StateElement::new(24, 190,  52,  26,  26), // 24,  1,  0
	StateElement::new(24, 191,  53,  53,  26), // 24,  1,  1
	StateElement::new(25, 192,  25,  25,  25), // 25,  0,  0
	StateElement::new(25, 193,  26,  52,  25), // 25,  0,  1
	StateElement::new(25, 194,  52,  26,  26), // 25,  1,  0
	StateElement::new(25, 195,  53,  53,  26), // 25,  1,  1
	StateElement::new(26, 196,  25,  25,  25), // 26,  0,  0
	StateElement::new(26, 197,  26,  52,  25), // 26,  0,  1
	StateElement::new(26, 198,  52,  26,  26), // 26,  1,  0
	StateElement::new(26, 199,  53,  53,  26), // 26,  1,  1
	StateElement::new(27, 200,  25,  25,  25), // 27,  0,  0
	StateElement::new(27, 201,  26,  52,  25), // 27,  0,  1
	StateElement::new(27, 202,  52,  26,  26), // 27,  1,  0
	StateElement::new(27, 203,  53,  53,  26), // 27,  1,  1
	StateElement::new(28, 204,  25,  25,  25), // 28,  0,  0
	StateElement::new(28, 205,  26,  52,  25), // 28,  0,  1
	StateElement::new(28, 206,  52,  26,  26), // 28,  1,  0
	StateElement::new(28, 207,  53,  53,  26), // 28,  1,  1
	StateElement::new(29, 208,  25,  25,  25), // 29,  0,  0
	StateElement::new(29, 209,  26,  52,  25), // 29,  0,  1
	StateElement::new(29, 210,  52,  26,  26), // 29,  1,  0
	StateElement::new(29, 211,  53,  53,  26), // 29,  1,  1
	StateElement::new(30, 212,  25,  25,  25), // 30,  0,  0
	StateElement::new(30, 213,  26,  52,  25), // 30,  0,  1
	StateElement::new(30, 214,  52,  26,  26), // 30,  1,  0
	StateElement::new(30, 215,  53,  53,  26), // 30,  1,  1
	StateElement::new(31, 216,  25,  25,  25), // 31,  0,  0
	StateElement::new(31, 217,  26,  52,  25), // 31,  0,  1
	StateElement::new(31, 218,  52,  26,  26), // 31,  1,  0
	StateElement::new(31, 219,  53,  53,  26), // 31,  1,  1
	StateElement::new(32, 220,  25,  25,  25), // 32,  0,  0
	StateElement::new(32, 220,  26,  52,  25), // 32,  0,  1
	StateElement::new(32, 220,  52,  26,  26), // 32,  1,  0
	StateElement::new(32, 220,  53,  53,  26), // 32,  1,  1
	StateElement::new(33, 221,  53,  53,  26), // 33,  1,  1
	StateElement::new(34, 222,  53,  53,  26), // 34,  1,  1
	StateElement::new(35, 223,  53,  53,  26), // 35,  1,  1
	StateElement::new(36, 224,  53,  53,  26), // 36,  1,  1
	StateElement::new(37, 225,  53,  53,  26), // 37,  1,  1
	StateElement::new(38, 226,  53,  53,  26), // 38,  1,  1
	StateElement::new(39, 227,  53,  53,  26), // 39,  1,  1
	StateElement::new(40, 228,  53,  53,  26), // 40,  1,  1
	StateElement::new(41, 229,  53,  53,  26), // 41,  1,  1
	StateElement::new(42, 230,  53,  53,  26), // 42,  1,  1
	StateElement::new(43, 231,  53,  53,  26), // 43,  1,  1
	StateElement::new(44, 232,  53,  53,  26), // 44,  1,  1
	StateElement::new(45, 233,  53,  53,  26), // 45,  1,  1
	StateElement::new(46, 234,  53,  53,  26), // 46,  1,  1
	StateElement::new(47, 235,  53,  53,  26), // 47,  1,  1
	StateElement::new(48, 236,  53,  53,  26), // 48,  1,  1
	StateElement::new(49, 237,  53,  53,  26), // 49,  1,  1
	StateElement::new(50, 238,  53,  53,  26), // 50,  1,  1
	StateElement::new(51, 239,  53,  53,  26), // 51,  1,  1
	StateElement::new(52, 240,  53,  53,  26), // 52,  1,  1
	StateElement::new(53, 241,  53,  53,  26), // 53,  1,  1
	StateElement::new(54, 242,  53,  53,  26), // 54,  1,  1
	StateElement::new(55, 243,  53,  53,  26), // 55,  1,  1
	StateElement::new(56, 244,  53,  53,  26), // 56,  1,  1
	StateElement::new(57, 245,  53,  53,  26), // 57,  1,  1
	StateElement::new(58, 246,  53,  53,  26), // 58,  1,  1
	StateElement::new(59, 247,  53,  53,  26), // 59,  1,  1
	StateElement::new(60, 248,  53,  53,  26), // 60,  1,  1
	StateElement::new(61, 249,  53,  53,  26), // 61,  1,  1
	StateElement::new(62, 250,  53,  53,  26), // 62,  1,  1
	StateElement::new(63, 251,  53,  53,  26), // 63,  1,  1
	StateElement::new(64, 252,  53,  53,  26), // 64,  1,  1
	StateElement::new(65, 253,  53,  53,  26), // 65,  1,  1
	StateElement::new(66, 254,  53,  53,  26), // 66,  1,  1
	StateElement::new(67, 254,  53,  53,  26), // 67,  1,  1
]);
