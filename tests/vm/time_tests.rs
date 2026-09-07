// Coverage tests for Time and the environment writes the C library sees

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

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── Building a time ────────────────────────────────────────────────────────

#[test]
fn time_now_answers_a_time_past_the_epoch() {
    assert_eq!(run("Time.now.class.name"), Some(Object::string("Time")));
    assert_eq!(run("Time.now.to_i > 1600000000"), Some(Object::Bool(true)));
}

#[test]
fn time_utc_reads_its_calendar_arguments() {
    assert_eq!(
        run("Time.utc(2007, 11, 1, 15, 25, 30).to_i"),
        Some(Object::Int(1193930730))
    );
    assert_eq!(run("Time.gm(2000, 1, 1).year"), Some(Object::Int(2000)));
}

#[test]
fn time_utc_takes_a_month_by_name() {
    assert_eq!(run("Time.utc(2000, \"feb\", 1).mon"), Some(Object::Int(2)));
    assert_eq!(run("Time.utc(2000, \"3\", 1).mon"), Some(Object::Int(3)));
}

#[test]
fn time_utc_refuses_a_month_it_cannot_read() {
    let err = run_err("Time.utc(2000, \"nope\", 1)");
    assert!(err.contains("mon out of range"), "Error was: {}", err);
}

#[test]
fn time_at_reads_a_count_of_seconds() {
    assert_eq!(run("Time.at(100).to_i"), Some(Object::Int(100)));
    assert_eq!(run("Time.at(100, 100).to_f"), Some(Object::Float(100.0001)));
    assert_eq!(
        run("Time.at(Rational(3, 2)).subsec.inspect"),
        Some(Object::string("(1/2)"))
    );
    assert_eq!(
        run("Time.at(10.75).subsec.inspect"),
        Some(Object::string("(3/4)"))
    );
}

#[test]
fn time_at_carries_over_whether_another_time_was_read_in_utc() {
    assert_eq!(
        run("Time.at(Time.utc(2000, 1, 1)).utc?"),
        Some(Object::Bool(true))
    );
}

#[test]
fn time_new_without_arguments_is_the_current_time() {
    assert_eq!(run("Time.new.to_i > 1600000000"), Some(Object::Bool(true)));
}

#[test]
fn time_new_reads_an_offset_as_its_last_argument() {
    assert_eq!(
        run("Time.new(2000, 1, 1, 0, 0, 0, 3600).utc_offset"),
        Some(Object::Int(3600))
    );
    assert_eq!(
        run("Time.new(2000, 1, 1, 0, 0, 0, \"+02:30\").utc_offset"),
        Some(Object::Int(9000))
    );
    assert_eq!(
        run("Time.new(2000, 1, 1, 0, 0, 0, \"-02:00\").utc_offset"),
        Some(Object::Int(-7200))
    );
    assert_eq!(
        run("Time.new(2000, 1, 1, 0, 0, 0, \"UTC\").utc_offset"),
        Some(Object::Int(0))
    );
}

#[test]
fn time_refuses_an_offset_it_cannot_read() {
    let err = run_err("Time.new(2000, 1, 1, 0, 0, 0, \"noon\")");
    assert!(
        err.contains("expected for utc_offset"),
        "Error was: {}",
        err
    );
}

// ── Reading the calendar fields ────────────────────────────────────────────

#[test]
fn time_answers_each_calendar_field() {
    let moment = "held = Time.utc(2007, 11, 1, 15, 25, 30, 123456)\n";
    assert_eq!(run(&format!("{moment}held.year")), Some(Object::Int(2007)));
    assert_eq!(run(&format!("{moment}held.month")), Some(Object::Int(11)));
    assert_eq!(run(&format!("{moment}held.mday")), Some(Object::Int(1)));
    assert_eq!(run(&format!("{moment}held.hour")), Some(Object::Int(15)));
    assert_eq!(run(&format!("{moment}held.min")), Some(Object::Int(25)));
    assert_eq!(run(&format!("{moment}held.sec")), Some(Object::Int(30)));
    assert_eq!(run(&format!("{moment}held.wday")), Some(Object::Int(4)));
    assert_eq!(run(&format!("{moment}held.yday")), Some(Object::Int(305)));
    assert_eq!(
        run(&format!("{moment}held.usec")),
        Some(Object::Int(123456))
    );
    assert_eq!(
        run(&format!("{moment}held.nsec")),
        Some(Object::Int(123456000))
    );
    assert_eq!(
        run(&format!("{moment}held.thursday?")),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!("{moment}held.zone")),
        Some(Object::string("UTC"))
    );
}

