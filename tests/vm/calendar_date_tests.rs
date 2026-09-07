// Coverage tests for Date

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

/// Loading and running the date library nests deeper than the stack a test
/// thread is given, so each program runs on a thread sized like the one the
/// binary itself uses.
fn on_a_deep_stack(work: impl FnOnce() -> String + Send + 'static) -> String {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(work)
        .expect("thread failed")
        .join()
        .expect("thread panicked")
}

/// The value a program answers, written the way `inspect` writes it.
fn shown(code: &str) -> String {
    let held = format!("__answered__ = begin\n{code}\nend\n__answered__.inspect");
    on_a_deep_stack(move || {
        let tokens = Lexer::new(&held).tokenize();
        let stmts = Parser::new(tokens).parse().expect("parse failed");
        let mut vm = VirtualMachine::new();
        match vm.execute_program(&stmts).expect("execution failed") {
            Some(Object::String(written)) => written.to_string(),
            other => panic!("expected a string, got {other:?}"),
        }
    })
}

/// The text a program answers, with the quotes `inspect` puts around a
/// string taken back off.
fn quoted(code: &str) -> String {
    let written = shown(code);
    written
        .strip_prefix('"')
        .and_then(|held| held.strip_suffix('"'))
        .expect("expected a string")
        .to_string()
}

fn run_err(code: &str) -> String {
    let held = code.to_string();
    on_a_deep_stack(move || {
        let tokens = Lexer::new(&held).tokenize();
        let stmts = Parser::new(tokens).parse().expect("parse failed");
        let mut vm = VirtualMachine::new();
        vm.execute_program(&stmts).unwrap_err().to_string()
    })
}

const DAY: &str = "require 'date'\nday = Date.civil(2008, 1, 16)\n";

// ── Building a date ────────────────────────────────────────────────────────

#[test]
fn a_civil_date_reports_the_julian_day_number_it_stands_for() {
    assert_eq!(shown(&format!("{DAY}day.jd")), "2454482");
    assert_eq!(shown(&format!("{DAY}day.mjd")), "54481");
    assert_eq!(shown(&format!("{DAY}day.ld")), "155322");
}

#[test]
fn the_calendar_fields_read_back_off_the_julian_day_number() {
    assert_eq!(shown(&format!("{DAY}day.year")), "2008");
    assert_eq!(shown(&format!("{DAY}day.month")), "1");
    assert_eq!(shown(&format!("{DAY}day.day")), "16");
    assert_eq!(shown(&format!("{DAY}day.yday")), "16");
    assert_eq!(shown(&format!("{DAY}day.wday")), "3");
}

#[test]
fn the_commercial_fields_count_weeks_that_open_on_monday() {
    assert_eq!(shown(&format!("{DAY}day.cwyear")), "2008");
    assert_eq!(shown(&format!("{DAY}day.cweek")), "3");
    assert_eq!(shown(&format!("{DAY}day.cwday")), "3");
    assert_eq!(shown("require 'date'\nDate.civil(2010, 1, 1).cweek"), "53");
}

#[test]
fn a_date_is_built_from_a_julian_day_an_ordinal_or_a_commercial_week() {
    assert_eq!(shown(&format!("{DAY}Date.jd(2454482) == day")), "true");
    assert_eq!(
        shown(&format!("{DAY}Date.ordinal(2008, 16) == day")),
        "true"
    );
    assert_eq!(
        shown(&format!("{DAY}Date.commercial(2008, 3, 3) == day")),
        "true"
    );
    assert_eq!(
        shown("require 'date'\nDate.ordinal(2007, -100).to_s"),
        "\"2007-09-23\""
    );
}

#[test]
fn an_impossible_date_raises() {
    assert!(run_err("require 'date'\nDate.civil(2007, 2, 29)").contains("invalid date"));
    assert!(run_err("require 'date'\nDate.civil(2007, 13, 1)").contains("invalid date"));
    assert!(run_err("require 'date'\nDate.commercial(2007, 54, 1)").contains("invalid date"));
}

// ── Which calendar a date is read on ───────────────────────────────────────

#[test]
fn the_day_before_the_reform_is_julian_and_the_day_after_is_gregorian() {
    assert_eq!(
        quoted("require 'date'\nDate.civil(1582, 10, 4).to_s"),
        "1582-10-04"
    );
    assert_eq!(
        quoted("require 'date'\n(Date.civil(1582, 10, 4) + 1).to_s"),
        "1582-10-15"
    );
    assert_eq!(
        shown("require 'date'\nDate.civil(1582, 10, 4).julian?"),
        "true"
    );
    assert_eq!(
        shown("require 'date'\n(Date.civil(1582, 10, 4) + 1).gregorian?"),
        "true"
    );
}

