use super::run_example;

/// The expected output of both `hash_methods/lookup_and_transform` variants,
/// which differ only in whether the calls are written with parentheses.
const LOOKUP_AND_TRANSFORM_OUTPUT: &str = "0\n0\n5\n5\nfalse\ntrue\n[:alice, :bob, :carol]\n[10, nil, 30]\n3\ntrue\nfalse\n:carol\n{10 => :alice, nil => :bob, 30 => :carol}\n{alice: 10, carol: 30}\n{alice: 10, carol: 30}\n{alice: 10, carol: 30}\n[10, nil]\n[10, 30]\n[:alice, 10]\n[:carol, 30]\n[:alice, 10, :bob, nil, :carol, 30]\n[[:alice, 10], [:bob, nil], [:carol, 30]]\ntrue\ntrue\n[[:alice, 10], [:bob, nil], [:carol, 30]]\n[:alice, :bob, :carol]\n[10, nil, 30]\n{alice: 10, carol: 30}\n{bob: nil}\n{alice: \"10\", bob: \"\", carol: \"30\"}\n{\"alice\" => 10, \"bob\" => nil, \"carol\" => 30}\n{b: 2}\n{b: 2, c: 3}\n[:b, 2]\n{c: 3}\n{}\ntrue\ntrue\ntrue\nfalse\n{a: 1, b: 2}\n{1 => 2}\n{k: :v}\n{x: 1, y: 2}\n\"int\"\n\"float\"\n[1, :spliced, 4, 5]\n[:first, :spliced, 4, 5]\n[:first, :spliced, 4, 5, nil, nil, nil, nil, :far]\n[:a, :b, :c, 4, 5, nil, nil, nil, nil, :far]\n:zero\n";

#[test]
fn test_hash_lookup_and_transform_execution() {
    let output = run_example("hash_methods/lookup_and_transform.rb");
    assert_eq!(output, LOOKUP_AND_TRANSFORM_OUTPUT);
}

#[test]
fn test_hash_lookup_and_transform_no_parens_execution() {
    let output = run_example("hash_methods/lookup_and_transform_no_parens.rb");
    assert_eq!(output, LOOKUP_AND_TRANSFORM_OUTPUT);
}

/// The expected output of both `hash_methods/keys_and_merging` variants,
/// which differ only in whether the calls are written with parentheses.
const KEYS_AND_MERGING_OUTPUT: &str = "true\nfalse\n1\ntrue\n1\n\"Key\"\n:fallback\n:from_block\n{a: 1, b: 20, c: 3}\n{a: 1, b: 20, d: 4}\n{a: 1, b: 22}\n{a: 1, b: 2}\n{a: 1, b: 2, c: 3}\n{a: 1, b: 2, c: 3, d: 4}\n{z: 26}\n{b: 2}\n{b: 2}\nnil\n{a: 1}\nnil\ntrue\ntrue\nfalse\n9\n9\n\"name\"\n\"other\"\n\"Proc\"\n[[:x, 1], [:y, 2]]\n2\n{\"x\" => 2}\n[:nil, :false]\n{first: 1, second: 2}\n\"Counts\"\n1\n\"Counts\"\n3\n";

#[test]
fn test_hash_keys_and_merging_execution() {
    let output = run_example("hash_methods/keys_and_merging.rb");
    assert_eq!(output, KEYS_AND_MERGING_OUTPUT);
}

#[test]
fn test_hash_keys_and_merging_no_parens_execution() {
    let output = run_example("hash_methods/keys_and_merging_no_parens.rb");
    assert_eq!(output, KEYS_AND_MERGING_OUTPUT);
}
