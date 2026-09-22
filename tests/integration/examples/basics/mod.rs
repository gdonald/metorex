// The example scripts this topic covers, each run and compared against
// the output it is expected to write.

mod collections;
mod control_flow;
mod numbers;
mod output;
mod text;

/// The expected output of both `basics/string_bytes` variants, which differ
/// only in whether the calls are written with parentheses.
pub(super) const STRING_BYTES_OUTPUT: &str = "5\n6\n[104, 195, 169, 108, 108, 111]\n104\n195\n111\nnil\n\"h\"\nfalse\ntrue\ntrue\n[97, 98, 99]\n[97, 98, 99]\n10\n31\n2880289470\n725008\n11259375\n0\n-4660\n511\n10\n15\n99\n255\n-511\n0\n";

/// The expected output of both `basics/logical_keyword_precedence` variants.
pub(super) const LOGICAL_KEYWORD_PRECEDENCE_OUTPUT: &str =
    "[1, 2]\n[false, 7]\n4\n[92, 110, 92, 116]\n\"set\"\n\"set\"\nnil\nnil\n5\n";

/// The expected output of both `basics/hash_and_array_constructors` variants,
/// which differ only in whether the calls are written with parentheses.
pub(super) const HASH_AND_ARRAY_CONSTRUCTORS_OUTPUT: &str = concat!(
    "{a: :b, c: :d}\n",
    "{a: nil}\n",
    "\"wrong element type Symbol at 0 (expected array)\"\n",
    "\"invalid number of elements (3 for 1..2)\"\n",
    "{a: :b}\n",
    "[4, 5, 6]\n",
    "[1, 2]\n",
    "TypeError\n",
);

/// The expected output of both `basics/integer_bits` variants, which differ
/// only in whether the calls are written with parentheses.
pub(super) const INTEGER_BITS_OUTPUT: &str = concat!(
    "1\n0\n1\n0\n1\n3\n166\n8\n3\n41\n0\n",
    "The beginless range for Integer#[] results in infinity\n",
    "Infinity\n"
);

/// The expected output of both `basics/whole_number_limits` variants, which
/// differ only in whether the calls are written with parentheses.
pub(super) const WHOLE_NUMBER_LIMITS_OUTPUT: &str = "48\n0\n0\n0\n71\n\"exponent is too large\"\n1\n1\n\"divided by 0\"\nInfinity\n(4/1)\n1.4142135623730951\n10\n24\n\"Integer#pow() 2nd argument not allowed unless all arguments are integers\"\n";

/// The expected output of both `basics/exact_powers` variants, which differ only in
/// whether the calls are written with parentheses.
pub(super) const EXACT_POWERS_OUTPUT: &str = "(81/256)\n(256/81)\n(1/1)\n(9/16)\n(4/3)\n0.681420222312\n[-0.733761610865, 1.270912390663]\n27.0\n(1/1)\n(1/1)\n\"divided by 0\"\n\"exponent is too large\"\n";

/// The expected output of both `basics/heredoc_details` variants, which
/// differ only in whether the calls are written with parentheses.
pub(super) const HEREDOC_DETAILS_OUTPUT: &str = concat!(
    "hello world\n",
    "US-ASCII\n",
    "\"a\\nbc\\n\"\n",
    "SyntaxError\n",
    "30\n",
);