#[test]
fn england_moved_to_the_gregorian_calendar_later_than_italy() {
    assert_eq!(
        shown("require 'date'\nDate.civil(1582, 10, 10, Date::ENGLAND).jd"),
        "2299166"
    );
    assert_eq!(
        shown("require 'date'\nDate.civil(1700, 1, 1).new_start(Date::ENGLAND).to_s"),
        "\"1699-12-22\""
    );
}

#[test]
fn a_month_added_across_the_reform_lands_on_the_last_day_before_the_gap() {
    assert_eq!(
        quoted("require 'date'\n(Date.civil(1582, 9, 10) >> 1).to_s"),
        "1582-10-04"
    );
}

// ── Moving about ───────────────────────────────────────────────────────────

#[test]
fn adding_and_subtracting_days_walks_the_julian_day_number() {
    assert_eq!(quoted(&format!("{DAY}(day + 30).to_s")), "2008-02-15");
    assert_eq!(quoted(&format!("{DAY}(day - 30).to_s")), "2007-12-17");
    assert_eq!(
        shown("require 'date'\nDate.civil(2008, 3, 1) - Date.civil(2008, 2, 1)"),
        "29"
    );
}

#[test]
fn adding_months_clamps_to_the_last_day_a_month_holds() {
    assert_eq!(
        quoted("require 'date'\n(Date.civil(2008, 3, 31) >> 1).to_s"),
        "2008-04-30"
    );
    assert_eq!(
        quoted("require 'date'\n(Date.civil(2008, 3, 31) << 1).to_s"),
        "2008-02-29"
    );
}

#[test]
fn shifting_by_a_symbol_raises_a_type_error() {
    assert!(run_err(&format!("{DAY}day >> :soon")).contains("TypeError"));
    assert!(run_err(&format!("{DAY}day + :soon")).contains("TypeError"));
}

#[test]
fn upto_downto_and_step_walk_a_range_of_days() {
    assert_eq!(
        quoted(&format!(
            "{DAY}found = []\nday.upto(day + 3) {{ |each| found.push(each.day) }}\nfound.inspect"
        )),
        "[16, 17, 18, 19]"
    );
    assert_eq!(
        quoted(&format!(
            "{DAY}found = []\nday.downto(day - 2) {{ |each| found.push(each.day) }}\nfound.inspect"
        )),
        "[16, 15, 14]"
    );
    assert_eq!(
        quoted(&format!(
            "{DAY}day.step(day + 6, 3).map {{ |each| each.day }}.inspect"
        )),
        "[16, 19, 22]"
    );
}

// ── Asking about a year ────────────────────────────────────────────────────

#[test]
fn a_leap_year_holds_a_twenty_ninth_of_february() {
    assert_eq!(shown("require 'date'\nDate.leap?(2008)"), "true");
    assert_eq!(shown("require 'date'\nDate.leap?(1900)"), "false");
    assert_eq!(shown("require 'date'\nDate.julian_leap?(1900)"), "true");
}

#[test]
fn the_validity_questions_answer_without_raising() {
    assert_eq!(
        shown("require 'date'\nDate.valid_civil?(1582, 10, 14)"),
        "false"
    );
    assert_eq!(
        shown("require 'date'\nDate.valid_civil?(1582, -3, -21)"),
        "true"
    );
    assert_eq!(
        shown("require 'date'\nDate.valid_ordinal?(2007, 366)"),
        "false"
    );
    assert_eq!(
        shown("require 'date'\nDate.valid_commercial?(1999, 53, 1)"),
        "false"
    );
    assert_eq!(shown("require 'date'\nDate.valid_jd?(:soon)"), "false");
}

// ── Reading and writing text ───────────────────────────────────────────────

#[test]
fn strftime_writes_the_directives_a_template_names() {
    assert_eq!(
        quoted(&format!("{DAY}day.strftime('%A, %B %-d, %Y')")),
        "Wednesday, January 16, 2008"
    );
    assert_eq!(quoted(&format!("{DAY}day.strftime('%j')")), "016");
    assert_eq!(
        quoted(&format!("{DAY}day.strftime('%U %W %V')")),
        "02 02 03"
    );
    assert_eq!(
        quoted(&format!("{DAY}day.strftime('%C %y %G %g')")),
        "20 08 2008 08"
    );
    assert_eq!(quoted(&format!("{DAY}day.strftime('%s')")), "1200441600");
}

#[test]
fn the_gnu_modifiers_pad_shift_case_and_drop_leading_zeros() {
    assert_eq!(quoted(&format!("{DAY}day.strftime('%^b')")), "JAN");
    assert_eq!(quoted(&format!("{DAY}day.strftime('%_5b')")), "  Jan");
    assert_eq!(quoted(&format!("{DAY}day.strftime('%0^5b')")), "00JAN");
    assert_eq!(quoted(&format!("{DAY}day.strftime('%-m')")), "1");
}

