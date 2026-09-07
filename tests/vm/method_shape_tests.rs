// Coverage tests for what a Method, an UnboundMethod, and a Proc report
// about their parameters

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

// ── The parameter list a method reports ────────────────────────────────────

#[test]
fn method_parameters_name_each_kind() {
    assert_eq!(
        run("def m(a, b = 1, *c, d:, e: 2, **f, &g); end\nmethod(:m).parameters.inspect"),
        Some(Object::string(
            "[[:req, :a], [:opt, :b], [:rest, :c], [:keyreq, :d], [:key, :e], [:keyrest, :f], [:block, :g]]"
        ))
    );
}

#[test]
fn method_parameters_leave_an_unnamed_splat_unnamed() {
    assert_eq!(
        run("def m(*); end\nmethod(:m).parameters.inspect"),
        Some(Object::string("[[:rest]]"))
    );
}

#[test]
fn method_parameters_of_a_method_taking_nothing_are_empty() {
    assert_eq!(
        run("def m; end\nmethod(:m).parameters.inspect"),
        Some(Object::string("[]"))
    );
}

// ── The arity a method reports ─────────────────────────────────────────────

#[test]
fn method_arity_counts_the_required_parameters() {
    assert_eq!(
        run("def m(a, b); end\nmethod(:m).arity"),
        Some(Object::Int(2))
    );
    assert_eq!(run("def m; end\nmethod(:m).arity"), Some(Object::Int(0)));
    assert_eq!(
        run("def m(&b); end\nmethod(:m).arity"),
        Some(Object::Int(0))
    );
}

#[test]
fn method_arity_turns_negative_where_a_call_may_pass_more_or_fewer() {
    assert_eq!(
        run("def m(a = 1); end\nmethod(:m).arity"),
        Some(Object::Int(-1))
    );
    assert_eq!(
        run("def m(a, *b); end\nmethod(:m).arity"),
        Some(Object::Int(-2))
    );
    assert_eq!(
        run("def m(a, b = 1, *c, d, e); end\nmethod(:m).arity"),
        Some(Object::Int(-4))
    );
}

#[test]
fn method_arity_counts_a_required_keyword_as_one_more() {
    assert_eq!(
        run("def m(a:); end\nmethod(:m).arity"),
        Some(Object::Int(1))
    );
    assert_eq!(
        run("def m(a:, b:, c: 1); end\nmethod(:m).arity"),
        Some(Object::Int(1))
    );
    assert_eq!(
        run("def m(a, b:); end\nmethod(:m).arity"),
        Some(Object::Int(2))
    );
    assert_eq!(
        run("def m(a, b, c:, d: 1, **k, &l); end\nmethod(:m).arity"),
        Some(Object::Int(3))
    );
}

#[test]
fn method_arity_turns_negative_for_optional_keywords_alone() {
    assert_eq!(
        run("def m(a: 1); end\nmethod(:m).arity"),
        Some(Object::Int(-1))
    );
    assert_eq!(
        run("def m(**k); end\nmethod(:m).arity"),
        Some(Object::Int(-1))
    );
    assert_eq!(
        run("def m(a = 1, *b, c:, d: 2, **k, &l); end\nmethod(:m).arity"),
        Some(Object::Int(-2))
    );
}

#[test]
fn unbound_method_reports_the_same_arity_as_the_bound_one() {
    assert_eq!(
        run("class C\n  def m(a, b = 1); end\nend\nC.instance_method(:m).arity"),
        Some(Object::Int(-2))
    );
}

// ── What a Proc reports ────────────────────────────────────────────────────

#[test]
fn a_lambda_counts_its_parameters_the_way_a_method_does() {
    assert_eq!(run("lambda { |a, b| }.arity"), Some(Object::Int(2)));
    assert_eq!(run("lambda { |a, b = 1| }.arity"), Some(Object::Int(-2)));
    assert_eq!(run("lambda { |a: 1| }.arity"), Some(Object::Int(-1)));
    assert_eq!(run("lambda { |a:| }.arity"), Some(Object::Int(1)));
}

