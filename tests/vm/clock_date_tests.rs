// Coverage tests for DateTime and the time library

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

const MOMENT: &str =
    "require 'date'\nmoment = DateTime.new(2012, 12, 24, 13, 45, 30, \"+03:00\")\n";

// ── Building ───────────────────────────────────────────────────────────────

#[test]
fn a_datetime_carries_a_time_of_day_and_an_offset() {
    assert_eq!(shown(&format!("{MOMENT}moment.hour")), "13");
    assert_eq!(shown(&format!("{MOMENT}moment.min")), "45");
    assert_eq!(shown(&format!("{MOMENT}moment.minute")), "45");
    assert_eq!(shown(&format!("{MOMENT}moment.sec")), "30");
    assert_eq!(shown(&format!("{MOMENT}moment.second")), "30");
    assert_eq!(shown(&format!("{MOMENT}moment.sec_fraction")), "(0/1)");
    assert_eq!(shown(&format!("{MOMENT}moment.second_fraction")), "(0/1)");
    assert_eq!(shown(&format!("{MOMENT}moment.offset")), "(1/8)");
    assert_eq!(quoted(&format!("{MOMENT}moment.zone")), "+03:00");
    assert_eq!(
        shown(&format!("{MOMENT}moment.day_fraction")),
        "(1651/2880)"
    );
}

#[test]
fn every_field_defaults_the_way_a_bare_datetime_does() {
    assert_eq!(shown("require 'date'\nDateTime.new.year"), "-4712");
    assert_eq!(shown("require 'date'\nDateTime.new.hour"), "0");
    assert_eq!(shown("require 'date'\nDateTime.new.offset"), "(0/1)");
    assert_eq!(
        quoted("require 'date'\nDateTime.new(2011, 2, 3, 4, 5, 6).to_s"),
        "2011-02-03T04:05:06+00:00"
    );
}

#[test]
fn a_second_may_carry_a_fraction_where_an_hour_and_a_minute_may_not() {
    assert_eq!(
        shown("require 'date'\nDateTime.new(2011, 2, 3, 4, 5, 6.25).sec_fraction"),
        "(1/4)"
    );
    assert_eq!(
        shown("require 'date'\nDateTime.new(2011, 2, 3, 4, 5, Rational(11, 2)).sec"),
        "5"
    );
    assert!(run_err("require 'date'\nDateTime.new(2011, 2, 3, 4.5)").contains("invalid date"));
    assert!(run_err("require 'date'\nDateTime.new(2011, 2, 3, 4, 5.5)").contains("invalid date"));
}

#[test]
fn a_negative_field_counts_back_from_the_unit_above_it() {
    assert_eq!(
        shown("require 'date'\nDateTime.new(2011, 2, 3, -10).hour"),
        "14"
    );
    assert_eq!(
        shown("require 'date'\nDateTime.new(2011, 2, 3, 0, -20).min"),
        "40"
    );
    assert_eq!(
        shown("require 'date'\nDateTime.new(2011, 2, 3, 0, 0, -20).sec"),
        "40"
    );
}

#[test]
fn a_field_outside_its_range_raises() {
    assert!(run_err("require 'date'\nDateTime.new(2011, 2, 3, 25)").contains("invalid date"));
    assert!(run_err("require 'date'\nDateTime.new(2011, 2, 3, -25)").contains("invalid date"));
    assert!(run_err("require 'date'\nDateTime.new(2011, 2, 3, 0, 60)").contains("invalid date"));
    assert!(run_err("require 'date'\nDateTime.new(2011, 2, 3, 0, 0, 60)").contains("invalid date"));
    assert!(run_err("require 'date'\nDateTime.new(2011, 2, 3, 24, 30)").contains("invalid date"));
}

#[test]
fn an_hour_of_twenty_four_names_midnight_on_the_day_after() {
    assert_eq!(
        shown("require 'date'\nDateTime.new(2012, 12, 24, 24).day"),
        "25"
    );
    assert_eq!(
        shown("require 'date'\nDateTime.new(2012, 12, 24, 24).hour"),
        "0"
    );
}