#[test]
fn parse_reads_names_numbers_and_separators() {
    assert_eq!(
        quoted("require 'date'\nDate.parse('23 feb 2008').to_s"),
        "2008-02-23"
    );
    assert_eq!(
        quoted("require 'date'\nDate.parse('5th.november.2005').to_s"),
        "2005-11-05"
    );
    assert_eq!(
        quoted("require 'date'\nDate.parse('19101101').to_s"),
        "1910-11-01"
    );
    assert_eq!(
        quoted("require 'date'\nDate.parse('10.01.2007').to_s"),
        "2007-01-10"
    );
    assert_eq!(
        quoted("require 'date'\nDate.parse('10-01-07').to_s"),
        "2010-01-07"
    );
    assert_eq!(
        quoted("require 'date'\nDate.parse('10-01-07', false).to_s"),
        "0010-01-07"
    );
}

#[test]
fn parse_refuses_text_that_names_no_date() {
    assert!(run_err("require 'date'\nDate.parse('1')").contains("invalid date"));
    assert!(run_err("require 'date'\nDate.parse(1)").contains("TypeError"));
}

#[test]
fn strptime_reads_a_date_by_the_template_that_wrote_it() {
    assert_eq!(
        quoted("require 'date'\nDate.strptime('16/01/2008', '%d/%m/%Y').to_s"),
        "2008-01-16"
    );
    assert_eq!(
        quoted("require 'date'\nDate.strptime('Thu Apr  6 00:00:00 2000', '%c').to_s"),
        "2000-04-06"
    );
    assert_eq!(
        quoted("require 'date'\nDate.strptime(' 9-Apr-2000', '%v').to_s"),
        "2000-04-09"
    );
    assert_eq!(
        quoted("require 'date'\nDate.strptime('2010/1', '%Y/%W').to_s"),
        "2010-01-04"
    );
    assert!(run_err("require 'date'\nDate.strptime('nope', '%Y')").contains("invalid date"));
}

#[test]
fn iso8601_reads_and_writes_the_standard_form() {
    assert_eq!(
        quoted("require 'date'\nDate.iso8601('20180715').to_s"),
        "2018-07-15"
    );
    assert_eq!(quoted(&format!("{DAY}day.iso8601")), "2008-01-16");
    assert_eq!(
        quoted(&format!("{DAY}day.rfc3339")),
        "2008-01-16T00:00:00+00:00"
    );
    assert_eq!(
        quoted("require 'date'\nDate._iso8601('nope').inspect"),
        "{}"
    );
}

#[test]
fn a_date_inspects_as_its_julian_day_number_and_reform_point() {
    assert_eq!(
        quoted(&format!("{DAY}day.inspect")),
        "#<Date: 2008-01-16 ((2454482j),(0/1),(2299161j))>"
    );
    assert_eq!(
        quoted(&format!("{DAY}day.asctime")),
        "Wed Jan 16 00:00:00 2008"
    );
}

#[test]
fn dates_compare_and_hash_by_the_day_they_stand_for() {
    assert_eq!(shown(&format!("{DAY}day == Date.jd(2454482)")), "true");
    assert_eq!(shown(&format!("{DAY}day <=> Date.jd(2454483)")), "-1");
    assert_eq!(shown(&format!("{DAY}day.eql?(Date.jd(2454482))")), "true");
    assert_eq!(
        shown(&format!("{DAY}day.hash == Date.jd(2454482).hash")),
        "true"
    );
    assert_eq!(
        quoted(&format!("{DAY}day.deconstruct_keys([:year, :day]).inspect")),
        "{year: 2008, day: 16}"
    );
}

// ── The endless calendars ──────────────────────────────────────────────────

#[test]
fn infinity_compares_greater_or_less_than_every_other_value() {
    assert_eq!(shown("require 'date'\nDate::Infinity.new.infinite?"), "1");
    assert_eq!(
        shown("require 'date'\nDate::Infinity.new(0).infinite?"),
        "nil"
    );
    assert_eq!(shown("require 'date'\nDate::Infinity.new(0).nan?"), "true");
    assert_eq!(
        shown("require 'date'\nDate::Infinity.new(-1) <=> Date::Infinity.new"),
        "-1"
    );
    assert_eq!(
        shown("require 'date'\n(-Date::Infinity.new).infinite?"),
        "-1"
    );
    assert_eq!(
        quoted("require 'date'\nDate::Infinity.new.coerce(1).inspect"),
        "[-1, 1]"
    );
    assert_eq!(
        shown("require 'date'\nDate.civil(2008, 1, 16).julian.julian?"),
        "true"
    );
    assert_eq!(
        shown("require 'date'\nDate.civil(2008, 1, 16).gregorian.gregorian?"),
        "true"
    );
}
