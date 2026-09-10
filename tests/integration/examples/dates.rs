// Examples covering Time

use super::run_example;

/// The expected output of both `dates/points_in_time` variants, which differ
/// only in whether the calls are written with parentheses.
const POINTS_IN_TIME_OUTPUT: &str = "2007\n11\n1\n15\n25\n30\n4\n305\n123456\n123456000\n(1929/15625)\ntrue\n\"UTC\"\n0\n1193930730\n1193930730.123456\n(18655167658179/15625)\n[30, 25, 15, 1, 11, 2007, 4, 305, false, \"UTC\"]\n\"2007-11-01 15:25:30.123456 UTC\"\n\"2007-11-01 15:25:30 UTC\"\n\"Thu Nov  1 15:25:30 2007\"\n\"2007-11-01T15:25:30Z\"\n\"2007-11-01T15:25:30.123Z\"\n\"2007/11/01 15:25:30\"\n{year: 2007, month: 11, day: 1}\n100.0001\n(1/2)\n(3/4)\n1.0\n101\ntrue\ntrue\ntrue\nfalse\ntrue\n482196050\n482196050\n7245\nnil\n1\n";

#[test]
fn test_dates_points_in_time_execution() {
    let output = run_example("dates/points_in_time.rb");
    assert_eq!(output, POINTS_IN_TIME_OUTPUT);
}

#[test]
fn test_dates_points_in_time_no_parens_execution() {
    let output = run_example("dates/points_in_time_no_parens.rb");
    assert_eq!(output, POINTS_IN_TIME_OUTPUT);
}

/// The expected output of both `dates/calendar_days` variants, which differ
/// only in whether the calls are written with parentheses.
const CALENDAR_DAYS_OUTPUT: &str = "2454482\n2008\n1\n16\n16\n3\n2008\n3\n3\n54481\n(4908963/2)\ntrue\ntrue\n\"2008-01-16\"\n\"#<Date: 2008-01-16 ((2454482j),(0/1),(2299161j))>\"\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n\"2008-02-15\"\n\"2007-12-17\"\n\"2008-03-16\"\n\"2007-12-16\"\n\"2008-01-17\"\n\"2008-02-16\"\n\"2009-01-16\"\n29\ntrue\ntrue\nfalse\ntrue\ntrue\n\"Wednesday, January 16, 2008\"\n\"Wed Jan 16 2008\"\n\"016 02 02 03\"\n\"       JAN\"\n\"1582-10-04\"\n\"1582-10-15\"\ntrue\ntrue\ntrue\n[16, 17, 18, 19]\n[16, 18, 20, 22]\n";

#[test]
fn test_dates_calendar_days_execution() {
    let output = run_example("dates/calendar_days.rb");
    assert_eq!(output, CALENDAR_DAYS_OUTPUT);
}

#[test]
fn test_dates_calendar_days_no_parens_execution() {
    let output = run_example("dates/calendar_days_no_parens.rb");
    assert_eq!(output, CALENDAR_DAYS_OUTPUT);
}

/// The expected output of both `dates/clock_and_calendar` variants, which
/// differ only in whether the calls are written with parentheses.
const CLOCK_AND_CALENDAR_OUTPUT: &str = "2012\n12\n24\n13\n45\n30\n(0/1)\n(1/8)\n\"+03:00\"\n\"2012-12-24T13:45:30+03:00\"\n(1651/2880)\n\"2012-12-24T05:45:30-05:00\"\n\"2012-12-24T10:45:30+00:00\"\n\"2012-12-24\"\nDate\n\"2012-12-24T00:00:00+00:00\"\n\"2012-12-24 13:45:30 +0300\"\n\"01:45 PM\"\n\"Monday, 24 December 2012\"\n\"+03:00:00\"\n\"04:05:06.250000\"\n\"250\"\n\"2012-11-08T15:43:59+00:00\"\n\"2012-11-08T15:43:59+00:00\"\n\"2018-01-01T00:00:00+00:00\"\n\"2012-12-24T19:45:30+03:00\"\n\"2012-12-24T01:45:30+03:00\"\n(1/1)\n(1/4)\ntrue\n25\n0\n\"2012-12-31T23:58:59+00:00\"\n\"2012-12-31\"\n\"2012-12-31 23:58:59 UTC\"\n";

#[test]
fn test_dates_clock_and_calendar_execution() {
    let output = run_example("dates/clock_and_calendar.rb");
    assert_eq!(output, CLOCK_AND_CALENDAR_OUTPUT);
}

#[test]
fn test_dates_clock_and_calendar_no_parens_execution() {
    let output = run_example("dates/clock_and_calendar_no_parens.rb");
    assert_eq!(output, CLOCK_AND_CALENDAR_OUTPUT);
}

/// The expected output of both `dates/zone_names/utc` variants.
const ZONE_NAMES_OUTPUT: &str = concat!(
    "\"UTC\"\n",
    "\"UTC\"\n",
    "\"UTC\"\n",
    "\"UTC\"\n",
    "\"UTC\"\n",
    "\"UTC\"\n",
    "#<Encoding:US-ASCII>\n",
    "true\n",
    "nil\n"
);

#[test]
fn test_dates_zone_names_utc_execution() {
    let output = run_example("dates/zone_names/utc.rb");
    assert_eq!(output, ZONE_NAMES_OUTPUT);
}

#[test]
fn test_dates_zone_names_utc_parens_execution() {
    let output = run_example("dates/zone_names/utc_parens.rb");
    assert_eq!(output, ZONE_NAMES_OUTPUT);
}