#[test]
fn a_datetime_is_built_from_a_julian_day_an_ordinal_or_a_commercial_week() {
    assert_eq!(
        quoted("require 'date'\nDateTime.jd(2454482, 6).to_s"),
        "2008-01-16T06:00:00+00:00"
    );
    assert_eq!(
        quoted("require 'date'\nDateTime.ordinal(2008, 16, 6).to_s"),
        "2008-01-16T06:00:00+00:00"
    );
    assert_eq!(
        quoted("require 'date'\nDateTime.commercial(2008, 3, 3, 6).to_s"),
        "2008-01-16T06:00:00+00:00"
    );
    assert_eq!(
        quoted("require 'date'\nDateTime.civil(2008, 1, 16).to_s"),
        "2008-01-16T00:00:00+00:00"
    );
}

// ── Offsets ────────────────────────────────────────────────────────────────

#[test]
fn an_offset_may_be_written_as_text_or_as_a_fraction_of_a_day() {
    assert_eq!(
        shown("require 'date'\nDateTime.new(2011, 1, 1, 0, 0, 0, '-05:00').offset"),
        "(-5/24)"
    );
    assert_eq!(
        shown("require 'date'\nDateTime.new(2011, 1, 1, 0, 0, 0, 'Z').offset"),
        "(0/1)"
    );
    assert_eq!(
        shown("require 'date'\nDateTime.new(2011, 1, 1, 0, 0, 0, Rational(1, 4)).offset"),
        "(1/4)"
    );
    assert!(
        run_err("require 'date'\nDateTime.new(2011, 1, 1, 0, 0, 0, 'nowhere')")
            .contains("invalid date")
    );
}

#[test]
fn a_new_offset_moves_the_clock_without_moving_the_instant() {
    assert_eq!(
        quoted(&format!("{MOMENT}moment.new_offset('-05:00').to_s")),
        "2012-12-24T05:45:30-05:00"
    );
    assert_eq!(
        quoted(&format!("{MOMENT}moment.new_offset(0).to_s")),
        "2012-12-24T10:45:30+00:00"
    );
    assert_eq!(
        shown(&format!("{MOMENT}moment.new_offset(0) == moment")),
        "true"
    );
}

// ── Moving about ───────────────────────────────────────────────────────────

#[test]
fn adding_a_fraction_of_a_day_moves_the_clock() {
    assert_eq!(
        quoted(&format!("{MOMENT}(moment + Rational(1, 4)).to_s")),
        "2012-12-24T19:45:30+03:00"
    );
    assert_eq!(
        quoted(&format!("{MOMENT}(moment - Rational(1, 2)).to_s")),
        "2012-12-24T01:45:30+03:00"
    );
    assert_eq!(
        shown("require 'date'\nDateTime.new(2012, 1, 1, 6) - DateTime.new(2012, 1, 1)"),
        "(1/4)"
    );
    assert!(run_err(&format!("{MOMENT}moment + :soon")).contains("TypeError"));
}

#[test]
fn a_difference_below_a_millisecond_is_kept_exactly() {
    assert_eq!(
        shown(
            "require 'date'\nheld = DateTime.new(2017)\nstep = Rational(123456789, 86400000000)\n(held + step) - held"
        ),
        "(13717421/9600000000)"
    );
    assert_eq!(
        shown("require 'date'\n(DateTime.new(2017) + 0.00001001).to_time.usec"),
        "864864"
    );
}

#[test]
fn a_shifted_datetime_keeps_the_time_it_holds() {
    assert_eq!(
        quoted(&format!("{MOMENT}(moment >> 1).to_s")),
        "2013-01-24T13:45:30+03:00"
    );
    assert_eq!(
        quoted(&format!("{MOMENT}moment.next_day.to_s")),
        "2012-12-25T13:45:30+03:00"
    );
}

