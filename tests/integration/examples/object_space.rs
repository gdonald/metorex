use super::run_example;

/// The expected output of both `object_space/finalizers` variants. The
/// report of the finalizer that raised goes to stderr.
const FINALIZERS_OUTPUT: &str = concat!(
    "true\n",
    "true\n",
    "\"cannot define finalizer for Symbol\"\n",
    "\"wrong type argument Object (should be callable)\"\n",
    "first finalized\n",
    "second finalized\n",
    "third finalized\n",
);

#[test]
fn test_object_space_finalizers_execution() {
    let output = run_example("object_space/finalizers.rb");
    assert_eq!(output, FINALIZERS_OUTPUT);
}

#[test]
fn test_object_space_finalizers_no_parens_execution() {
    let output = run_example("object_space/finalizers_no_parens.rb");
    assert_eq!(output, FINALIZERS_OUTPUT);
}

/// The expected output of both `object_space/each_object` variants.
const EACH_OBJECT_OUTPUT: &str = concat!(
    "2\n",
    "[:in_a_hash, :in_an_array]\n",
    "Enumerator\n",
    "[:in_a_hash, :in_an_array]\n",
    "3\n",
    "true\n",
    "\"class or module required\"\n",
    "[1, 1, false]\n",
);

#[test]
fn test_object_space_each_object_execution() {
    let output = run_example("object_space/each_object.rb");
    assert_eq!(output, EACH_OBJECT_OUTPUT);
}

#[test]
fn test_object_space_each_object_no_parens_execution() {
    let output = run_example("object_space/each_object_no_parens.rb");
    assert_eq!(output, EACH_OBJECT_OUTPUT);
}

/// The expected output of both `object_space/gc_latest_info` variants.
const GC_LATEST_INFO_OUTPUT: &str = concat!(
    "[:major_by, :need_major_by, :gc_by, :have_finalizer, :immediate_sweep, :state, :weak_references_count]\n",
    "{major_by: :force, need_major_by: nil, gc_by: :method, have_finalizer: false, immediate_sweep: true, state: :none, weak_references_count: 0}\n",
    "true\n",
    "8\n",
    "[:method, :none]\n",
    "unknown key: unknown\n",
    "non-hash or symbol given\n",
);

#[test]
fn test_object_space_gc_latest_info_execution() {
    let output = run_example("object_space/gc_latest_info.rb");
    assert_eq!(output, GC_LATEST_INFO_OUTPUT);
}

#[test]
fn test_object_space_gc_latest_info_no_parens_execution() {
    let output = run_example("object_space/gc_latest_info_no_parens.rb");
    assert_eq!(output, GC_LATEST_INFO_OUTPUT);
}

/// The expected output of both `object_space/gc_answers_object_methods`
/// variants.
const GC_ANSWERS_OBJECT_METHODS_OUTPUT: &str =
    "\"answered for GC\"\n\"answered for ObjectSpace\"\ntrue\n";

#[test]
fn test_object_space_gc_answers_object_methods_execution() {
    let output = run_example("object_space/gc_answers_object_methods.rb");
    assert_eq!(output, GC_ANSWERS_OBJECT_METHODS_OUTPUT);
}

#[test]
fn test_object_space_gc_answers_object_methods_no_parens_execution() {
    let output = run_example("object_space/gc_answers_object_methods_no_parens.rb");
    assert_eq!(output, GC_ANSWERS_OBJECT_METHODS_OUTPUT);
}
