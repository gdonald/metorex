use super::run_example;

/// The expected output of both `c_extensions/c_methods` variants.
const C_METHODS_OUTPUT: &str = concat!(
    "true\n",
    "Object\n",
    "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]\n",
    "3\n",
    "[1, \"two\", :three]\n",
    "true\n",
    "true\n",
    "false\n",
    "[true, true, true, true, true, true, true, true]\n",
    "1180591620717411303424\n",
    "1.5\n",
    "[NilClass, TrueClass, FalseClass, Integer]\n",
    "true\n",
    "true\n",
    "true\n",
    "nil\n",
    "true\n",
    "inherited by CChild\n",
    "CChild\n",
    "ArgumentError: wrong number of arguments (given 1, expected 2)\n",
    "TypeError: superclass mismatch for class CMethods\n",
    "TypeError: superclass must be an instance of Class (given an instance of Integer)\n",
    "TypeError: RUBY_VERSION is not a class (String)\n",
    "ArgumentError: arity out of range: 16 for -2..15\n",
    "TypeError: 1 is not a class/module\n",
    "LoadError\n",
    "false\n",
    "LoadError\n",
);

#[test]
fn test_c_extensions_c_methods_execution() {
    let output = run_example("c_extensions/c_methods.rb");
    assert_eq!(output, C_METHODS_OUTPUT);
}

#[test]
fn test_c_extensions_c_methods_no_parens_execution() {
    let output = run_example("c_extensions/c_methods_no_parens.rb");
    assert_eq!(output, C_METHODS_OUTPUT);
}

/// The expected output of both `c_extensions/c_calls` variants.
const C_CALLS_OUTPUT: &str = concat!(
    ":name\n",
    "true\n",
    "false\n",
    "TypeError: wrong argument type String (expected symbol)\n",
    ":recorded\n",
    "[[1, \"two\"]]\n",
    "[1, 2, 3]\n",
    ":written_in_ruby\n",
    "RuntimeError: Cannot create Binding object for non-Ruby caller\n",
    "NoMethodError: undefined method 'no_such_method' for an instance of Integer\n",
    "Comparable\n",
    "Outer::Loaded\n",
    "NameError: uninitialized constant NoSuchConstant\n",
    "true\n",
    "true\n",
    "\"text\"\n",
    "true\n",
    "ArgumentError: wrong type argument Integer (should be callable)\n",
    "true\n",
    "[42, -7, 3, 5]\n",
    "TypeError: no implicit conversion from nil to integer\n",
    "RangeError: float inf out of range of integer\n",
    "RangeError: bignum too big to convert into 'long'\n",
    "TypeError: no implicit conversion of String into Integer\n",
    "[4611686018427387903, 2305843009213693952]\n",
    "RangeError: bignum out of range of unsigned long\n",
    "[2147483647, -2147483648]\n",
    "RangeError: integer 2147483648 too big to convert to 'int'\n",
    "RangeError: integer -2147483649 too small to convert to 'int'\n",
    "[4294967295, 4294967295]\n",
    "RangeError: integer 4294967296 too big to convert to 'unsigned int'\n",
    "RangeError: integer -2147483649 too small to convert to 'unsigned int'\n",
    "[-14, 42]\n",
    "Holder::Inner\n",
    "true\n",
    "TypeError: superclass mismatch for class Holder::Inner (String is given but was Object)\n",
    "TypeError: Holder::Taken is not a class (Integer)\n",
    "Holder::ById\n",
    "Parent\n",
    "Outer::Loaded\n",
    "CTopModule\n",
    "true\n",
    "CTopModule::Nested\n",
    "TypeError: String is not a module (Class)\n",
    "finalizer ran\n",
);

#[test]
fn test_c_extensions_c_calls_execution() {
    let output = run_example("c_extensions/c_calls.rb");
    assert_eq!(output, C_CALLS_OUTPUT);
}

#[test]
fn test_c_extensions_c_calls_no_parens_execution() {
    let output = run_example("c_extensions/c_calls_no_parens.rb");
    assert_eq!(output, C_CALLS_OUTPUT);
}

/// The expected output of both `c_extensions/c_files` variants.
const C_FILES_OUTPUT: &str = concat!(
    "[104, 195, 169, 108, 108, 111]\n",
    "[255, 0]\n",
    "[]\n",
    "true\n",
    "16\n",
    "true\n",
    "TypeError: wrong argument type Integer (expected String)\n",
    "true\n",
    "\"from_path\"\n",
    "\"from_str\"\n",
    "\"through_both\"\n",
    "TypeError: can't convert TextHolder to String (TextHolder#to_str gives Integer)\n",
    "TypeError: no implicit conversion of Integer into String\n",
    "TypeError: no implicit conversion of nil into String\n",
    "File\n",
    "\"written from C\"\n",
    "\"written from C\"\n",
    "ArgumentError: invalid access mode \n",
    "TypeError: no implicit conversion of Integer into String\n",
);

#[test]
fn test_c_extensions_c_files_execution() {
    let output = run_example("c_extensions/c_files.rb");
    assert_eq!(output, C_FILES_OUTPUT);
}

#[test]
fn test_c_extensions_c_files_no_parens_execution() {
    let output = run_example("c_extensions/c_files_no_parens.rb");
    assert_eq!(output, C_FILES_OUTPUT);
}