#[test]
fn datetimes_compare_by_the_instant_they_name() {
    assert_eq!(
        shown("require 'date'\nDateTime.new(2012, 1, 1, 6) > DateTime.new(2012, 1, 1)"),
        "true"
    );
    assert_eq!(
        shown("require 'date'\nDateTime.new(2012, 1, 1) == DateTime.new(2012, 1, 1)"),
        "true"
    );
    assert_eq!(
        shown("require 'date'\nDateTime.new(2012, 1, 1).eql?(DateTime.new(2012, 1, 1))"),
        "true"
    );
    assert_eq!(
        shown("require 'date'\nDateTime.new(2012, 1, 1).hash == DateTime.new(2012, 1, 1).hash"),
        "true"
    );
    assert_eq!(
        shown("require 'date'\nDateTime.new(2012, 1, 1) <=> DateTime.new(2012, 1, 2)"),
        "-1"
    );
}

// ── Reading and writing text ───────────────────────────────────────────────

#[test]
fn strftime_writes_the_clock_as_well_as_the_calendar() {
    assert_eq!(
        quoted(&format!("{MOMENT}moment.strftime('%Y-%m-%d %H:%M:%S %z')")),
        "2012-12-24 13:45:30 +0300"
    );
    assert_eq!(
        quoted(&format!("{MOMENT}moment.strftime('%I:%M %p')")),
        "01:45 PM"
    );
    assert_eq!(
        quoted(&format!("{MOMENT}moment.strftime('%::z')")),
        "+03:00:00"
    );
    assert_eq!(quoted(&format!("{MOMENT}moment.strftime('%:z')")), "+03:00");
    assert_eq!(
        quoted("require 'date'\nDateTime.new(2001, 2, 3, 4, 5, 6.25).strftime('%H:%M:%S.%6N')"),
        "04:05:06.250000"
    );
    assert_eq!(
        quoted("require 'date'\nDateTime.new(2001, 2, 3, 4, 5, 6.25).strftime('%L')"),
        "250"
    );
    assert_eq!(
        quoted("require 'date'\nDateTime.new(2005, 1, 1).strftime('%s')"),
        "1104537600"
    );
}

#[test]
fn parse_and_strptime_read_a_clock_out_of_text() {
    assert_eq!(
        quoted("require 'date'\nDateTime.parse('2012-11-08T15:43:59').to_s"),
        "2012-11-08T15:43:59+00:00"
    );
    assert_eq!(
        quoted(
            "require 'date'\nDateTime.strptime('2012-11-08 15:43:59', '%Y-%m-%d %H:%M:%S').to_s"
        ),
        "2012-11-08T15:43:59+00:00"
    );
    assert_eq!(
        quoted("require 'date'\nDateTime.iso8601('2018-01-01').to_s"),
        "2018-01-01T00:00:00+00:00"
    );
    assert!(
        run_err("require 'date'\nDateTime.parse('2012-12-31T25:43:59')").contains("invalid date")
    );
    assert!(
        run_err("require 'date'\nDateTime.parse('2012-11-08T15:43:61')").contains("invalid date")
    );
}

#[test]
fn a_datetime_writes_itself_in_the_standard_forms() {
    assert_eq!(
        quoted(&format!("{MOMENT}moment.iso8601")),
        "2012-12-24T13:45:30+03:00"
    );
    assert_eq!(
        quoted(&format!("{MOMENT}moment.xmlschema")),
        "2012-12-24T13:45:30+03:00"
    );
    assert_eq!(
        quoted(&format!("{MOMENT}moment.rfc3339")),
        "2012-12-24T13:45:30+03:00"
    );
    assert_eq!(
        quoted(&format!("{MOMENT}moment.rfc2822")),
        "Mon, 24 Dec 2012 13:45:30 +0300"
    );
    assert_eq!(
        quoted(&format!("{MOMENT}moment.httpdate")),
        "Mon, 24 Dec 2012 10:45:30 GMT"
    );
    assert_eq!(
        quoted(&format!("{MOMENT}moment.inspect")),
        "#<DateTime: 2012-12-24T13:45:30+03:00 ((2456286j),(1651/2880),(1/8))>"
    );
}