#[test]
fn a_proc_takes_what_it_is_handed_so_only_a_splat_leaves_the_count_open() {
    assert_eq!(run("proc { |a, b = 1| }.arity"), Some(Object::Int(1)));
    assert_eq!(run("proc { |a: 1| }.arity"), Some(Object::Int(0)));
    assert_eq!(run("proc { |**k| }.arity"), Some(Object::Int(0)));
    assert_eq!(run("proc { |a = 1, *b| }.arity"), Some(Object::Int(-1)));
    assert_eq!(run("proc { |a, | }.arity"), Some(Object::Int(1)));
}

#[test]
fn a_proc_reports_its_positional_parameters_as_optional() {
    assert_eq!(
        run("proc { |a, b| }.parameters.inspect"),
        Some(Object::string("[[:opt, :a], [:opt, :b]]"))
    );
    assert_eq!(
        run("lambda { |a, b| }.parameters.inspect"),
        Some(Object::string("[[:req, :a], [:req, :b]]"))
    );
    assert_eq!(
        run("lambda { |a, *b, c:, **d, &e| }.parameters.inspect"),
        Some(Object::string(
            "[[:req, :a], [:rest, :b], [:keyreq, :c], [:keyrest, :d], [:block, :e]]"
        ))
    );
}

// ── How a method renders itself ────────────────────────────────────────────

#[test]
fn a_bound_method_names_its_receiver_class_and_parameters() {
    assert_eq!(
        run(
            "class C\n  def m(a, b = 1); end\nend\nC.new.method(:m).inspect.split(\" \")[0, 2].join(\" \")"
        ),
        Some(Object::string("#<Method: C#m(a,"))
    );
}

#[test]
fn a_method_from_a_module_names_the_module_it_came_from() {
    assert_eq!(
        run(
            "module M\n  def m; end\nend\nclass C\n  include M\nend\nC.new.method(:m).inspect.start_with?(\"#<Method: C(M)#m()\")"
        ),
        Some(Object::Bool(true))
    );
}

#[test]
fn an_unbound_method_says_so_in_its_rendering() {
    assert_eq!(
        run(
            "class C\n  def m(a:); end\nend\nC.instance_method(:m).inspect.start_with?(\"#<UnboundMethod: C#m(a:)\")"
        ),
        Some(Object::Bool(true))
    );
}

// ── The method super would reach ───────────────────────────────────────────

#[test]
fn super_method_answers_the_next_definition_up() {
    assert_eq!(
        run(
            "class Up\n  def m; end\nend\nclass Down < Up\n  def m; end\nend\nDown.new.method(:m).super_method.owner.name"
        ),
        Some(Object::string("Up"))
    );
}

#[test]
fn super_method_answers_nil_when_nothing_is_above() {
    assert_eq!(
        run("class Only\n  def m; end\nend\nOnly.new.method(:m).super_method"),
        Some(Object::Nil)
    );
}

#[test]
fn super_method_answers_nil_when_the_one_above_was_undefined() {
    assert_eq!(
        run(
            "class Up\n  def m; end\nend\nclass Down < Up\n  def m; end\nend\nUp.class_eval { undef :m }\nDown.new.method(:m).super_method"
        ),
        Some(Object::Nil)
    );
}

#[test]
fn super_method_of_an_unbound_method_walks_the_class_it_came_from() {
    assert_eq!(
        run(
            "class Up\n  def m; end\nend\nclass Down < Up\n  def m; end\nend\nDown.instance_method(:m).super_method.owner.name"
        ),
        Some(Object::string("Up"))
    );
}

// ── Binding and calling in one step ────────────────────────────────────────

#[test]
fn bind_call_runs_the_method_against_the_given_receiver() {
    assert_eq!(
        run(
            "class C\n  def m(n)\n    n * 2\n  end\nend\nC.instance_method(:m).bind_call(C.new, 4)"
        ),
        Some(Object::Int(8))
    );
}

#[test]
fn method_names_a_builtin_the_receiver_redefined() {
    assert_eq!(
        run("def p(a, b, c, d); end\nmethod(:p).arity"),
        Some(Object::Int(4))
    );
}