#[test]
fn time_names_the_day_of_the_week() {
    let day = "Time.utc(2000, 1, 1)";
    assert_eq!(run(&format!("{day}.saturday?")), Some(Object::Bool(true)));
    assert_eq!(run(&format!("{day}.sunday?")), Some(Object::Bool(false)));
    assert_eq!(
        run("Time.utc(2000, 1, 2).sunday?"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("Time.utc(2000, 1, 3).monday?"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("Time.utc(2000, 1, 4).tuesday?"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("Time.utc(2000, 1, 5).wednesday?"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("Time.utc(2000, 1, 7).friday?"),
        Some(Object::Bool(true))
    );
}

#[test]
fn time_answers_a_whole_second_subsec_as_an_integer() {
    assert_eq!(run("Time.at(100).subsec"), Some(Object::Int(0)));
}

#[test]
fn time_reads_a_year_the_c_library_would_refuse() {
    assert_eq!(run("Time.utc(12, 4, 12).year"), Some(Object::Int(12)));
    assert_eq!(
        run("Time.utc(12, 4, 12).xmlschema"),
        Some(Object::string("0012-04-12T00:00:00Z"))
    );
    assert_eq!(
        run("Time.utc(-2000, 4, 12).xmlschema"),
        Some(Object::string("-2000-04-12T00:00:00Z"))
    );
    assert_eq!(
        run("Time.utc(40000, 4, 12).xmlschema"),
        Some(Object::string("40000-04-12T00:00:00Z"))
    );
}

// ── Rendering a time ───────────────────────────────────────────────────────

#[test]
fn time_renders_itself_with_its_zone() {
    assert_eq!(
        run("Time.utc(2000, 1, 1, 20, 15, 1).to_s"),
        Some(Object::string("2000-01-01 20:15:01 UTC"))
    );
    assert_eq!(
        run("Time.new(2000, 1, 1, 20, 15, 1, 3600).to_s"),
        Some(Object::string("2000-01-01 20:15:01 +0100"))
    );
}

#[test]
fn time_inspect_shows_the_fraction_without_its_trailing_zeros() {
    assert_eq!(
        run("Time.utc(2007, 11, 1, 15, 25, 0, 123456).inspect"),
        Some(Object::string("2007-11-01 15:25:00.123456 UTC"))
    );
    assert_eq!(
        run("Time.utc(2007, 11, 1, 15, 25, 0, 100000).inspect"),
        Some(Object::string("2007-11-01 15:25:00.1 UTC"))
    );
    assert_eq!(
        run("Time.utc(2007, 11, 1, 15, 25, 0).inspect"),
        Some(Object::string("2007-11-01 15:25:00 UTC"))
    );
}

#[test]
fn time_asctime_writes_the_c_library_shape() {
    assert_eq!(
        run("Time.utc(2000, 1, 1).asctime"),
        Some(Object::string("Sat Jan  1 00:00:00 2000"))
    );
    assert_eq!(
        run("Time.utc(2000, 1, 1).ctime"),
        Some(Object::string("Sat Jan  1 00:00:00 2000"))
    );
}

#[test]
fn time_strftime_fills_in_the_directives_ruby_adds() {
    assert_eq!(
        run("Time.utc(2000, 1, 2, 3, 4, 5).strftime(\"%Y-%m-%d %H:%M:%S\")"),
        Some(Object::string("2000-01-02 03:04:05"))
    );
    assert_eq!(
        run("Time.utc(2000, 1, 1, 0, 0, 0, 123456).strftime(\"%L %N\")"),
        Some(Object::string("123 123456000"))
    );
    assert_eq!(
        run("Time.new(2000, 1, 1, 0, 0, 0, 3600).strftime(\"%z %:z\")"),
        Some(Object::string("+0100 +01:00"))
    );
}

#[test]
fn time_xmlschema_takes_the_number_of_fraction_digits() {
    assert_eq!(
        run("Time.utc(1985, 4, 12, 23, 20, 50, 521245).xmlschema(2)"),
        Some(Object::string("1985-04-12T23:20:50.52Z"))
    );
    assert_eq!(
        run("Time.new(1985, 4, 12, 23, 20, 50, \"+02:00\").iso8601"),
        Some(Object::string("1985-04-12T23:20:50+02:00"))
    );
}

// ── Comparing and shifting a time ──────────────────────────────────────────

#[test]
fn time_arithmetic_answers_a_time_or_a_count_of_seconds() {
    assert_eq!(run("(Time.at(100) + 1).to_i"), Some(Object::Int(101)));
    assert_eq!(run("(Time.at(100) - 100).to_i"), Some(Object::Int(0)));
    assert_eq!(
        run("(Time.at(100) - Time.at(99))"),
        Some(Object::Float(1.0))
    );
    assert_eq!(run("(Time.at(100) - -1.3).usec"), Some(Object::Int(300000)));
}

#[test]
fn time_refuses_to_add_another_time() {
    let err = run_err("Time.at(1) + Time.at(2)");
    assert!(err.contains("time + time?"), "Error was: {}", err);
}

#[test]
fn time_refuses_a_string_however_much_it_looks_like_a_number() {
    let err = run_err("Time.at(1) - \"1\"");
    assert!(err.contains("into an exact number"), "Error was: {}", err);
}

#[test]
fn time_compares_by_the_exact_second_count() {
    assert_eq!(
        run("Time.at(100) == Time.at(100)"),
        Some(Object::Bool(true))
    );
    assert_eq!(run("Time.at(100) < Time.at(200)"), Some(Object::Bool(true)));
    assert_eq!(
        run("Time.at(100) > Time.at(200)"),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run("Time.at(100).eql?(Time.at(100))"),
        Some(Object::Bool(true))
    );
    assert_eq!(run("Time.at(100).eql?(100)"), Some(Object::Bool(false)));
    assert_eq!(
        run("(Time.at(1) <=> 5).inspect"),
        Some(Object::string("nil"))
    );
    assert_eq!(
        run("Time.at(1234).hash == Time.at(1234).hash"),
        Some(Object::Bool(true))
    );
}

#[test]
fn time_rounds_to_the_asked_for_number_of_digits() {
    assert_eq!(
        run("Time.at(Rational(3, 2)).round.to_i"),
        Some(Object::Int(2))
    );
    assert_eq!(
        run("Time.at(Rational(3, 2)).floor.to_i"),
        Some(Object::Int(1))
    );
    assert_eq!(
        run("Time.at(Rational(3, 2)).ceil.to_i"),
        Some(Object::Int(2))
    );
    assert_eq!(
        run("Time.at(1, 123456).round(3).usec"),
        Some(Object::Int(123000))
    );
}

// ── Moving between UTC and a zone ──────────────────────────────────────────

#[test]
fn time_moves_itself_into_utc_and_answers_a_copy_that_is() {
    assert_eq!(
        run("held = Time.at(0)\nheld.utc\nheld.utc?"),
        Some(Object::Bool(true))
    );
    assert_eq!(run("Time.at(0).getutc.utc?"), Some(Object::Bool(true)));
    assert_eq!(run("Time.at(0).getgm.zone"), Some(Object::string("UTC")));
    assert_eq!(
        run("held = Time.utc(2000, 1, 1)\nheld.localtime(9 * 3600).hour"),
        Some(Object::Int(9))
    );
    assert_eq!(
        run("Time.utc(2000, 1, 1).getlocal(9 * 3600).utc_offset"),
        Some(Object::Int(32400))
    );
}

#[test]
fn time_with_a_fixed_offset_names_no_zone() {
    assert_eq!(
        run("Time.new(2000, 1, 1, 0, 0, 0, 3600).zone.inspect"),
        Some(Object::string("nil"))
    );
}

#[test]
fn time_to_a_lists_the_calendar_fields() {
    assert_eq!(
        run("Time.utc(2000, 1, 2, 3, 4, 5).to_a.inspect"),
        Some(Object::string(
            "[5, 4, 3, 2, 1, 2000, 0, 2, false, \"UTC\"]"
        ))
    );
}

#[test]
fn time_deconstruct_keys_answers_only_the_names_it_holds() {
    assert_eq!(
        run("Time.utc(2022, 10, 5).deconstruct_keys([:year, :a, :month]).inspect"),
        Some(Object::string("{year: 2022, month: 10}"))
    );
    assert_eq!(
        run("Time.utc(2022, 10, 5).deconstruct_keys(nil).keys.size"),
        Some(Object::Int(11))
    );
}

#[test]
fn time_deconstruct_keys_refuses_anything_but_a_list_or_nil() {
    let err = run_err("Time.utc(2022).deconstruct_keys(:year)");
    assert!(err.contains("expected Array or nil"), "Error was: {}", err);
}

// ── A write to ENV reaches the C library ───────────────────────────────────

#[test]
fn setting_the_time_zone_through_env_changes_what_a_local_time_reads() {
    assert_eq!(
        run("ENV[\"TZ\"] = \"UTC\"\nTime.at(0).hour"),
        Some(Object::Int(0))
    );
    assert_eq!(
        run("ENV[\"TZ\"] = \"UTC\"\nTime.at(0).wday"),
        Some(Object::Int(4))
    );
}

#[test]
fn setting_the_time_zone_through_the_store_method_reaches_it_too() {
    assert_eq!(
        run("ENV.[]=(\"TZ\", \"UTC\")\nTime.at(0).zone"),
        Some(Object::string("UTC"))
    );
}
