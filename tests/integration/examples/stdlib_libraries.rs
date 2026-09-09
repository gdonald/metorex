// Examples covering the libraries metorex carries

use super::run_example;

/// The expected output of both `stdlib_libraries/shipped_libraries` variants,
/// which differ only in whether the calls are written with parentheses.
const SHIPPED_LIBRARIES_OUTPUT: &str = "\"Tm93IGlzIHRoZSB0aW1lIGZvciBhbGwgZ29vZCBjb2RlcnMKdG8gbGVhcm4g\\nUnVieQ==\\n\"\n\"aGVsbG8=\"\n\"hello\"\n\"hello\"\n\"aGVsbG8_\"\n\"aGVsbG8_\"\n\"hello?\"\n[\"ruby\", \"-e\", \"puts 1\", \"--name\", \"value\"]\n\"a\\\\ b\"\n\"ruby a\\\\ b\"\n[\"one\", \"two three\"]\n{\"r\" => \"ruby\", \"ru\" => \"ruby\", \"rub\" => \"ruby\", \"ruby\" => \"ruby\"}\n{\"ca\" => \"car\", \"car\" => \"car\", \"co\" => \"cone\", \"con\" => \"cone\", \"cone\" => \"cone\"}\ntrue\n0\nNoMethodError\nTypeError\n0\n1\n:moved\n0\n32\n8\n\"\"\n36\nInteger\n7\n7\n\"e\"\n1\n\"xam\"\n4\nfalse\nnil\nnil\nfalse\ntrue\ntrue\nfalse\ntrue\n\"first\\n\"\n1\n[\"second\\n\"]\n0\n[\"first\\n\", \"second\\n\"]\n[\"a\", \"b\", \"c\"]\n[97, 98, 99]\n\"a\"\n97\n\"one two!\\nthree\"\n14\n13\n0\n2\n0\n\"exam\"\nIOError\n\"Ada\"\n1843\n\"computing\"\n\"Ada\"\n\"London\"\n{name: \"Ada\", year: 1843, field: \"computing\", city: \"London\"}\n\"#<OpenStruct name=\\\"Ada\\\", year=1843, field=\\\"computing\\\", city=\\\"London\\\">\"\ntrue\nfalse\nnil\ntrue\n\"#<OpenStruct>\"\ntrue\nfalse\n[2, 3, 5, 7, 11]\n[[2, 3], [3, 2], [5, 1]]\n72\n[[-1, 1], [2, 1], [5, 1]]\ntrue\n[[2, 2], [3, 1]]\n20\n2\n3\n2\n";

#[test]
fn test_stdlib_libraries_shipped_libraries_execution() {
    let output = run_example("stdlib_libraries/shipped_libraries.rb");
    assert_eq!(output, SHIPPED_LIBRARIES_OUTPUT);
}

#[test]
fn test_stdlib_libraries_shipped_libraries_no_parens_execution() {
    let output = run_example("stdlib_libraries/shipped_libraries_no_parens.rb");
    assert_eq!(output, SHIPPED_LIBRARIES_OUTPUT);
}

/// The expected output of both `stdlib_libraries/reading_the_user_database`
/// variants.
const READING_THE_USER_DATABASE_OUTPUT: &str = "Etc::Passwd\nString\ntrue\ntrue\nString\nString\ntrue\nEtc::Group\nString\ntrue\nArray\nHash\n[:sysname, :nodename, :release, :version, :machine]\ntrue\ntrue\nString\n\"/etc\"\nString\ntrue\nRuntimeError\n\"no implicit conversion of String into Integer\"\n";

#[test]
fn test_stdlib_libraries_reading_the_user_database_execution() {
    let output = run_example("stdlib_libraries/reading_the_user_database.rb");
    assert_eq!(output, READING_THE_USER_DATABASE_OUTPUT);
}

#[test]
fn test_stdlib_libraries_reading_the_user_database_no_parens_execution() {
    let output = run_example("stdlib_libraries/reading_the_user_database_no_parens.rb");
    assert_eq!(output, READING_THE_USER_DATABASE_OUTPUT);
}

#[test]
fn test_stdlib_libraries_message_digests_execution() {
    let expected = concat!(
        "\"900150983cd24fb0d6963f7d28e17f72\"\n",
        "\"a9993e364706816aba3e25717850c26c9cd0d89d\"\n",
        "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\"\n",
        "\"cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7\"\n",
        "\"ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f\"\n",
        "\"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"\n",
        "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\"\n",
        "32\n",
        "64\n",
        "true\n",
        "true\n",
        "true\n",
        "\"#<Digest::MD5: d41d8cd98f00b204e9800998ecf8427e>\"\n",
        "true\n",
        "\"47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=\"\n",
        "\"73616d706c6520737472696e67\"\n",
        "\"xexax\"\n",
        "\"xinik-zorox\"\n",
        "\"xesef-disof-gytuf-katof-movif-baxux\"\n",
    );
    let output = run_example("stdlib_libraries/message_digests.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_message_digests_parens_execution() {
    let expected = concat!(
        "\"900150983cd24fb0d6963f7d28e17f72\"\n",
        "\"a9993e364706816aba3e25717850c26c9cd0d89d\"\n",
        "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\"\n",
        "\"cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7\"\n",
        "\"ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f\"\n",
        "\"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"\n",
        "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\"\n",
        "32\n",
        "64\n",
        "true\n",
        "true\n",
        "true\n",
        "\"#<Digest::MD5: d41d8cd98f00b204e9800998ecf8427e>\"\n",
        "true\n",
        "\"47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=\"\n",
        "\"73616d706c6520737472696e67\"\n",
        "\"xexax\"\n",
        "\"xinik-zorox\"\n",
        "\"xesef-disof-gytuf-katof-movif-baxux\"\n",
    );
    let output = run_example("stdlib_libraries/message_digests_parens.rb");
    assert_eq!(output, expected);
}