/// The expected output of both `c_extensions/c_arguments` variants.
const C_ARGUMENTS_OUTPUT: &str = concat!(
    "[1, nil]\n",
    "[1, 2]\n",
    "ArgumentError: wrong number of arguments (given 0, expected 1..2)\n",
    "ArgumentError: wrong number of arguments (given 3, expected 1..2)\n",
    "[1, 2]\n",
    "ArgumentError: wrong number of arguments (given 1, expected 2)\n",
    "[1, [], 2]\n",
    "[1, [2, 3], 4]\n",
    "ArgumentError: wrong number of arguments (given 1, expected 2+)\n",
    "[nil, nil]\n",
    "[1, 2]\n",
    "ArgumentError: wrong number of arguments (given 3, expected 0..2)\n",
    "[]\n",
    "[1, 2]\n",
    "3\n",
    "RuntimeError: bad scan arg format: 1x\n",
    "[1, \"two\", :three, nil]\n",
    "true\n",
    "[]\n",
    "TypeError: wrong argument type String (expected Array)\n",
    "Enumerator\n",
    "[1, 2, 3]\n",
    "nil\n",
    "[2, 3, 4]\n",
    "true\n",
    "[2, 4]\n",
    "true\n",
    "nil\n",
);

#[test]
fn test_c_extensions_c_arguments_execution() {
    let output = run_example("c_extensions/c_arguments.rb");
    assert_eq!(output, C_ARGUMENTS_OUTPUT);
}

#[test]
fn test_c_extensions_c_arguments_no_parens_execution() {
    let output = run_example("c_extensions/c_arguments_no_parens.rb");
    assert_eq!(output, C_ARGUMENTS_OUTPUT);
}

/// The expected output of both `c_extensions/c_integers` variants.
const C_INTEGERS_OUTPUT: &str = concat!(
    "\"aXc\"\n",
    "[255, 88, 99]\n",
    "false\n",
    "[1, 254]\n",
    "\"Ayz\"\n",
    "true\n",
    "[103, 114, 111, 120]\n",
    "[65, 66]\n",
    "[0, [0, 0, 0, 0]]\n",
    "[1, [0, 0, 1, 2]]\n",
    "[1, [2, 1, 0, 0]]\n",
    "true\n",
    "[1, [2, 1, 4, 3]]\n",
    "[1, [3, 4, 1, 2]]\n",
    "[-1, [255, 255]]\n",
    "[-1, [0, 1]]\n",
    "[-1, [0, 0]]\n",
    "[-2, [0, 0]]\n",
    "[-2, [255, 255]]\n",
    "[2, [0, 0]]\n",
    "[2, [15, 15]]\n",
    "[-1, [15, 15]]\n",
    "[1, [2]]\n",
    "[1, [1, 2]]\n",
    "ArgumentError: unsupported flags specified\n",
    "ArgumentError: word order not specified\n",
    "ArgumentError: unexpected word order\n",
    "ArgumentError: byte order not specified\n",
    "ArgumentError: unexpected byte order\n",
    "ArgumentError: invalid wordsize: 0\n",
    "ArgumentError: too big nails: 8\n",
    "TypeError: no implicit conversion of String into Integer\n",
    "1024\n",
    "-27\n",
    "22539340290692258087863249\n",
    "17\n",
    "1\n",
    "example.rb:75: warning: rb_define_const: invalid name 'lower' for constant\n",
    "FrozenError: can't modify frozen Module: Holder\n",
    "FrozenError: can't modify frozen Module: Holder\n",
);

#[test]
fn test_c_extensions_c_integers_execution() {
    let output = run_example("c_extensions/c_integers.rb");
    assert_eq!(output, C_INTEGERS_OUTPUT);
}

#[test]
fn test_c_extensions_c_integers_no_parens_execution() {
    let output = run_example("c_extensions/c_integers_no_parens.rb");
    assert_eq!(output, C_INTEGERS_OUTPUT);
}

/// The expected output of both `c_extensions/c_numerics` variants.
const C_NUMERICS_OUTPUT: &str = concat!(
    "0.5\n",
    "2.5\n",
    "TypeError: wrong argument type Integer (expected Float)\n",
    "[true, false, false]\n",
    "2.5\n",
    "3.0\n",
    "ArgumentError: invalid value for Float(): \"many\"\n",
    "(1+2i)\n",
    "(3+4i)\n",
    "(5+0i)\n",
    "(10+4i)\n",
    "(6+0i)\n",
    "(1.5+2i)\n",
    "(1/2)\n",
    "(3/4)\n",
    "(5/1)\n",
    "(5/2)\n",
    "(6/1)\n",
    "ZeroDivisionError: divided by 0\n",
    "[7, 2]\n",
);

#[test]
fn test_c_extensions_c_numerics_execution() {
    let output = run_example("c_extensions/c_numerics.rb");
    assert_eq!(output, C_NUMERICS_OUTPUT);
}

#[test]
fn test_c_extensions_c_numerics_no_parens_execution() {
    let output = run_example("c_extensions/c_numerics_no_parens.rb");
    assert_eq!(output, C_NUMERICS_OUTPUT);
}

/// The expected output of both `c_extensions/c_tracepoints` variants.
const C_TRACEPOINTS_OUTPUT: &str = concat!(
    "TracePoint\n",
    "false\n",
    "true\n",
    "false\n",
    "[:call, :return]\n",
    "[:line]\n",
    "[]\n",
);

#[test]
fn test_c_extensions_c_tracepoints_execution() {
    let output = run_example("c_extensions/c_tracepoints.rb");
    assert_eq!(output, C_TRACEPOINTS_OUTPUT);
}

#[test]
fn test_c_extensions_c_tracepoints_no_parens_execution() {
    let output = run_example("c_extensions/c_tracepoints_no_parens.rb");
    assert_eq!(output, C_TRACEPOINTS_OUTPUT);
}

/// The expected output of both `c_extensions/c_fibers` variants.
const C_FIBERS_OUTPUT: &str = concat!(
    "Fiber\n",
    "[1, :data, 2, [1, 2]]\n",
    "false\n",
    "[nil, :none, 0, []]\n",
    "true\n",
    "40\n",
    "6\n",
    "false\n",
    "42\n",
    "\"rescued stopped\"\n",
    "[1, 3, nil, nil]\n",
);

