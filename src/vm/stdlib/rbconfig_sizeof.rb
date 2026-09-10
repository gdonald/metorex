# How wide the C types the interpreter was built against are, and the range
# each of them holds.

require 'rbconfig'

module RbConfig
  SIZEOF = {
    "int" => 4,
    "short" => 2,
    "long" => 8,
    "long long" => 8,
    "__int64" => 8,
    "off_t" => 8,
    "void*" => 8,
    "float" => 4,
    "double" => 8,
    "time_t" => 8,
    "clock_t" => 8,
    "size_t" => 8,
    "ptrdiff_t" => 8,
    "dev_t" => 4,
    "ino_t" => 8,
    "int8_t" => 1,
    "int16_t" => 2,
    "int32_t" => 4,
    "int64_t" => 8,
    "uint8_t" => 1,
    "uint16_t" => 2,
    "uint32_t" => 4,
    "uint64_t" => 8,
    "intptr_t" => 8,
    "uintptr_t" => 8
  }

  LIMITS = {
    "FIXNUM_MAX" => (2**62) - 1,
    "FIXNUM_MIN" => -(2**62),
    "CHAR_MIN" => -128,
    "CHAR_MAX" => 127,
    "SCHAR_MIN" => -128,
    "SCHAR_MAX" => 127,
    "UCHAR_MAX" => 255,
    "SHRT_MIN" => -32768,
    "SHRT_MAX" => 32767,
    "USHRT_MAX" => 65535,
    "INT_MIN" => -2147483648,
    "INT_MAX" => 2147483647,
    "UINT_MAX" => 4294967295,
    "LONG_MIN" => -(2**63),
    "LONG_MAX" => (2**63) - 1,
    "ULONG_MAX" => (2**64) - 1,
    "LLONG_MIN" => -(2**63),
    "LLONG_MAX" => (2**63) - 1,
    "ULLONG_MAX" => (2**64) - 1,
    "INT8_MIN" => -128,
    "INT8_MAX" => 127,
    "UINT8_MAX" => 255,
    "INT16_MIN" => -32768,
    "INT16_MAX" => 32767,
    "UINT16_MAX" => 65535,
    "INT32_MIN" => -2147483648,
    "INT32_MAX" => 2147483647,
    "UINT32_MAX" => 4294967295,
    "INT64_MIN" => -(2**63),
    "INT64_MAX" => (2**63) - 1,
    "UINT64_MAX" => (2**64) - 1,
    "FLT_MAX" => 3.4028234663852886e+38,
    "FLT_MIN" => 1.1754943508222875e-38,
    "DBL_MAX" => 1.7976931348623157e+308,
    "DBL_MIN" => 2.2250738585072014e-308
  }
end