#[test]
fn deconstruct_keys_reports_the_clock_alongside_the_calendar() {
    assert_eq!(
        shown("require 'date'\nDateTime.new(2022, 10, 5, 13, 30).deconstruct_keys([:zone, :hour])"),
        "{zone: \"+00:00\", hour: 13}"
    );
    assert_eq!(
        shown("require 'date'\nDateTime.new(2022, 10, 5, 13, 30).deconstruct_keys([])"),
        "{}"
    );
    assert!(
        run_err("require 'date'\nDateTime.new(2022, 10, 5).deconstruct_keys(1)")
            .contains("wrong argument type Integer (expected Array or nil)")
    );
}

// ── Crossing to Date and Time ──────────────────────────────────────────────

#[test]
fn a_datetime_converts_to_the_date_and_the_time_it_names() {
    assert_eq!(
        quoted(&format!("{MOMENT}moment.to_date.to_s")),
        "2012-12-24"
    );
    assert_eq!(shown(&format!("{MOMENT}moment.to_date.class")), "Date");
    assert_eq!(
        shown(&format!("{MOMENT}moment.to_datetime == moment")),
        "true"
    );
    assert_eq!(
        quoted("require 'date'\nDateTime.new(2012, 12, 31, 23, 58, 59).to_time.utc.to_s"),
        "2012-12-31 23:58:59 UTC"
    );
    assert_eq!(
        quoted("require 'date'\nDateTime.civil(1582, 10, 4, 23, 58, 59).to_time.utc.to_s"),
        "1582-10-14 23:58:59 UTC"
    );
}

#[test]
fn a_time_converts_to_the_datetime_and_date_it_names() {
    assert_eq!(
        quoted("require 'date'\nTime.utc(2012, 12, 31, 23, 58, 59).to_datetime.to_s"),
        "2012-12-31T23:58:59+00:00"
    );
    assert_eq!(
        quoted("require 'date'\nTime.utc(1582, 10, 14, 23, 58, 59).to_datetime.to_s"),
        "1582-10-04T23:58:59+00:00"
    );
    assert_eq!(
        quoted("require 'date'\nTime.utc(2012, 12, 31).to_date.to_s"),
        "2012-12-31"
    );
    assert_eq!(
        shown("require 'date'\nTime.utc(1582, 10, 15).to_date.jd"),
        "2299161"
    );
    assert_eq!(
        quoted("require 'date'\nDate.civil(2012, 12, 24).to_datetime.to_s"),
        "2012-12-24T00:00:00+00:00"
    );
}

// ── The time library ───────────────────────────────────────────────────────

#[test]
fn the_time_library_reads_and_writes_a_time_as_text() {
    assert_eq!(
        quoted("require 'time'\nTime.parse('2012-11-08T15:43:59Z').utc.to_s"),
        "2012-11-08 15:43:59 UTC"
    );
    assert_eq!(
        quoted("require 'time'\nTime.iso8601('2012-11-08T15:43:59Z').utc.to_s"),
        "2012-11-08 15:43:59 UTC"
    );
    assert_eq!(
        quoted("require 'time'\nTime.utc(2012, 11, 8, 15, 43, 59).iso8601"),
        "2012-11-08T15:43:59Z"
    );
    assert_eq!(
        quoted("require 'time'\nTime.utc(2012, 11, 8, 15, 43, 59).xmlschema(3)"),
        "2012-11-08T15:43:59.000Z"
    );
    assert_eq!(
        quoted("require 'time'\nTime.utc(2012, 11, 8, 15, 43, 59).httpdate"),
        "Thu, 08 Nov 2012 15:43:59 GMT"
    );
    assert_eq!(
        quoted("require 'time'\nTime.utc(2012, 11, 8, 15, 43, 59).rfc2822"),
        "Thu, 8 Nov 2012 15:43:59 +0000"
    );
}