#[test]
fn test_c_extensions_c_fibers_execution() {
    let output = run_example("c_extensions/c_fibers.rb");
    assert_eq!(output, C_FIBERS_OUTPUT);
}

#[test]
fn test_c_extensions_c_fibers_no_parens_execution() {
    let output = run_example("c_extensions/c_fibers_no_parens.rb");
    assert_eq!(output, C_FIBERS_OUTPUT);
}

/// The expected output of both `c_extensions/c_blocks` variants.
const C_BLOCKS_OUTPUT: &str = concat!(
    "false\n",
    "true\n",
    "10\n",
    "[]\n",
    "3\n",
    "[4, 3]\n",
    "ArgumentError: not an array\n",
    "30\n",
    "ArgumentError: not an array\n",
    "TypeError: can't convert Pair to Array (Pair#to_ary gives Integer)\n",
    "LocalJumpError: no block given (yield)\n",
    "101\n",
    ":finished\n",
    "Set[1, 3, 4, 5]\n",
    "[1, 2, 3, 4]\n",
    "Set[]\n",
    "[true, false, true, false, 1, true, false, Set[]]\n",
    "Set[]\n",
    "[-5, 18446744073709551615, -9223372036854775808, 9223372036854775808, 7, -7]\n",
);

#[test]
fn test_c_extensions_c_blocks_execution() {
    let output = run_example("c_extensions/c_blocks.rb");
    assert_eq!(output, C_BLOCKS_OUTPUT);
}

#[test]
fn test_c_extensions_c_blocks_no_parens_execution() {
    let output = run_example("c_extensions/c_blocks_no_parens.rb");
    assert_eq!(output, C_BLOCKS_OUTPUT);
}

/// The expected output of both `c_extensions/c_regexps` variants.
const C_REGEXPS_OUTPUT: &str = concat!(
    "/b+/i\n",
    "1\n",
    "[255]\n",
    "/x(y)/\n",
    "6\n",
    "2\n",
    "\"b\"\n",
    "nil\n",
    "nil\n",
    "[\"a\", nil, \"c\", nil]\n",
    "[\"c\", \"a\", nil, nil]\n",
    "\"k\"\n",
    "nil\n",
    "[0, -1, 0]\n",
    "\"converted\"\n",
    "TypeError: no implicit conversion of Integer into String\n",
    "65\n",
    "4\n",
    "ArgumentError: string contains null byte\n",
    "ArgumentError: string contains null char\n",
    "1\n",
);

#[test]
fn test_c_extensions_c_regexps_execution() {
    let output = run_example("c_extensions/c_regexps.rb");
    assert_eq!(output, C_REGEXPS_OUTPUT);
}

#[test]
fn test_c_extensions_c_regexps_no_parens_execution() {
    let output = run_example("c_extensions/c_regexps_no_parens.rb");
    assert_eq!(output, C_REGEXPS_OUTPUT);
}

/// The expected output of both `c_extensions/c_mutexes` variants.
const C_MUTEXES_OUTPUT: &str = concat!(
    "Thread::Mutex\n",
    "false\n",
    "true\n",
    "false\n",
    "ThreadError: deadlock; recursive locking\n",
    "true\n",
    "ThreadError: Attempt to unlock a mutex which is not locked\n",
    "true\n",
    "Integer\n",
    "true\n",
    "ThreadError: Attempt to unlock a mutex which is not locked\n",
    "true\n",
    "ArgumentError: stopped inside\n",
    "false\n",
);

#[test]
fn test_c_extensions_c_mutexes_execution() {
    let output = run_example("c_extensions/c_mutexes.rb");
    assert_eq!(output, C_MUTEXES_OUTPUT);
}

#[test]
fn test_c_extensions_c_mutexes_no_parens_execution() {
    let output = run_example("c_extensions/c_mutexes_no_parens.rb");
    assert_eq!(output, C_MUTEXES_OUTPUT);
}

/// The expected output of both `c_extensions/c_times` variants.
const C_TIMES_OUTPUT: &str = concat!(
    "[102, 500000, false]\n",
    "[99, 999999999]\n",
    "[1, 500000000, 7200]\n",
    "true\n",
    "[3, 500000000]\n",
    "[5, 7, 3600, false]\n",
    "true\n",
    "[5, false]\n",
    "ArgumentError: utc_offset out of range\n",
    "true\n",
    "[12, 0]\n",
    "[1, 250000]\n",
    "[1, 250000]\n",
    "ArgumentError: time interval must not be negative\n",
    "ArgumentError: time interval must not be negative\n",
    "ArgumentError: time interval must not be negative\n",
    "TypeError: can't convert Time into time interval\n",
    "[-2, 500000]\n",
    "[-2, 500000000]\n",
    "[1, 0]\n",
    "[-2, 0]\n",
    "[-2, 500000000]\n",
    "[3, 500000000]\n",
    "[4, 250]\n",
    "[4, 250]\n",
    "RangeError: 1000000000000000019884624838656.000000 out of Time range\n",
    "TypeError: can't convert String into time\n",
    "TypeError: can't convert nil into time\n",
    "TypeError: can't convert Broken into time\n",
    "[1, 2]\n",
    "FrozenError: can't modify frozen Array: []\n",
);

#[test]
fn test_c_extensions_c_times_execution() {
    let output = run_example("c_extensions/c_times.rb");
    assert_eq!(output, C_TIMES_OUTPUT);
}

#[test]
fn test_c_extensions_c_times_no_parens_execution() {
    let output = run_example("c_extensions/c_times_no_parens.rb");
    assert_eq!(output, C_TIMES_OUTPUT);
}

