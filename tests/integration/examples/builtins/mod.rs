// The example scripts this topic covers, each run and compared against
// the output it is expected to write.

mod encodings;
mod formatting;
mod kernel;
mod numbers;
mod packing;
mod randomness;
mod streams;
mod walks;

/// The expected output of both `builtins/encoding_converter/paths` variants.
pub(super) const ENCODING_CONVERTER_OUTPUT: &str = concat!(
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:UTF-8>\n",
    "\"#<Encoding::Converter: US-ASCII to UTF-8>\"\n",
    "[[#<Encoding:US-ASCII>, #<Encoding:UTF-8>]]\n",
    "\"�\"\n",
    "[[#<Encoding:US-ASCII>, #<Encoding:UTF-8>], [#<Encoding:UTF-8>, #<Encoding:Big5>]]\n",
    "\"crlf_newline\"\n",
    "#<Encoding:UTF-8>\n",
    "nil\n",
    "true\n",
    "Encoding::ConverterNotFoundError\n",
    "\"fubar\"\n",
    "\"spelled out\"\n"
);

/// The expected output of both `builtins/environment_lookup/coercion` variants.
pub(super) const ENVIRONMENT_LOOKUP_OUTPUT: &str = concat!(
    "true\n",
    "true\n",
    "true\n",
    "true\n",
    "true\n",
    "true\n",
    "\"metorex_example\"\n",
    "[\"metorex_example\", \"held\"]\n",
    "[\"metorex_example\", \"held\"]\n",
    "nil\n",
    "nil\n",
    "\"no implicit conversion of Object into String\"\n",
    "nil\n"
);

/// The expected output of both `builtins/byte_views/reading` variants.
pub(super) const BYTE_VIEWS_OUTPUT: &str = concat!(
    "1\n",
    "3\n",
    "11\n",
    "#<Encoding:UTF-8>\n",
    "#<Encoding:BINARY (ASCII-8BIT)>\n",
    "#<Encoding:EUC-JP>\n",
    "#<Encoding:BINARY (ASCII-8BIT)>\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:US-ASCII>\n",
    "255\n",
    "\"can't convert Whole into Rational (Whole#to_r gives Integer)\"\n"
);

/// The expected output of both `builtins/encoding_converter/converting`
/// variants, which differ only in whether the calls are written with
/// parentheses.
pub(super) const ENCODING_CONVERTING_OUTPUT: &str = "\"plain\"\n\"UTF-8\"\n\"US-ASCII\"\n\"\u{8765}\"\n\"UTF-8\"\n\"ISO-8859-1\"\n[241]\n[97]\n\"UTF-8\"\nEncoding\ntrue\n";

/// The expected output of both `builtins/numeric_parts` variants.
pub(super) const NUMERIC_PARTS_OUTPUT: &str = "(0+3i)\n(3+0i)\n[5, 0]\n[5, 3.141592653589793]\n[5, 0]\ntrue\n0\n0\n9\n(5/2)\n0.8\n1\n-1\n1\n-100\n(1/3)\n(3/10)\n(4806858197361/1421)\n(4+6i)\n(Infinity+Infinity*i)\n(2.0+6.0i)\n((1/1)+(3/1)*i)\n[2, 1024, -1021]\nnil\nnil\n";

/// The expected output of both `builtins/range_bounds` variants.
pub(super) const RANGE_BOUNDS_OUTPUT: &str =
    "true\ntrue\ntrue\nfalse\nfalse\ntrue\nfalse\ntrue\nfalse\n\"bad value for range\"\n";

/// The expected output of both `builtins/comparing_values` variants, which
/// differ only in whether the calls are written with parentheses.
pub(super) const COMPARING_VALUES_OUTPUT: &str = concat!(
    "true\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\nfalse\n4\n5\n",
    "\"cannot exclude non Integer end value\"\ntrue\nfalse\nfalse\ntrue\n"
);

/// The expected output of both `builtins/optimized_redefinition` variants.
pub(super) const OPTIMIZED_REDEFINITION_OUTPUT: &str = "[performance] Redefining 'Integer#+' disables interpreter and JIT optimizations\nreplaced\n3\nquiet\n85968058271978839505040\n";

/// The expected output of both `builtins/enumerator/walk_shapes` variants.
pub(super) const WALK_SHAPES_OUTPUT: &str = "100\n201\n\"#<Enumerator: uninitialized>\"\n\"#<Enumerator: 1..3:each>\"\n\"#<Enumerator: 1..3:each_slice(2)>\"\n3\ntrue\n:answered\n[3, 2, [:more]]\n[1, 2, 3, 4]\ntrue\n[1, 2, 3, 4]\n";

/// The expected output of both `builtins/complex_powers` variants, which show
/// a complex number raised to a power that is not a whole number and differ only in whether the calls are
/// written with parentheses.
pub(super) const COMPLEX_POWERS_OUTPUT: &str =
    "(3+4i)\n(1.0+0.0i)\n[-38.0, 41.0]\n[1.719133, 0.623125]\n[-0.504825, 3.104144]\n32\n";

/// The expected output of both `builtins/frozen_answers` variants, which show
/// what the runtime hands back frozen, and how a copy carries that state and differ only in whether the calls are
/// written with parentheses.
pub(super) const FROZEN_ANSWERS_OUTPUT: &str = "[true, true, true]\ntrue\ntrue\n[true, true]\nfalse\n[1, 2, 3, 4]\ntrue\n[true, false, false]\n";

/// The expected output of both `builtins/numbers_written_out` variants, which differ only in whether the
/// calls are written with parentheses.
pub(super) const NUMBERS_WRITTEN_OUT_OUTPUT: &str = concat!(
    "1.5\n",
    "1.0e+15\n",
    "0.0001\n",
    "1.0e-05\n",
    "2.554021731435405e+163\n",
    "1.0e+16\n",
    "US-ASCII\n",
    "1024\n",
    "3697379018277258\n",
    "-4\n",
);

/// The expected output of both `builtins/packed_text` variants, which differ
/// only in whether the calls are written with parentheses.
pub(super) const PACKED_TEXT_OUTPUT: &str = "\"Aあ\"\n#<Encoding:UTF-8>\n[244, 144, 128, 128]\n[253, 191, 191, 191, 191, 191]\nRangeError\n\"&86)C9&5F\\n&9VAI:FML\\n!;0``\\n\"\n#<Encoding:US-ASCII>\n#<Encoding:US-ASCII>\n#<Encoding:BINARY (ASCII-8BIT)>\n[12354]\nArgumentError\n\"'!' allowed only after types sSiIlLqQjJ\"\n";

/// The expected output of both `builtins/pointer_packing` variants, which differ only in
/// whether the calls are written with parentheses.
pub(super) const POINTER_PACKING_OUTPUT: &str = "true\n[\"hello\"]\n[\"h\"]\n[\"hello\"]\n[\"hello\"]\n[\"hello\"]\n[0]\n\"no associated pointer\"\n\"a b=\\n\"\n\"\\t=\\n\\n\"\n\"abcd=\\nefgh=\\ni=\\n\"\n\"YWJj\\n\"\n\"YWJj\\nZGVm\\nZw==\\n\"\n\"\"\n";
