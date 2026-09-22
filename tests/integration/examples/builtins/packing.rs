// Reading and writing values as runs of bytes.

use super::super::run_example;
use super::*;
#[test]
fn test_builtins_pack_and_trace_execution() {
    let expected = concat!(
        "\"ABC\"\n",
        "[97, 98, 99]\n",
        "[97, 98]\n",
        "[25185]\n",
        "[24930]\n",
        "[25185]\n",
        "[24930]\n",
        "[7523094288207667809]\n",
        "4\n",
        "\"ab  \"\n",
        "[\"abc\", \"def\"]\n",
        "[\"10000000\"]\n",
        "[\"8f\"]\n",
        "2\n",
        "97\n",
        "[97, 98, 99, 99]\n",
        "8\n",
        "4\n",
        "\"\\u0001\"\n",
        "\"\\n\"\n",
        "\"A\"\n",
        "false\n",
        "false\n",
        "1\n",
        ":line\n",
        "[[:call, :traced_method], [:return, :traced_method]]\n",
        "4\n",
        "8\n",
        "Enumerator\n",
        "\"comparison of Integer with \\\"A\\\" failed\"\n",
        "6\n",
    );
    let output = run_example("builtins/pack_and_trace.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_pack_and_trace_parens_execution() {
    let expected = concat!(
        "\"ABC\"\n",
        "[97, 98, 99]\n",
        "[97, 98]\n",
        "[25185]\n",
        "[24930]\n",
        "[25185]\n",
        "[24930]\n",
        "[7523094288207667809]\n",
        "4\n",
        "\"ab  \"\n",
        "[\"abc\", \"def\"]\n",
        "[\"10000000\"]\n",
        "[\"8f\"]\n",
        "2\n",
        "97\n",
        "[97, 98, 99, 99]\n",
        "8\n",
        "4\n",
        "\"\\u0001\"\n",
        "\"\\n\"\n",
        "\"A\"\n",
        "false\n",
        "false\n",
        "1\n",
        ":line\n",
        "[[:call, :traced_method], [:return, :traced_method]]\n",
        "4\n",
        "8\n",
        "Enumerator\n",
        "\"comparison of Integer with \\\"A\\\" failed\"\n",
        "6\n",
    );
    let output = run_example("builtins/pack_and_trace_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_pack_directives_execution() {
    let expected = concat!(
        "4\n",
        "\"ab  \"\n",
        "4\n",
        "[\"ab\", \"cd\"]\n",
        "[\"ab\"]\n",
        "[\"ab\", \"d\"]\n",
        "[160]\n",
        "[5]\n",
        "[143]\n",
        "[143]\n",
        "[\"1010\"]\n",
        "[\"0000\"]\n",
        "[\"8f\"]\n",
        "[\"f8\"]\n",
        "[1, 2]\n",
        "[255]\n",
        "[1, 2]\n",
        "[2, 1]\n",
        "[0, 0, 1, 2]\n",
        "[2, 1, 0, 0]\n",
        "[0, 1]\n",
        "4\n",
        "8\n",
        "8\n",
        "[-1]\n",
        "[255]\n",
        "[258]\n",
        "[16909060]\n",
        "[72623859790382856]\n",
        "4\n",
        "8\n",
        "4\n",
        "8\n",
        "4\n",
        "8\n",
        "[1.5]\n",
        "[1.5]\n",
        "1\n",
        "[233]\n",
        "[130, 44]\n",
        "[300]\n",
        "3\n",
        "[2]\n",
        "4\n",
        "[97, 101]\n",
        "[99]\n",
        "[97, 98, 99]\n",
        "6\n",
        "[97, 98]\n",
        "[97, 98]\n",
        "[ArgumentError, \"unknown pack directive 'K' in 'K'\"]\n",
        "[ArgumentError, \"unknown unpack directive 'K' in 'K'\"]\n",
        "[ArgumentError, \"'!' allowed only after types sSiIlLqQjJ\"]\n",
        "[ArgumentError, \"too few arguments\"]\n",
        "[TypeError, \"no implicit conversion of String into Integer\"]\n",
        "[ArgumentError, \"x outside of string\"]\n",
        "[ArgumentError, \"X outside of string\"]\n",
        "[ArgumentError, \"@ outside of string\"]\n",
        "\"QUJD\\n\"\n",
        "[\"ABC\"]\n",
        "\"ab\\ncd=\\n\"\n",
        "\"#04)#\\n\"\n",
        "[\"ABC\"]\n",
        "1\n",
        "[1, 2]\n",
        "[\"ABC\"]\n",
        "[\"ABC\"]\n",
        "[\"A\"]\n",
        "[\"ab=cd\"]\n",
        "[\"abcd\"]\n",
        "[\"ab=ZZ\"]\n",
        "[\"Cac\"]\n",
        "[\"\"]\n",
        "1\n",
        "[960]\n",
        "[97, 98, 99]\n",
        "\"'_' allowed only after types sSiIlLqQjJ\"\n",
        "\"pack length too big\"\n",
        "[97, 98, 99]\n",
        "\"A\"\n",
    );
    let output = run_example("builtins/pack_directives.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_pack_directives_parens_execution() {
    let expected = concat!(
        "4\n",
        "\"ab  \"\n",
        "4\n",
        "[\"ab\", \"cd\"]\n",
        "[\"ab\"]\n",
        "[\"ab\", \"d\"]\n",
        "[160]\n",
        "[5]\n",
        "[143]\n",
        "[143]\n",
        "[\"1010\"]\n",
        "[\"0000\"]\n",
        "[\"8f\"]\n",
        "[\"f8\"]\n",
        "[1, 2]\n",
        "[255]\n",
        "[1, 2]\n",
        "[2, 1]\n",
        "[0, 0, 1, 2]\n",
        "[2, 1, 0, 0]\n",
        "[0, 1]\n",
        "4\n",
        "8\n",
        "8\n",
        "[-1]\n",
        "[255]\n",
        "[258]\n",
        "[16909060]\n",
        "[72623859790382856]\n",
        "4\n",
        "8\n",
        "4\n",
        "8\n",
        "4\n",
        "8\n",
        "[1.5]\n",
        "[1.5]\n",
        "1\n",
        "[233]\n",
        "[130, 44]\n",
        "[300]\n",
        "3\n",
        "[2]\n",
        "4\n",
        "[97, 101]\n",
        "[99]\n",
        "[97, 98, 99]\n",
        "6\n",
        "[97, 98]\n",
        "[97, 98]\n",
        "[ArgumentError, \"unknown pack directive 'K' in 'K'\"]\n",
        "[ArgumentError, \"unknown unpack directive 'K' in 'K'\"]\n",
        "[ArgumentError, \"'!' allowed only after types sSiIlLqQjJ\"]\n",
        "[ArgumentError, \"too few arguments\"]\n",
        "[TypeError, \"no implicit conversion of String into Integer\"]\n",
        "[ArgumentError, \"x outside of string\"]\n",
        "[ArgumentError, \"X outside of string\"]\n",
        "[ArgumentError, \"@ outside of string\"]\n",
        "\"QUJD\\n\"\n",
        "[\"ABC\"]\n",
        "\"ab\\ncd=\\n\"\n",
        "\"#04)#\\n\"\n",
        "[\"ABC\"]\n",
        "1\n",
        "[1, 2]\n",
        "[\"ABC\"]\n",
        "[\"ABC\"]\n",
        "[\"A\"]\n",
        "[\"ab=cd\"]\n",
        "[\"abcd\"]\n",
        "[\"ab=ZZ\"]\n",
        "[\"Cac\"]\n",
        "[\"\"]\n",
        "1\n",
        "[960]\n",
        "[97, 98, 99]\n",
        "\"'_' allowed only after types sSiIlLqQjJ\"\n",
        "\"pack length too big\"\n",
        "[97, 98, 99]\n",
        "\"A\"\n",
    );
    let output = run_example("builtins/pack_directives_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_packed_text_execution() {
    let output = run_example("builtins/packed_text.rb");
    assert_eq!(output, PACKED_TEXT_OUTPUT);
}

#[test]
fn test_builtins_packed_text_parens_execution() {
    let output = run_example("builtins/packed_text_parens.rb");
    assert_eq!(output, PACKED_TEXT_OUTPUT);
}

#[test]
fn test_builtins_pointer_packing_execution() {
    let output = run_example("builtins/pointer_packing.rb");
    assert_eq!(output, POINTER_PACKING_OUTPUT);
}

#[test]
fn test_builtins_pointer_packing_parens_execution() {
    let output = run_example("builtins/pointer_packing_parens.rb");
    assert_eq!(output, POINTER_PACKING_OUTPUT);
}