/// The expected output of both `c_extensions/c_tables` variants.
const C_TABLES_OUTPUT: &str = concat!(
    "[0, 1, 1, 11, 0, 4, [111, 220, 330], 3, 1, 40, 0, 0, 1, 0, 0]\n",
    "[1, 0, 1]\n",
    "[1, 1]\n",
);

#[test]
fn test_c_extensions_c_tables_execution() {
    let output = run_example("c_extensions/c_tables.rb");
    assert_eq!(output, C_TABLES_OUTPUT);
}

#[test]
fn test_c_extensions_c_tables_no_parens_execution() {
    let output = run_example("c_extensions/c_tables_no_parens.rb");
    assert_eq!(output, C_TABLES_OUTPUT);
}

/// The expected output of both `c_extensions/c_exceptions` variants.
const C_EXCEPTIONS_OUTPUT: &str = concat!(
    "nil\n",
    "\"outer\"\n",
    "true\n",
    "nil\n",
    "TypeError: assigning non-exception to $!\n",
    "#<ArgumentError: bytes>\n",
    "#<IOError: c string>\n",
    "#<TypeError: worded>\n",
    "TypeError: no implicit conversion of Integer into String\n",
    "IndexError: raised from C\n",
    "[\"second\", \"first\"]\n",
    "FrozenError: can't modify frozen Array: [1]\n",
    "FrozenError: can't modify frozen Looping:  ...\n",
    "\"text\"\n",
    "#<Errno::ENOENT: No such file or directory>\n",
    "#<Errno::ENOENT: No such file or directory - custom>\n",
    "Errno::EACCES\n",
    "\"Permission denied - custom\"\n",
    "nil\n",
    "#<RuntimeError: plain>\n",
    "#<RuntimeError: worded>\n",
    "true\n",
    "#<ArgumentError: with class>\n",
    "[\"here:1\"]\n",
    "TypeError: exception class/object expected\n",
    "TypeError: exception class/object expected\n",
    "TypeError: exception object expected\n",
    "ArgumentError: wrong number of arguments (given 4, expected 0..3)\n",
    "[StandardError, ZeroDivisionError, Encoding::CompatibilityError, Comparable, fatal]\n",
    "true\n",
);

#[test]
fn test_c_extensions_c_exceptions_execution() {
    let output = run_example("c_extensions/c_exceptions.rb");
    assert_eq!(output, C_EXCEPTIONS_OUTPUT);
}

#[test]
fn test_c_extensions_c_exceptions_no_parens_execution() {
    let output = run_example("c_extensions/c_exceptions_no_parens.rb");
    assert_eq!(output, C_EXCEPTIONS_OUTPUT);
}

/// The expected output of both `c_extensions/c_ranges` variants.
const C_RANGES_OUTPUT: &str = concat!(
    "1..4\n",
    "[\"a\", \"b\"]\n",
    "true\n",
    "ArgumentError: bad value for range\n",
    "[3, 9, true]\n",
    "[2, 6, true]\n",
    "false\n",
    "false\n",
    "[true, 2, 4]\n",
    "[true, 2, 3]\n",
    "[true, 7, 3]\n",
    "[true, 0, 9]\n",
    "[true, 8, 2]\n",
    "[nil, -1, -1]\n",
    "[true, 12, 9]\n",
    "RangeError: 12..20 out of range\n",
    "RangeError: -20..2 out of range\n",
    "[true, 2, 4]\n",
    "[false, -1, -1]\n",
    "[1, 10, 3, false]\n",
    "[1, 10, 2, true]\n",
    "[3, 7, 1, false]\n",
    "[2, 6, 1, true]\n",
    "false\n",
    "[true, 1, 9, 2]\n",
    "RangeError: ((1..12).step(2)) out of range\n",
    "RangeError: ((-20..2).step(2)) out of range\n",
    "[true, 2, 7, -2]\n",
    "[true, 9, 0, -1]\n",
    "[nil, -1, -1, 1]\n",
    "[false, -1, -1, 0]\n",
    "[1, 2, nil, nil, :later]\n",
    "[1, :last]\n",
    "IndexError: index -3 too small for array; minimum: -2\n",
);

#[test]
fn test_c_extensions_c_ranges_execution() {
    let output = run_example("c_extensions/c_ranges.rb");
    assert_eq!(output, C_RANGES_OUTPUT);
}

#[test]
fn test_c_extensions_c_ranges_no_parens_execution() {
    let output = run_example("c_extensions/c_ranges_no_parens.rb");
    assert_eq!(output, C_RANGES_OUTPUT);
}

/// The expected output of both `c_extensions/c_formats` variants.
const C_FORMATS_OUTPUT: &str = concat!(
    "\"-7|   42|3   |00009|123456789012|-5|4000000000|ff|FF|10|17|-2|z|%\"\n",
    "\"3.14|1.500000e+03|0.0001|   2.500|1.250000\"\n",
    "\"plain|abc|ab    |[café]|[\\\"café\\\"]\"\n",
    "#<Encoding:UTF-8>\n",
    "\"plain|abc|ab    |[sym]|[:sym]\"\n",
    "#<Encoding:BINARY (ASCII-8BIT)>\n",
    "true\n",
    "\"y and \"\n",
    "\"one and 2 more\"\n",
    "ArgumentError: bad [1, 2] at 7\n",
    "KeyError: bad  at 7\n",
    "example.rb:31: warning: careful with this\n",
    "example.rb:33: warning: careful with that\n",
    "example.rb:33: warning: only when verbose: 1\n",
    "[\"a\\x00b\", \"c string\", \"\"]\n",
    "[#<Encoding:BINARY (ASCII-8BIT)>]\n",
    "[\"12\", \"12\"]\n",
    "[\"text\", \"\\\"text\\\"\"]\n",
    "true\n",
);

