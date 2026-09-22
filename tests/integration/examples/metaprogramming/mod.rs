// The example scripts this topic covers, each run and compared against
// the output it is expected to write.

mod advanced;
mod blocks;
mod classes;
mod constants;
mod defining;
mod instances;
mod refinements;
mod reflection;
mod singletons;

/// The expected output of both `metaprogramming/instance_exec/singleton_def`
/// variants.
pub(super) const INSTANCE_EXEC_SINGLETON_DEF_OUTPUT: &str = "hello\n[:greeting]\nfalse\n1\n2\n";

/// The expected output of both `metaprogramming/alias_over_native` variants,
/// which differ only in whether the calls are written with parentheses.
pub(super) const ALIAS_OVER_NATIVE_OUTPUT: &str = "-1\n\"replaced\"\n-1\n[1, 2, 3]\ntrue\n";

/// The expected output of both `metaprogramming/inside_a_singleton_class` variants, which differ only in whether the
/// calls are written with parentheses.
pub(super) const INSIDE_A_SINGLETON_CLASS_OUTPUT: &str = concat!(
    "Foo\n",
    "Class\n",
    "true\n",
    "counted\n",
    "#<Class:Parent>\n",
    "[:greeting]\n",
);

/// The expected output of both `metaprogramming/alias_definee` variants,
/// which differ only in whether the calls are written with parentheses.
pub(super) const ALIAS_DEFINEE_OUTPUT: &str =
    "5\n[:second]\nfalse\ncan't define singleton\nObject\n7\n";

/// The expected output of both `metaprogramming/instance_exec/reading_a_string` variants,
/// which differ only in whether the calls are written with parentheses.
pub(super) const READING_A_STRING: &str = concat!(
    ":singleton_class\n:caller\n:assigned\n",
    "\"a_file:10:in '<main>'\"\n",
    "true\n1\n:block_scope\n"
);
