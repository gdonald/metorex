// Coverage tests for ObjectSpace::WeakMap, ObjectSpace::WeakKeyMap, what the
// object space reports about a size, and a Singleton's own builders.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

fn text(value: &str) -> Option<Object> {
    Some(Object::string(value))
}

fn yes() -> Option<Object> {
    Some(Object::Bool(true))
}

fn no() -> Option<Object> {
    Some(Object::Bool(false))
}

/// A map holding two entries under keys the test can reach.
const TWO_ENTRIES: &str = "map = ObjectSpace::WeakMap.new\nfirst = Object.new\nsecond = Object.new\nmap[first] = \"one\"\nmap[second] = \"two\"\n";

// ── The map keyed by identity ──────────────────────────────────────────────

#[test]
fn a_weak_map_answers_what_it_holds() {
    assert_eq!(run("ObjectSpace::WeakMap.include?(Enumerable)"), yes());
    assert_eq!(run("ObjectSpace::WeakMap.new.size"), Some(Object::Int(0)));
    assert_eq!(run(&format!("{TWO_ENTRIES}map.size")), Some(Object::Int(2)));
    assert_eq!(
        run(&format!("{TWO_ENTRIES}map.length")),
        Some(Object::Int(2))
    );
    assert_eq!(run(&format!("{TWO_ENTRIES}map[first]")), text("one"));
    assert_eq!(run(&format!("{TWO_ENTRIES}map.key?(second)")), yes());
    assert_eq!(run(&format!("{TWO_ENTRIES}map.member?(second)")), yes());
    assert_eq!(run(&format!("{TWO_ENTRIES}map.key?(Object.new)")), no());
    assert_eq!(
        run(&format!("{TWO_ENTRIES}map.values.sort.inspect")),
        text("[\"one\", \"two\"]")
    );
    assert_eq!(
        run(&format!("{TWO_ENTRIES}map.keys.size")),
        Some(Object::Int(2))
    );
    // `key` looks its value up by identity, so a fresh literal holding the
    // same text is a different value and finds nothing.
    assert_eq!(
        run(&format!("{TWO_ENTRIES}map.key(\"two\")")),
        Some(Object::Nil)
    );
    assert_eq!(
        run(&format!(
            "{TWO_ENTRIES}held = \"three\"\nmap[first] = held\nmap.key(held).equal?(first)"
        )),
        yes()
    );
    assert_eq!(
        run(&format!("{TWO_ENTRIES}map.key(\"none\")")),
        Some(Object::Nil)
    );
}

#[test]
fn setting_the_same_key_twice_leaves_one_entry() {
    assert_eq!(
        run(
            "map = ObjectSpace::WeakMap.new\nkey = Object.new\nmap[key] = 1\nmap[key] = 2\n[map.size, map[key]].inspect"
        ),
        text("[1, 2]")
    );
    assert_eq!(
        run(
            "map = ObjectSpace::WeakMap.new\nkey = Object.new\nmap[key] = nil\n[map.size, map.key?(key)].inspect"
        ),
        text("[1, true]")
    );
}

#[test]
fn a_weak_map_walks_its_entries_only_with_a_block() {
    assert_eq!(
        run(&format!(
            "{TWO_ENTRIES}seen = []\nmap.each {{ |key, value| seen.push(value) }}\nseen.sort.inspect"
        )),
        text("[\"one\", \"two\"]")
    );
    assert_eq!(
        run(&format!(
            "{TWO_ENTRIES}seen = []\nmap.each_value {{ |value| seen.push(value) }}\nseen.sort.inspect"
        )),
        text("[\"one\", \"two\"]")
    );
    assert_eq!(
        run(&format!(
            "{TWO_ENTRIES}counted = 0\nmap.each_key {{ |key| counted += 1 }}\ncounted"
        )),
        Some(Object::Int(2))
    );
    assert_eq!(
        run("ObjectSpace::WeakMap.new.each.class.name"),
        text("ObjectSpace::WeakMap")
    );
    assert_eq!(
        run(&format!(
            "{TWO_ENTRIES}begin\n  map.each\nrescue LocalJumpError => error\n  error.class.name\nend"
        )),
        text("LocalJumpError")
    );
}

#[test]
fn deleting_from_a_weak_map_answers_what_was_there() {
    assert_eq!(run(&format!("{TWO_ENTRIES}map.delete(first)")), text("one"));
    assert_eq!(
        run(&format!("{TWO_ENTRIES}map.delete(first)\nmap.key?(first)")),
        no()
    );
    assert_eq!(
        run("ObjectSpace::WeakMap.new.delete(Object.new)"),
        Some(Object::Nil)
    );
    assert_eq!(
        run("ObjectSpace::WeakMap.new.delete(Object.new) { |key| 5 }"),
        Some(Object::Int(5))
    );
}

// ── The map keyed by value ─────────────────────────────────────────────────