#[test]
fn test_c_extensions_c_formats_execution() {
    let output = run_example("c_extensions/c_formats.rb");
    assert_eq!(output, C_FORMATS_OUTPUT);
}

#[test]
fn test_c_extensions_c_formats_no_parens_execution() {
    let output = run_example("c_extensions/c_formats_no_parens.rb");
    assert_eq!(output, C_FORMATS_OUTPUT);
}

/// The expected output of both `c_extensions/c_structs` variants.
const C_STRUCTS_OUTPUT: &str = concat!(
    "Struct::Pair\n",
    "[:left, :right]\n",
    "nil\n",
    "NameError: identifier lower needs to be constant\n",
    "ArgumentError: duplicate member: same\n",
    "Holder::Inner\n",
    "true\n",
    "TypeError: superclass mismatch for class Holder::Plain (Struct is given but was Object)\n",
    "Data\n",
    "true\n",
    "TypeError: wrong argument type Array (expected Class)\n",
    "ArgumentError: duplicate member: width\n",
    "#<struct Struct::Pair left=1, right=2>\n",
    "#<struct Struct::Pair left=3, right=nil>\n",
    "[:left, :right]\n",
    "[:left, :right]\n",
    "2\n",
    "[1, 2, 2]\n",
    "10\n",
    "10\n",
    "NameError: no member 'missing' in struct\n",
    "IndexError: offset 2 too large for struct(size:2)\n",
    "IndexError: offset -3 too small for struct(size:2)\n",
    "2\n",
    "NameError: 'missing' is not a struct member\n",
    "nil\n",
    "#<struct Struct::Pair left=7, right=8>\n",
    "ArgumentError: struct size differs\n",
    "FrozenError: can't modify frozen Struct::Pair: #<struct Struct::Pair left=7, right=8>\n",
    "nil\n",
    "[4, nil, true]\n",
    "FrozenError: can't modify frozen Shape: #<data Shape width=4, height=nil>\n",
    "ArgumentError: struct size differs\n",
);

#[test]
fn test_c_extensions_c_structs_execution() {
    let output = run_example("c_extensions/c_structs.rb");
    assert_eq!(output, C_STRUCTS_OUTPUT);
}

#[test]
fn test_c_extensions_c_structs_no_parens_execution() {
    let output = run_example("c_extensions/c_structs_no_parens.rb");
    assert_eq!(output, C_STRUCTS_OUTPUT);
}

/// The expected output of both `c_extensions/c_bignums` variants.
const C_BIGNUMS_OUTPUT: &str = concat!(
    "[9223372036854775807, -9223372036854775808, 5]\n",
    "RangeError: bignum too big to convert into 'long'\n",
    "RangeError: bignum too big to convert into 'long'\n",
    "RangeError: bignum too big to convert into 'long'\n",
    "-9223372036854775808\n",
    "RangeError: bignum too big to convert into 'long long'\n",
    "RangeError: bignum too big to convert into 'long long'\n",
    "[18446744073709551615, 18446744073709551615, 9223372036854775808]\n",
    "RangeError: bignum out of range of unsigned long\n",
    "RangeError: bignum too big to convert into 'unsigned long'\n",
    "1.1805916207174113e+21\n",
    "-Infinity\n",
    "warning: Integer out of Float range\n",
    "219238102380912836608\n",
    "-2\n",
    "FloatDomainError: Infinity\n",
    "FloatDomainError: -Infinity\n",
    "FloatDomainError: NaN\n",
    "\"1180591620717411303424\"\n",
    "\"400000000000000000\"\n",
    "[[1, true, false], [0, false, true], [1, true, false]]\n",
    "[-1, 1, 0]\n",
    "[18446744073709551615, 0]\n",
    "[18446744073709551615, 18446744073709551487]\n",
    "[5]\n",
    "[[9, 0, 9], [1, 0, 1], [2, 7, 2], [0, 0, 0]]\n",
    "[3.0, 1.1805916207174113e+21, 1.5, 0.25]\n",
    "2.5\n",
    "TypeError: no implicit conversion of nil into Float\n",
    "TypeError: no implicit conversion of true into Float\n",
    "TypeError: no implicit conversion of Symbol into Float\n",
    "TypeError: no implicit conversion of String into Float\n",
    "TypeError: can't convert Object into Float\n",
    "TypeError: can't convert Wrong to Float (Wrong#to_f gives String)\n",
    "[]\n",
);

#[test]
fn test_c_extensions_c_bignums_execution() {
    let output = run_example("c_extensions/c_bignums.rb");
    assert_eq!(output, C_BIGNUMS_OUTPUT);
}

#[test]
fn test_c_extensions_c_bignums_no_parens_execution() {
    let output = run_example("c_extensions/c_bignums_no_parens.rb");
    assert_eq!(output, C_BIGNUMS_OUTPUT);
}

/// The expected output of both `c_extensions/c_symbols` variants.
const C_SYMBOLS_OUTPUT: &str = concat!(
    "[true, false]\n",
    ":abc\n",
    "[:Ω, #<Encoding:UTF-8>]\n",
    "#<Encoding:US-ASCII>\n",
    ":made_constant\n",
    "[true, false]\n",
    "\"named\"\n",
    "#<Encoding:UTF-16LE>\n",
    ":from_string\n",
    ":named\n",
    "nil\n",
    "false\n",
    "[[true, false, false], [false, true, false], [false, false, true], [false, false, false], [false, false, false]]\n",
    "\"text\"\n",
    "[:held, :given, :spelled]\n",
    "TypeError: 5 is not a symbol nor a string\n",
    "[\"UTF-8\", \"US-ASCII\", \"US-ASCII\", \"EUC-JP\", nil]\n",
    "nil\n",
    "[#<Encoding:UTF-8>, #<Encoding:US-ASCII>, #<Encoding:BINARY (ASCII-8BIT)>]\n",
);

