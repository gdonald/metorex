// Checksums, deflate streams and gzip members.

use super::*;

#[test]
fn each_checksum_answers_the_value_its_definition_gives() {
    let result = run(r#"
require 'zlib'
[Zlib.crc32(""), Zlib.crc32(" "), Zlib.crc32("123456789"),
 Zlib.adler32(""), Zlib.adler32("123456789")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[0, 3916222277, 3421780262, 1, 152961502]".to_string())
    );
}

#[test]
fn a_checksum_carries_on_from_the_value_it_is_given() {
    let result = run(r#"
require 'zlib'
held = "This is a test string! How exciting!%?"
[Zlib.crc32(held, 0), Zlib.crc32(held, 1), Zlib.crc32("p", -305419897)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[3864990561, 1809313411, 4046865307]".to_string())
    );
}

#[test]
fn a_checksum_refuses_a_starting_value_too_wide_to_be_one() {
    let error = run_err(
        r#"
require 'zlib'
Zlib.crc32("held", 2 ** 128)
"#,
    );
    assert!(error.contains("bignum too big"), "{error}");
}

#[test]
fn a_stream_another_zlib_wrote_reads_back_here() {
    let result = run(r#"
require 'zlib'
written = [120, 156, 99, 96, 128, 1, 0, 0, 10, 0, 1].pack("C*")
Zlib.inflate(written) == "\000" * 10
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn a_stream_this_writes_reads_back_as_what_it_was_given() {
    let result = run(r#"
require 'zlib'
held = "the quick brown fox " * 40
[Zlib.inflate(Zlib.deflate(held)) == held, Zlib.gunzip(Zlib.gzip(held)) == held]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true]".to_string())
    );
}

#[test]
fn a_stream_that_is_not_one_is_refused() {
    let error = run_err(
        r#"
require 'zlib'
Zlib.gunzip("not a member")
"#,
    );
    assert!(error.contains("not a stream this can read"), "{error}");
}

#[test]
fn a_gzip_member_names_what_it_holds_and_when_it_was_written() {
    let result = run(r#"
require 'zlib'
require 'stringio'
member = [31, 139, 8, 0, 44, 220, 209, 71, 0, 3, 51, 52, 50, 54, 49, 77,
          76, 74, 78, 73, 5, 0, 157, 5, 0, 36, 10, 0, 0, 0].pack("C*")
reader = Zlib::GzipReader.new(StringIO.new(member))
held = reader.read
finished = reader.eof?
reader.close
[held, finished]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[12345abcde, true]".to_string())
    );
}

#[test]
fn a_closed_member_has_nothing_left_to_say_about_itself() {
    let result = run(r#"
require 'zlib'
require 'stringio'
reader = Zlib::GzipReader.new(StringIO.new(Zlib.gzip("held")))
reader.close
begin
  reader.orig_name
rescue Zlib::GzipFile::Error => problem
  problem.message
end
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("closed gzip stream".to_string())
    );
}

#[test]
fn a_template_stands_for_the_text_its_tags_build() {
    let result = run(r#"
require 'erb'
list = %w[a b c]
ERB.new("<% list.each do |item| %><%= item %>;<% end %>").result(binding)
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("a;b;c;".to_string())
    );
}

#[test]
fn what_a_template_puts_into_a_page_is_escaped_for_it() {
    let result = run(r#"
require 'erb'
[ERB::Util.html_escape("<a href='x'>&</a>"), ERB::Util.url_encode("a b/c")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[&lt;a href=&#39;x&#39;&gt;&amp;&lt;/a&gt;, a%20b%2Fc]".to_string())
    );
}

#[test]
fn a_template_can_be_written_onto_a_class_as_a_method() {
    let result = run(r#"
require 'erb'
built = ERB.new("<%= @held %> is here").def_class(Object, "render")
made = built.new
made.instance_variable_set(:@held, "metorex")
made.render
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("metorex is here".to_string())
    );
}

#[test]
fn a_command_answers_what_it_wrote_and_how_it_ended() {
    let result = run(r#"
require 'open3'
output, status = Open3.capture2("echo written")
[output, status.exitstatus]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[written\n, 0]".to_string())
    );
}

#[test]
fn a_command_keeps_its_two_streams_apart_when_asked_to() {
    let result = run(r#"
require 'open3'
out, errors, _status = Open3.capture3("sh -c 'echo out; echo err 1>&2'")
[out, errors]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[out\n, err\n]".to_string())
    );
}

#[test]
fn a_stream_that_names_its_own_code_reads_back_here() {
    // A dynamic block carries the code it was written with, which the
    // decoder has to read before it can read anything else.
    let result = run(r#"
require 'zlib'
written = ([120, 156, 237, 193, 1, 1, 0, 0] +
           [0, 128, 144, 254, 175, 238, 8, 10] +
           Array.new(31, 0) +
           [24, 128, 0, 0, 1]).pack("C*")
Zlib.inflate(written) == "\000" * 32 * 1024
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn a_stream_with_no_header_in_front_of_it_reads_back_too() {
    let result = run(r#"
require 'zlib'
held = "a stream with no header"
written = Zlib.deflate(held)
raw = written[2, written.length - 6]
Zlib::Inflate.new(-Zlib::MAX_WBITS).inflate(raw)
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("a stream with no header".to_string())
    );
}

#[test]
fn a_stream_of_nothing_reads_back_as_nothing() {
    let result = run(r#"
require 'zlib'
[Zlib.inflate(Zlib.deflate("")), Zlib.gunzip(Zlib.gzip(""))]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[, ]".to_string())
    );
}

#[test]
fn a_gzip_member_carries_the_name_it_was_written_with() {
    let result = run(r#"
require 'zlib'
require 'stringio'
holder = StringIO.new(+"")
writer = Zlib::GzipWriter.new(holder)
writer.orig_name = "held.txt"
writer.mtime = 1234567
writer.write("what it holds")
writer.close
reader = Zlib::GzipReader.new(StringIO.new(holder.string))
answered = [reader.read, reader.orig_name, reader.mtime.to_i]
reader.close
answered
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[what it holds, held.txt, 1234567]".to_string())
    );
}

#[test]
fn a_stream_action_nothing_answers_to_is_refused() {
    let error = run_err(
        r#"
require 'zlib'
Zlib.__stream__("no_such_action", "held", 0)
"#,
    );
    assert!(error.contains("unknown stream action"), "{error}");
}

#[test]
fn a_stream_asked_for_with_nothing_to_act_on_is_refused() {
    let error = run_err(
        r#"
require 'zlib'
Zlib.__stream__
"#,
    );
    assert!(error.contains("wrong number of arguments"), "{error}");
}

#[test]
fn a_stream_that_is_cut_short_is_refused() {
    let error = run_err(
        r#"
require 'zlib'
Zlib.inflate([120, 156, 99].pack("C*"))
"#,
    );
    assert!(error.contains("not a stream this can read"), "{error}");
}