#[test]
fn a_weak_key_map_compares_its_keys_by_value() {
    assert_eq!(
        run("map = ObjectSpace::WeakKeyMap.new\nmap[\"a\".upcase] = 1\nmap[\"a\".upcase]"),
        Some(Object::Int(1))
    );
    assert_eq!(
        run("map = ObjectSpace::WeakKeyMap.new\nmap[\"a\".upcase] = 1\nmap.key?(\"A\")"),
        yes()
    );
    assert_eq!(
        run("map = ObjectSpace::WeakKeyMap.new\nmap[\"a\".upcase] = 1\nmap.getkey(\"A\")"),
        text("A")
    );
    assert_eq!(
        run("map = ObjectSpace::WeakKeyMap.new\nmap[[1.0]] = \"x\"\nmap[[1]]"),
        Some(Object::Nil)
    );
    assert_eq!(
        run(
            "map = ObjectSpace::WeakKeyMap.new\nmap[\"a\"] = 1\nmap[\"b\"] = 2\nmap.delete(\"a\")\nmap.size"
        ),
        Some(Object::Int(1))
    );
    assert_eq!(
        run("map = ObjectSpace::WeakKeyMap.new\nmap[\"a\"] = 1\nmap.clear.size"),
        Some(Object::Int(0))
    );
}

#[test]
fn a_weak_key_map_refuses_a_key_that_lives_for_the_whole_run() {
    for key in ["42", "1.0", ":name", "true", "false", "nil"] {
        assert_eq!(
            run(&format!(
                "map = ObjectSpace::WeakKeyMap.new\nbegin\n  map[{key}] = 1\nrescue ArgumentError => error\n  error.message\nend"
            )),
            text("WeakKeyMap must be garbage collectable")
        );
        assert_eq!(
            run(&format!("ObjectSpace::WeakKeyMap.new[{key}]")),
            Some(Object::Nil)
        );
        assert_eq!(
            run(&format!("ObjectSpace::WeakKeyMap.new.key?({key})")),
            no()
        );
        assert_eq!(
            run(&format!("ObjectSpace::WeakKeyMap.new.getkey({key})")),
            Some(Object::Nil)
        );
        assert_eq!(
            run(&format!("ObjectSpace::WeakKeyMap.new.delete({key})")),
            Some(Object::Nil)
        );
    }
}

#[test]
fn a_weak_key_map_writes_its_size_when_it_is_inspected() {
    assert_eq!(
        run("ObjectSpace::WeakKeyMap.new.inspect.start_with?(\"#<ObjectSpace::WeakKeyMap:0x\")"),
        yes()
    );
    assert_eq!(
        run("ObjectSpace::WeakKeyMap.new.inspect.end_with?(\" size=0>\")"),
        yes()
    );
    assert_eq!(
        run("map = ObjectSpace::WeakKeyMap.new\nmap[\"a\"] = 1\nmap.inspect.include?(\"size=1\")"),
        yes()
    );
}

// ── What the object space says about a size ────────────────────────────────

#[test]
fn the_object_space_reports_the_size_of_an_object() {
    let with_objspace = |body: &str| run(&format!("require 'objspace'\n{body}"));
    assert_eq!(
        with_objspace("ObjectSpace.memsize_of(nil)"),
        Some(Object::Int(0))
    );
    assert_eq!(
        with_objspace("ObjectSpace.memsize_of(true)"),
        Some(Object::Int(0))
    );
    assert_eq!(
        with_objspace("ObjectSpace.memsize_of(42)"),
        Some(Object::Int(0))
    );
    assert_eq!(
        with_objspace("ObjectSpace.memsize_of(:name)"),
        Some(Object::Int(0))
    );
    assert_eq!(
        with_objspace("ObjectSpace.memsize_of(Object.new) > 0"),
        yes()
    );
    assert_eq!(
        with_objspace(
            "held = Object.new\nbefore = ObjectSpace.memsize_of(held)\nheld.instance_variable_set(:@one, 1)\nObjectSpace.memsize_of(held) > before"
        ),
        yes()
    );
    // A name the object space keeps no account of still answers nil.
    assert_eq!(run("ObjectSpace.count_objects"), Some(Object::Nil));
}

// ── A singleton is reached through instance alone ──────────────────────────

#[test]
fn a_singleton_is_not_built_by_new_or_allocate() {
    let singleton = "require 'singleton'\nclass Held\n  include Singleton\nend\n";
    assert_eq!(
        run(&format!("{singleton}Held.instance.equal?(Held.instance)")),
        yes()
    );
    assert_eq!(
        run(&format!(
            "{singleton}begin\n  Held.new\nrescue NoMethodError => error\n  error.class.name\nend"
        )),
        text("NoMethodError")
    );
    assert_eq!(
        run(&format!(
            "{singleton}begin\n  Held.allocate\nrescue NoMethodError => error\n  error.class.name\nend"
        )),
        text("NoMethodError")
    );
}