#[test]
fn test_c_extensions_c_symbols_execution() {
    let output = run_example("c_extensions/c_symbols.rb");
    assert_eq!(output, C_SYMBOLS_OUTPUT);
}

#[test]
fn test_c_extensions_c_symbols_no_parens_execution() {
    let output = run_example("c_extensions/c_symbols_no_parens.rb");
    assert_eq!(output, C_SYMBOLS_OUTPUT);
}

/// The expected output of both `c_extensions/c_utilities` variants.
const C_UTILITIES_OUTPUT: &str = concat!(
    "[1, 1, {size: 2}, nil, true]\n",
    "ArgumentError: wrong number of arguments (given 2, expected 0..1)\n",
    "[0, nil, {size: 2}]\n",
    "[0, nil, nil, true, false]\n",
    "ArgumentError: wrong number of arguments (given 2, expected 0..1)\n",
    "[nil, {size: 2}]\n",
    "[3, nil]\n",
    "ArgumentError: tried to create Proc object without a block\n",
    "true\n",
    "[2, 2, 1]\n",
    "{c: 3}\n",
    "[1, 1, nil]\n",
    "0\n",
    "ArgumentError: missing keywords: :a, :b\n",
    "ArgumentError: unknown keywords: :x, :y\n",
    "ArgumentError: unknown keyword: :x\n",
    "1\n",
    "ArgumentError: unknown keyword: :b\n",
    ":stopped\n",
    "nil\n",
    "[1, 2]\n",
    "[\"c_utilities.rb\", true]\n",
    "2147483647\n",
    "RangeError: integer 2147483648 too big to convert to 'int'\n",
    "RangeError: integer -2147483649 too small to convert to 'int'\n",
    "[\"14.25test\", [14.25, \"test\"]]\n",
    "[\"  -3e2x\", [-300.0, \"x\"]]\n",
    "[\"+\", [0.0, \"+\"]]\n",
    "[\"test\", [0.0, \"test\"]]\n",
    "[\"1e\", [1.0, \"e\"]]\n",
    "[\"0.\", [0.0, \"\"]]\n",
    "[\"000\", [0.0, \"\"]]\n",
    "[\".5\", [0.5, \"\"]]\n",
    "[\"0x1Ap1rest\", [52.0, \"rest\"]]\n",
    "[\"0x.8\", [0.5, \"\"]]\n",
    "[\"0x\", [0.0, \"0x\"]]\n",
    "[\"0xg\", [0.0, \"0xg\"]]\n",
    "[\"1e400\", [Infinity, \"\"]]\n",
    "[\"0x0\", [0.0, \"\"]]\n",
    "[\"-0x10p-1\", [-8.0, \"\"]]\n",
    "[{kept: 1}, false]\n",
);

#[test]
fn test_c_extensions_c_utilities_execution() {
    let output = run_example("c_extensions/c_utilities.rb");
    assert_eq!(output, C_UTILITIES_OUTPUT);
}

#[test]
fn test_c_extensions_c_utilities_no_parens_execution() {
    let output = run_example("c_extensions/c_utilities_no_parens.rb");
    assert_eq!(output, C_UTILITIES_OUTPUT);
}

/// The expected output of both `c_extensions/c_coercion` variants.
const C_COERCION_OUTPUT: &str = concat!(
    "ArgumentError: comparison of Integer with nil failed\n",
    "ArgumentError: comparison of String with 2.5 failed\n",
    "ArgumentError: comparison of Symbol with :b failed\n",
    "ArgumentError: comparison of Array with Object failed\n",
    "ZeroDivisionError: divided by 0\n",
    "[1, -1, 0, 1, -1, 0]\n",
    "[1, -1, 0, 1]\n",
    "ArgumentError: comparison of Integer with 2 failed: comparator returned nil\n",
    "15\n",
    "TypeError: Object can't be coerced into Integer\n",
    "TypeError: nil can't be coerced into Integer\n",
    "TypeError: coerce must return [x, y]\n",
    "TypeError: coerce must return [x, y]\n",
    "-1\n",
    "nil\n",
    "nil\n",
    "true\n",
    "ArgumentError: comparison of Integer with Pairing failed: coercion was not possible\n",
    "NoMethodError: undefined method '<' for an instance of Object\n",
    "[42, 3]\n",
    "ArgumentError: invalid value for Integer(): \"forty\"\n",
    "[32767, -32768, 4]\n",
    "RangeError: integer 32768 too big to convert to 'short'\n",
    "RangeError: integer -32769 too small to convert to 'short'\n",
    "[65, 124, 255]\n",
    "TypeError: no implicit conversion of String into Integer\n",
    "[0, 1, 1, 0, 1, 0]\n",
    "8\n",
);

#[test]
fn test_c_extensions_c_coercion_execution() {
    let output = run_example("c_extensions/c_coercion.rb");
    assert_eq!(output, C_COERCION_OUTPUT);
}

#[test]
fn test_c_extensions_c_coercion_no_parens_execution() {
    let output = run_example("c_extensions/c_coercion_no_parens.rb");
    assert_eq!(output, C_COERCION_OUTPUT);
}

/// The expected output of both `c_extensions/c_procs` variants.
const C_PROCS_OUTPUT: &str = concat!(
    "[1, :data, 2, [1, 2], false]\n",
    "[[1, 2], :data, 1, [[1, 2]], false]\n",
    "[nil, :data, 0, [], false]\n",
    "[nil, :data, 0, [], false]\n",
    "[-1, false, nil]\n",
    "true\n",
    "false\n",
    "true\n",
    "nil\n",
    "[2, -2, -1]\n",
    "[true, true, true, false, false]\n",
    "42\n",
    "TypeError: wrong argument type Integer (expected Array)\n",
    "[[], {}]\n",
    "[[{b: 2}], {a: 1}]\n",
    "[[], {}]\n",
    "TypeError: no implicit conversion of Integer into Hash\n",
    "84\n",
    "42\n",
    "[[], {a: 1}, 42]\n",
    "[[], {}, nil]\n",
);

#[test]
fn test_c_extensions_c_procs_execution() {
    let output = run_example("c_extensions/c_procs.rb");
    assert_eq!(output, C_PROCS_OUTPUT);
}

#[test]
fn test_c_extensions_c_procs_no_parens_execution() {
    let output = run_example("c_extensions/c_procs_no_parens.rb");
    assert_eq!(output, C_PROCS_OUTPUT);
}

/// The expected output of both `c_extensions/c_collection` variants.
const C_COLLECTION_OUTPUT: &str = concat!(
    "[\"kept text\", [\"kept text\"]]\n",
    "nil\n",
    "[false, true, true, false]\n",
    "2\n",
    ":method\n",
    "ArgumentError: unknown key: unknown\n",
    "TypeError: non-hash or symbol given\n",
    "[42, 42]\n",
);

#[test]
fn test_c_extensions_c_collection_execution() {
    let output = run_example("c_extensions/c_collection.rb");
    assert_eq!(output, C_COLLECTION_OUTPUT);
}

#[test]
fn test_c_extensions_c_collection_no_parens_execution() {
    let output = run_example("c_extensions/c_collection_no_parens.rb");
    assert_eq!(output, C_COLLECTION_OUTPUT);
}

/// The expected output of both `c_extensions/c_globals` variants.
const C_GLOBALS_OUTPUT: &str = concat!(
    "\"stored in C\"\n",
    "\"written from Ruby\"\n",
    "15\n",
    "NameError: $c_fixed is a read-only variable\n",
    "42\n",
    "\"written from Ruby\"\n",
    "nil\n",
    "nil\n",
    "false\n",
    "NameError: $c_virtual is a read-only variable\n",
    "[:$c_counting, 0]\n",
    "[:$c_counting, 1]\n",
    "[:$c_counting, 100]\n",
    "[[4], 8]\n",
    "[8, 8]\n",
    ":set\n",
    ":set\n",
    ":set\n",
    "true\n",
    "\"the last line\"\n",
    "\"the last line\"\n",
    "[nil, \"\\n\", nil, nil, \"\\n\"]\n",
    "[\",\", \"\\n\", \"-\", \"!\", \"\\n\"]\n",
    "true\n",
    "[0, 1, 2, 1]\n",
);

#[test]
fn test_c_extensions_c_globals_execution() {
    let output = run_example("c_extensions/c_globals.rb");
    assert_eq!(output, C_GLOBALS_OUTPUT);
}

#[test]
fn test_c_extensions_c_globals_no_parens_execution() {
    let output = run_example("c_extensions/c_globals_no_parens.rb");
    assert_eq!(output, C_GLOBALS_OUTPUT);
}

/// The expected output of both `c_extensions/c_modules` variants.
const C_MODULES_OUTPUT: &str = concat!(
    "[[true, true], [true, false], [true, false]]\n",
    "[:from_base, String]\n",
    "[:own, \"missing INHERITED\"]\n",
    "[:from_base, \"missing String\"]\n",
    "[7, [true, true]]\n",
    "NameError: uninitialized constant Base::_hidden\n",
    "NameError: wrong constant name _hidden\n",
    "[:replaced, true]\n",
    "42\n",
    "FrozenError: can't modify frozen Module: FrozenPlain\n",
    "[:original, :original]\n",
    "[:answered, 1, 3, [1, 2]]\n",
    "[0, 1, -1, -1]\n",
    "[true, true]\n",
    "NoMethodError: private method 'hidden' called for an instance of Holder\n",
    ":answered\n",
    ":answered\n",
    "NoMethodError: undefined method 'only_here' for an instance of Object\n",
    "[:answered, 0, true]\n",
    ":answered\n",
    "NoMethodError: undefined method 'removed' for an instance of Undone\n",
    "[]\n",
    "NoMethodError: undefined method 'initialize_copy' for an instance of Undone\n",
    "FrozenError: can't modify frozen class: FrozenUndone\n",
    "NameError: undefined method 'not_there' for class 'Undone'\n",
    "[\"Derived\", \"Derived\", \"Derived\"]\n",
    "Comparable\n",
    "\"AutoloadHolder::FromExtension\"\n",
    "false\n",
);

#[test]
fn test_c_extensions_c_modules_execution() {
    let output = run_example("c_extensions/c_modules.rb");
    assert_eq!(output, C_MODULES_OUTPUT);
}

#[test]
fn test_c_extensions_c_modules_no_parens_execution() {
    let output = run_example("c_extensions/c_modules_no_parens.rb");
    assert_eq!(output, C_MODULES_OUTPUT);
}

/// The expected output of both `c_extensions/c_classes` variants.
const C_CLASSES_OUTPUT: &str = concat!(
    "[\"hello\", \"hello hello \", true]\n",
    "NoMethodError: super: no superclass method 'nothing_above' for an instance of LoudGreeting\n",
    "[\"Outer::Inner\", \"File::Stat\"]\n",
    "[Outer::Inner, Outer]\n",
    "ArgumentError: undefined class/module Outer::Missing\n",
    "ArgumentError: undefined class/module Outer::String\n",
    "TypeError: Outer::VALUE does not refer to class/module\n",
    "[[:guarded, :shown], [:shown], [:guarded], [:hidden]]\n",
    "[Greeting, nil, \"hello\"]\n",
    "TypeError: can't make subclass of Class\n",
    "TypeError: can't make subclass of singleton class\n",
    "TypeError: superclass must be an instance of Class (given an instance of Module)\n",
    "[[{first: 1}], {second: 2}]\n",
    "TypeError: no implicit conversion of Integer into Hash\n",
    "[LoudGreeting, LoudGreeting, true]\n",
    "[[Greeting, Greeting], [nil, false]]\n",
    "NoMethodError: undefined method 'superclass' for module Comparable\n",
    "[true, false, true]\n",
    "[1, 2, 3]\n",
    "[:@@by_id, :@@by_name, :@@count, :@@defined]\n",
    "NameError: uninitialized class variable @@missing in Counted\n",
    "[1, 2, 4]\n",
    "NoMethodError: undefined method 'readable=' for an instance of Stored\n",
    ":waving\n",
    "[21, 17, 18, 19, 4, 5, 20, 7, 8, 2, 3, 10, 9, 1]\n",
    "[true, true, false]\n",
    "ArgumentError: no super class for 'Outer::NoParent'\n",
    "TypeError: superclass must be an instance of Class (given an instance of Module)\n",
    "TypeError: superclass mismatch for class Outer::Inner (Greeting is given but was Object)\n",
    "TypeError: Outer::VALUE is not a class (Integer)\n",
    "TypeError: superclass mismatch for class LoudGreeting\n",
    "[\"Outer::_hidden\", true]\n",
);

#[test]
fn test_c_extensions_c_classes_execution() {
    let output = run_example("c_extensions/c_classes.rb");
    assert_eq!(output, C_CLASSES_OUTPUT);
}

#[test]
fn test_c_extensions_c_classes_no_parens_execution() {
    let output = run_example("c_extensions/c_classes_no_parens.rb");
    assert_eq!(output, C_CLASSES_OUTPUT);
}

/// The expected output of both `c_extensions/c_data` variants.
const C_DATA_OUTPUT: &str = concat!(
    "[5, \"first\", 6]\n",
    "[5, 5]\n",
    "true\n",
    "[1024, [true, \"counter\", true, true]]\n",
    "[true, \"counter\", true, true]\n",
    "7\n",
    "TypeError: wrong argument type counter (expected other)\n",
    "TypeError: wrong argument type String (expected base)\n",
    "TypeError: wrong argument type nil (expected base)\n",
    "[42, 9, [false, nil, false, true]]\n",
    "TypeError: wrong argument type Object (expected base)\n",
    "[true, true]\n",
    "TypeError: wrong argument type Object (expected Data)\n",
    "TypeError: wrong argument type Integer (expected String)\n",
    "TypeError: wrong argument type nil (expected Data)\n",
    "TypeError: wrong argument type Object (expected Data)\n",
);

#[test]
fn test_c_extensions_c_data_execution() {
    let output = run_example("c_extensions/c_data.rb");
    assert_eq!(output, C_DATA_OUTPUT);
}

#[test]
fn test_c_extensions_c_data_no_parens_execution() {
    let output = run_example("c_extensions/c_data_no_parens.rb");
    assert_eq!(output, C_DATA_OUTPUT);
}

/// The expected output of both `c_extensions/c_threads` variants.
const C_THREADS_OUTPUT: &str = concat!(
    "[true, true]\n",
    "[1, 2, nil]\n",
    "[Thread, 42]\n",
    "RuntimeError: failed in the thread\n",
    "true\n",
    "[true, :woken]\n",
    "ThreadError: killed thread\n",
    "[[true, true], false]\n",
    "42\n",
    "\"sleep\"\n",
    "true\n",
    "true\n",
    "true\n",
    "true\n",
);

#[test]
fn test_c_extensions_c_threads_execution() {
    let output = run_example("c_extensions/c_threads.rb");
    assert_eq!(output, C_THREADS_OUTPUT);
}

#[test]
fn test_c_extensions_c_threads_no_parens_execution() {
    let output = run_example("c_extensions/c_threads_no_parens.rb");
    assert_eq!(output, C_THREADS_OUTPUT);
}

/// The expected output of both `c_extensions/c_hashes` variants.
const C_HASHES_OUTPUT: &str = concat!(
    "[5, true, 12]\n",
    "TypeError: no implicit conversion from nil to integer\n",
    "[{}, {}, {\"converted\" => true}]\n",
    "TypeError: can't convert Integer into Hash\n",
    "[{}, {}, true, false]\n",
    "RuntimeError: st_table too big\n",
    "true\n",
    "[[1, 1, 1], [0, nil, :missing]]\n",
    "[true, false]\n",
    "[1, {first: 1}]\n",
    "[1, nil, {}]\n",
    "[{one: 1, three: 3}, Enumerator]\n",
    ":yes\n",
    "KeyError: key not found: :absent\n",
    "[true, {}, 2]\n",
    "10\n",
    "{b: 2, c: 4, a: 3}\n",
    "[{name: \"Ada\", year: 1815}, {name: \"Ada\"}]\n",
    "[{name: \"Ada\", year: 1815}, {}]\n",
    "[true, true]\n",
);

#[test]
fn test_c_extensions_c_hashes_execution() {
    let output = run_example("c_extensions/c_hashes.rb");
    assert_eq!(output, C_HASHES_OUTPUT);
}

#[test]
fn test_c_extensions_c_hashes_no_parens_execution() {
    let output = run_example("c_extensions/c_hashes_no_parens.rb");
    assert_eq!(output, C_HASHES_OUTPUT);
}
