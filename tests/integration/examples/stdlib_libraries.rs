// Examples covering the libraries metorex carries

use super::run_example;

/// The expected output of both `stdlib_libraries/shipped_libraries` variants,
/// which differ only in whether the calls are written with parentheses.
const SHIPPED_LIBRARIES_OUTPUT: &str = "\"Tm93IGlzIHRoZSB0aW1lIGZvciBhbGwgZ29vZCBjb2RlcnMKdG8gbGVhcm4g\\nUnVieQ==\\n\"\n\"aGVsbG8=\"\n\"hello\"\n\"hello\"\n\"aGVsbG8_\"\n\"aGVsbG8_\"\n\"hello?\"\n[\"ruby\", \"-e\", \"puts 1\", \"--name\", \"value\"]\n\"a\\\\ b\"\n\"ruby a\\\\ b\"\n[\"one\", \"two three\"]\n{\"ruby\" => \"ruby\", \"rub\" => \"ruby\", \"ru\" => \"ruby\", \"r\" => \"ruby\"}\n{\"car\" => \"car\", \"ca\" => \"car\", \"cone\" => \"cone\", \"con\" => \"cone\", \"co\" => \"cone\"}\ntrue\n0\nNoMethodError\nTypeError\n0\n1\n:moved\n0\n32\n8\n\"\"\n36\nInteger\n7\n7\n\"e\"\n1\n\"xam\"\n4\nfalse\nnil\nnil\nfalse\ntrue\ntrue\nfalse\ntrue\n\"first\\n\"\n1\n[\"second\\n\"]\n0\n[\"first\\n\", \"second\\n\"]\n[\"a\", \"b\", \"c\"]\n[97, 98, 99]\n\"a\"\n97\n\"one two!\\nthree\"\n14\n13\n0\n2\n0\n\"exam\"\nIOError\n\"Ada\"\n1843\n\"computing\"\n\"Ada\"\n\"London\"\n{name: \"Ada\", year: 1843, field: \"computing\", city: \"London\"}\n\"#<OpenStruct name=\\\"Ada\\\", year=1843, field=\\\"computing\\\", city=\\\"London\\\">\"\ntrue\nfalse\nnil\ntrue\n\"#<OpenStruct>\"\ntrue\nfalse\n[2, 3, 5, 7, 11]\n[[2, 3], [3, 2], [5, 1]]\n72\n[[-1, 1], [2, 1], [5, 1]]\ntrue\n[[2, 2], [3, 1]]\n20\n2\n3\n2\n";

#[test]
fn test_stdlib_libraries_shipped_libraries_execution() {
    let output = run_example("stdlib_libraries/shipped_libraries.rb");
    assert_eq!(output, SHIPPED_LIBRARIES_OUTPUT);
}

#[test]
fn test_stdlib_libraries_shipped_libraries_no_parens_execution() {
    let output = run_example("stdlib_libraries/shipped_libraries_no_parens.rb");
    assert_eq!(output, SHIPPED_LIBRARIES_OUTPUT);
}

/// The expected output of both `stdlib_libraries/reading_the_user_database`
/// variants.
const READING_THE_USER_DATABASE_OUTPUT: &str = "Etc::Passwd\nString\ntrue\ntrue\nString\nString\ntrue\nEtc::Group\nString\ntrue\nArray\nHash\n[:sysname, :nodename, :release, :version, :machine]\ntrue\ntrue\nString\n\"/etc\"\nString\ntrue\nRuntimeError\n\"no implicit conversion of String into Integer\"\n";

#[test]
fn test_stdlib_libraries_reading_the_user_database_execution() {
    let output = run_example("stdlib_libraries/reading_the_user_database.rb");
    assert_eq!(output, READING_THE_USER_DATABASE_OUTPUT);
}

#[test]
fn test_stdlib_libraries_reading_the_user_database_no_parens_execution() {
    let output = run_example("stdlib_libraries/reading_the_user_database_no_parens.rb");
    assert_eq!(output, READING_THE_USER_DATABASE_OUTPUT);
}

#[test]
fn test_stdlib_libraries_message_digests_execution() {
    let expected = concat!(
        "\"900150983cd24fb0d6963f7d28e17f72\"\n",
        "\"a9993e364706816aba3e25717850c26c9cd0d89d\"\n",
        "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\"\n",
        "\"cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7\"\n",
        "\"ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f\"\n",
        "\"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"\n",
        "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\"\n",
        "32\n",
        "64\n",
        "true\n",
        "true\n",
        "true\n",
        "\"#<Digest::MD5: d41d8cd98f00b204e9800998ecf8427e>\"\n",
        "true\n",
        "\"47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=\"\n",
        "\"73616d706c6520737472696e67\"\n",
        "\"xexax\"\n",
        "\"xinik-zorox\"\n",
        "\"xesef-disof-gytuf-katof-movif-baxux\"\n",
    );
    let output = run_example("stdlib_libraries/message_digests.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_message_digests_parens_execution() {
    let expected = concat!(
        "\"900150983cd24fb0d6963f7d28e17f72\"\n",
        "\"a9993e364706816aba3e25717850c26c9cd0d89d\"\n",
        "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\"\n",
        "\"cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7\"\n",
        "\"ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f\"\n",
        "\"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"\n",
        "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\"\n",
        "32\n",
        "64\n",
        "true\n",
        "true\n",
        "true\n",
        "\"#<Digest::MD5: d41d8cd98f00b204e9800998ecf8427e>\"\n",
        "true\n",
        "\"47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=\"\n",
        "\"73616d706c6520737472696e67\"\n",
        "\"xexax\"\n",
        "\"xinik-zorox\"\n",
        "\"xesef-disof-gytuf-katof-movif-baxux\"\n",
    );
    let output = run_example("stdlib_libraries/message_digests_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_delegation_execution() {
    let expected = concat!(
        "[1, 2, 3]\n",
        "3\n",
        "true\n",
        "true\n",
        "false\n",
        "\"TEXT\"\n",
        "\"text\"\n",
        "\"HELLO!\"\n",
        "5\n",
        "true\n",
        "1\n",
        "6\n",
        ":named\n",
        "true\n",
    );
    let output = run_example("stdlib_libraries/delegation.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_delegation_parens_execution() {
    let expected = concat!(
        "[1, 2, 3]\n",
        "3\n",
        "true\n",
        "true\n",
        "false\n",
        "\"TEXT\"\n",
        "\"text\"\n",
        "\"HELLO!\"\n",
        "5\n",
        "true\n",
        "1\n",
        "6\n",
        ":named\n",
        "true\n",
    );
    let output = run_example("stdlib_libraries/delegation_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_logging_and_scratch_files_execution() {
    let expected = concat!(
        "2\n",
        "true\n",
        "false\n",
        "2\n",
        "true\n",
        "3\n",
        "false\n",
        "true\n",
        "true\n",
        "false\n",
        "ThreadError\n",
        "true\n",
        "true\n",
        "nil\n",
    );
    let output = run_example("stdlib_libraries/logging_and_scratch_files.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_logging_and_scratch_files_parens_execution() {
    let expected = concat!(
        "2\n",
        "true\n",
        "false\n",
        "2\n",
        "true\n",
        "3\n",
        "false\n",
        "true\n",
        "true\n",
        "false\n",
        "ThreadError\n",
        "true\n",
        "true\n",
        "nil\n",
    );
    let output = run_example("stdlib_libraries/logging_and_scratch_files_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_decimal_numbers_execution() {
    let expected = concat!(
        "\"0.33333333333333333333333333333333e0\"\n",
        "\"0.3e0\"\n",
        "false\n",
        "true\n",
        "[-1, \"12345\", 10, 3]\n",
        "3\n",
        "5\n",
        "\"0.12345e3\"\n",
        "\"123.45\"\n",
        "\"10 00010.0\"\n",
        "\"0.14142135623730950488e1\"\n",
        "\"0.1e1\"\n",
        "[\"2\", \"0.1e1\"]\n",
        "\"0.1024e4\"\n",
        "0.2e1\n",
        "0.3e1\n",
        "-2\n",
        "-1\n",
        "\"0.198e1\"\n",
        "true\n",
        "1\n",
        "true\n",
        "true\n",
        "\"0.0\"\n",
    );
    let output = run_example("stdlib_libraries/decimal_numbers.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_decimal_numbers_parens_execution() {
    let expected = concat!(
        "\"0.33333333333333333333333333333333e0\"\n",
        "\"0.3e0\"\n",
        "false\n",
        "true\n",
        "[-1, \"12345\", 10, 3]\n",
        "3\n",
        "5\n",
        "\"0.12345e3\"\n",
        "\"123.45\"\n",
        "\"10 00010.0\"\n",
        "\"0.14142135623730950488e1\"\n",
        "\"0.1e1\"\n",
        "[\"2\", \"0.1e1\"]\n",
        "\"0.1024e4\"\n",
        "0.2e1\n",
        "0.3e1\n",
        "-2\n",
        "-1\n",
        "\"0.198e1\"\n",
        "true\n",
        "1\n",
        "true\n",
        "true\n",
        "\"0.0\"\n",
    );
    let output = run_example("stdlib_libraries/decimal_numbers_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_compressed_streams_execution() {
    let expected = concat!(
        "0\n",
        "3421780262\n",
        "152961502\n",
        "256\n",
        "true\n",
        "true\n",
        "true\n",
        "\"12345abcde\"\n",
        "true\n",
        "[\"one\\n\", \"two\\n\"]\n",
        "[\"a\", \"b\"]\n",
        "\"closed gzip stream\"\n",
    );
    let output = run_example("stdlib_libraries/compressed_streams.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_compressed_streams_parens_execution() {
    let expected = concat!(
        "0\n",
        "3421780262\n",
        "152961502\n",
        "256\n",
        "true\n",
        "true\n",
        "true\n",
        "\"12345abcde\"\n",
        "true\n",
        "[\"one\\n\", \"two\\n\"]\n",
        "[\"a\", \"b\"]\n",
        "\"closed gzip stream\"\n",
    );
    let output = run_example("stdlib_libraries/compressed_streams_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_templates_and_commands_execution() {
    let expected = concat!(
        "\"AAA;BBB;CCC;\"\n",
        "\"&lt;a href=&#39;x&#39;&gt;&amp;&lt;/a&gt;\"\n",
        "\"a%20b%2Fc\"\n",
        "\"metorex is here\"\n",
        "\"hello, world\"\n",
        "\"written\\n\"\n",
        "0\n",
        "[\"err\", \"out\"]\n",
        "[\"out\\n\", \"err\\n\", 0]\n",
    );
    let output = run_example("stdlib_libraries/templates_and_commands.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_templates_and_commands_parens_execution() {
    let expected = concat!(
        "\"AAA;BBB;CCC;\"\n",
        "\"&lt;a href=&#39;x&#39;&gt;&amp;&lt;/a&gt;\"\n",
        "\"a%20b%2Fc\"\n",
        "\"metorex is here\"\n",
        "\"hello, world\"\n",
        "\"written\\n\"\n",
        "0\n",
        "[\"err\", \"out\"]\n",
        "[\"out\\n\", \"err\\n\", 0]\n",
    );
    let output = run_example("stdlib_libraries/templates_and_commands_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_keys_and_escapes_execution() {
    let expected = concat!(
        "\"SHA1\"\n",
        "\"SHA256\"\n",
        "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\"\n",
        "20\n",
        "128\n",
        "\"de7c9b85b8b78aa6bc8a7a36f70a90701c9db4d9\"\n",
        "16\n",
        "true\n",
        "true\n",
        "false\n",
        "\"a+b%26c~\"\n",
        "\"a b&c\"\n",
        "\"a%20b%2Fc\"\n",
        "\"a b/c\"\n",
        "\"&amp; &lt; &gt; &quot; &#39;\"\n",
        "\"& < > \\\" c\"\n",
        "\"<BR>&lt;A HREF=&quot;url&quot;&gt;&lt;/A&gt;\"\n",
        "\"<BR><A HREF=\\\"url\\\">\"\n",
    );
    let output = run_example("stdlib_libraries/keys_and_escapes.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_keys_and_escapes_parens_execution() {
    let expected = concat!(
        "\"SHA1\"\n",
        "\"SHA256\"\n",
        "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\"\n",
        "20\n",
        "128\n",
        "\"de7c9b85b8b78aa6bc8a7a36f70a90701c9db4d9\"\n",
        "16\n",
        "true\n",
        "true\n",
        "false\n",
        "\"a+b%26c~\"\n",
        "\"a b&c\"\n",
        "\"a%20b%2Fc\"\n",
        "\"a b/c\"\n",
        "\"&amp; &lt; &gt; &quot; &#39;\"\n",
        "\"& < > \\\" c\"\n",
        "\"<BR>&lt;A HREF=&quot;url&quot;&gt;&lt;/A&gt;\"\n",
        "\"<BR><A HREF=\\\"url\\\">\"\n",
    );
    let output = run_example("stdlib_libraries/keys_and_escapes_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_logging_to_the_system_execution() {
    let expected = concat!(
        "false\n",
        "true\n",
        "\"metorex_example\"\n",
        "true\n",
        "true\n",
        "255\n",
        "31\n",
        "false\n",
        "nil\n",
        "128\n",
        "16\n",
        "\"syslog not opened\"\n",
        "true\n",
        "1\n",
        "3\n",
        "true\n",
        "false\n",
        "4\n",
        "true\n",
    );
    let output = run_example("stdlib_libraries/logging_to_the_system.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_logging_to_the_system_parens_execution() {
    let expected = concat!(
        "false\n",
        "true\n",
        "\"metorex_example\"\n",
        "true\n",
        "true\n",
        "255\n",
        "31\n",
        "false\n",
        "nil\n",
        "128\n",
        "16\n",
        "\"syslog not opened\"\n",
        "true\n",
        "1\n",
        "3\n",
        "true\n",
        "false\n",
        "4\n",
        "true\n",
    );
    let output = run_example("stdlib_libraries/logging_to_the_system_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_network_addresses_execution() {
    let expected = concat!(
        "\"127.0.0.1\"\n",
        "80\n",
        "[\"127.0.0.1\", 80]\n",
        "true\n",
        "\"#<Addrinfo: 127.0.0.1:80 TCP>\"\n",
        "\"#<Addrinfo: [::1]:80 TCP>\"\n",
        "\"#<Addrinfo: 127.0.0.1:80 UDP>\"\n",
        "\"#<Addrinfo: 127.0.0.1>\"\n",
        "\"#<Addrinfo: /tmp/held.sock SOCK_STREAM>\"\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "[80, \"127.0.0.1\"]\n",
        "\"127.0.0.1\"\n",
        "\"hello\"\n",
        "\"127.0.0.1\"\n",
        "\"AF_INET\"\n",
        "true\n",
    );
    let output = run_example("stdlib_libraries/network_addresses.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_libraries_network_addresses_parens_execution() {
    let expected = concat!(
        "\"127.0.0.1\"\n",
        "80\n",
        "[\"127.0.0.1\", 80]\n",
        "true\n",
        "\"#<Addrinfo: 127.0.0.1:80 TCP>\"\n",
        "\"#<Addrinfo: [::1]:80 TCP>\"\n",
        "\"#<Addrinfo: 127.0.0.1:80 UDP>\"\n",
        "\"#<Addrinfo: 127.0.0.1>\"\n",
        "\"#<Addrinfo: /tmp/held.sock SOCK_STREAM>\"\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "[80, \"127.0.0.1\"]\n",
        "\"127.0.0.1\"\n",
        "\"hello\"\n",
        "\"127.0.0.1\"\n",
        "\"AF_INET\"\n",
        "true\n",
    );
    let output = run_example("stdlib_libraries/network_addresses_parens.rb");
    assert_eq!(output, expected);
}

/// The expected output of both `stdlib_libraries/http_headers` variants.
const HTTP_HEADERS_OUTPUT: &str = "\"POST\"\n\"/orders\"\n\"text/html; charset=utf-8\"\n\"alpha, beta\"\n[\"alpha\", \"beta\"]\n\"text\"\n\"html\"\n\"text/html\"\n{\"charset\" => \"utf-8\"}\ntrue\n\"fallback\"\n42\n\"bytes=10-200\"\n[10..200]\n0..499\n500\n\"Basic bWFpbnRlbmFuY2U6d2luZG93\"\n[\"cmd=search\", \"max=50\"]\n\"application/x-www-form-urlencoded\"\ntrue\n[\"alpha\", \"beta\"]\nfalse\nfalse\ntrue\n";

#[test]
fn test_stdlib_libraries_http_headers_execution() {
    let output = run_example("stdlib_libraries/http_headers.rb");
    assert_eq!(output, HTTP_HEADERS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_http_headers_no_parens_execution() {
    let output = run_example("stdlib_libraries/http_headers_no_parens.rb");
    assert_eq!(output, HTTP_HEADERS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/http_response` variants.
const HTTP_RESPONSE_OUTPUT: &str = "Net::HTTPOK\n\"200\"\n\"OK\"\n\"1.1\"\n\"text/plain\"\n{\"content-type\" => [\"text/plain\"], \"x-trace\" => [\"alpha\"]}\nNet::HTTPOK\nNet::HTTPError\ntrue\nfalse\n\"#<Net::HTTPOK 200 OK readbody=false>\"\n\"maintenance window moves to 02:00\\n\"\n\"#<Net::HTTPOK 200 OK readbody=true>\"\nnil\nNet::HTTPClientException\n\"404 Not Found\"\ntrue\nNet::HTTPRetriableError\n";

#[test]
fn test_stdlib_libraries_http_response_execution() {
    let output = run_example("stdlib_libraries/http_response.rb");
    assert_eq!(output, HTTP_RESPONSE_OUTPUT);
}

#[test]
fn test_stdlib_libraries_http_response_no_parens_execution() {
    let output = run_example("stdlib_libraries/http_response_no_parens.rb");
    assert_eq!(output, HTTP_RESPONSE_OUTPUT);
}

/// The expected output of both `stdlib_libraries/http_request_exec` variants.
const HTTP_REQUEST_EXEC_OUTPUT: &str = "[\"POST /orders HTTP/1.1\", \"Content-Type: text/plain\", \"Accept: */*\", \"User-Agent: Ruby\", \"Content-Length: 33\", \"\", \"maintenance window moves to 02:00\"]\n[\"PUT /orders HTTP/1.0\", \"Content-Type: text/plain\", \"Transfer-Encoding: chunked\", \"Accept: */*\", \"User-Agent: Ruby\", \"\", \"8\", \"chunk me\", \"0\"]\nArgumentError\ntrue\ntrue\n\"#<Net::HTTP::Trace TRACE>\"\n";

#[test]
fn test_stdlib_libraries_http_request_exec_execution() {
    let output = run_example("stdlib_libraries/http_request_exec.rb");
    assert_eq!(output, HTTP_REQUEST_EXEC_OUTPUT);
}

#[test]
fn test_stdlib_libraries_http_request_exec_no_parens_execution() {
    let output = run_example("stdlib_libraries/http_request_exec_no_parens.rb");
    assert_eq!(output, HTTP_REQUEST_EXEC_OUTPUT);
}

/// The expected output of both `stdlib_libraries/ftp_session` variants.
const FTP_SESSION_OUTPUT: &str = "true\nfalse\nfalse\nfalse\nnil\n60\ntrue\n[false, true, true, true]\nnil\n[true, true, 100]\n[42, 23]\n\"private_data_connection can be set to true only when ssl is enabled\"\ntrue\ntrue\ntrue\ntrue\n";

#[test]
fn test_stdlib_libraries_ftp_session_execution() {
    let output = run_example("stdlib_libraries/ftp_session.rb");
    assert_eq!(output, FTP_SESSION_OUTPUT);
}

#[test]
fn test_stdlib_libraries_ftp_session_no_parens_execution() {
    let output = run_example("stdlib_libraries/ftp_session_no_parens.rb");
    assert_eq!(output, FTP_SESSION_OUTPUT);
}

/// The expected output of both `stdlib_libraries/socket_settings` variants.
const SOCKET_SETTINGS_OUTPUT: &str = "[true, true]\ntrue\ntrue\ntrue\n4\n\"#<Socket::Option: UNSPEC SOCKET LINGER off 0sec>\"\n\"#<Socket::Option: UNSPEC SOCKET LINGER on 30sec>\"\n[true, 30]\nTypeError\n\"unknown socket domain: INET4\"\n[80, \"127.0.0.1\"]\n[80, \"127.0.0.1\"]\n[80, \"0.0.0.0\"]\n16\ntrue\n\"#<Addrinfo: /foo SOCK_DGRAM>\"\ntrue\n\"\"\n[true, true, true]\n";

#[test]
fn test_stdlib_libraries_socket_settings_execution() {
    let output = run_example("stdlib_libraries/socket_settings.rb");
    assert_eq!(output, SOCKET_SETTINGS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_socket_settings_no_parens_execution() {
    let output = run_example("stdlib_libraries/socket_settings_no_parens.rb");
    assert_eq!(output, SOCKET_SETTINGS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/yaml_documents` variants.
const YAML_DOCUMENTS_OUTPUT: &str = "\"str\"\n:locked\n47\n[\"a\", \"b\", \"c\"]\n[\"a\", \"b\", \"c\"]\n{\"a\" => \"b\", \"c\" => 2}\n[[[\"one\", \"two\", \"three\"]]]\n{\"user name\": \"This is the user name.\"}\nnil\n[[\"Mark McGwire\", \"Sammy Sosa\"], [\"Chicago Cubs\"]]\n2\n\"--- :locked\\n\"\n\"--- str\\n\"\n\"--- \\na: b\\n\"\n\"--- \\n- a\\n- b\\n- c\\n\"\n\"--- foo\\n--- 20\\n--- []\\n\\n--- {}\\n\\n\"\n\"--- \\n- a: b\\n- b: c\\n\"\n\"--- !ruby/module 'Enumerable'\\n\"\n\"--- !ruby/exception:StandardError\\nmessage: foobar\\nbacktrace: \\n\"\n\"--- !ruby/range\\nbegin: 1\\nend: 3\\nexcl: false\\n\"\n\"--- .nan\\n\"\n{[\"Detroit Tigers\", \"Chicago Cubs\"] => [\"first\"]}\nPsych::SyntaxError\n";

#[test]
fn test_stdlib_libraries_yaml_documents_execution() {
    let output = run_example("stdlib_libraries/yaml_documents.rb");
    assert_eq!(output, YAML_DOCUMENTS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_yaml_documents_no_parens_execution() {
    let output = run_example("stdlib_libraries/yaml_documents_no_parens.rb");
    assert_eq!(output, YAML_DOCUMENTS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/process_identity` variants.
const PROCESS_IDENTITY_OUTPUT: &str = "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n3\ntrue\n\"metorex-example\"\n0\nErrno::ESRCH\nErrno::EPERM\n";

#[test]
fn test_stdlib_libraries_process_identity_execution() {
    let output = run_example("stdlib_libraries/process_identity.rb");
    assert_eq!(output, PROCESS_IDENTITY_OUTPUT);
}

#[test]
fn test_stdlib_libraries_process_identity_no_parens_execution() {
    let output = run_example("stdlib_libraries/process_identity_no_parens.rb");
    assert_eq!(output, PROCESS_IDENTITY_OUTPUT);
}

#[cfg(target_os = "linux")]
#[test]
fn test_stdlib_libraries_first_thread_execution() {
    let output = run_example("stdlib_libraries/first_thread.rb");
    assert_eq!(output, "1\n");
}

#[cfg(target_os = "linux")]
#[test]
fn test_stdlib_libraries_first_thread_no_parens_execution() {
    let output = run_example("stdlib_libraries/first_thread_no_parens.rb");
    assert_eq!(output, "1\n");
}

/// The expected output of both `stdlib_libraries/build_settings` variants.
const BUILD_SETTINGS_OUTPUT: &str = "true\ntrue\ntrue\ntrue\nnil\n64\n[4, 8]\ntrue\n[-32768, 32767]\n\"127.0.0.1\"\ntrue\nResolv::ResolvError\n{verbose: true, require: \"optparse\"}\n[\"leftover\"]\ntrue\n";

#[test]
fn test_stdlib_libraries_build_settings_execution() {
    let output = run_example("stdlib_libraries/build_settings.rb");
    assert_eq!(output, BUILD_SETTINGS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_build_settings_no_parens_execution() {
    let output = run_example("stdlib_libraries/build_settings_no_parens.rb");
    assert_eq!(output, BUILD_SETTINGS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/log_rotation` variants.
const LOG_ROTATION_OUTPUT: &str = "true\ntrue\ntrue\n";

#[test]
fn test_stdlib_libraries_log_rotation_execution() {
    let output = run_example("stdlib_libraries/log_rotation.rb");
    assert_eq!(output, LOG_ROTATION_OUTPUT);
}

#[test]
fn test_stdlib_libraries_log_rotation_parens_execution() {
    let output = run_example("stdlib_libraries/log_rotation_parens.rb");
    assert_eq!(output, LOG_ROTATION_OUTPUT);
}

/// The expected output of both `stdlib_libraries/time_limits` variants, which show
/// a block given a limit on how long it may run and differ only in whether the calls are
/// written with parentheses.
const TIME_LIMITS_OUTPUT: &str = "42\n[Timeout::Error, \"execution expired\"]\n\"took too long\"\n\"Timeout sec must be a non-negative number\"\n:inside\n0\ntrue\ntrue\n";

#[test]
fn test_stdlib_libraries_time_limits_execution() {
    let output = run_example("stdlib_libraries/time_limits.rb");
    assert_eq!(output, TIME_LIMITS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_time_limits_no_parens_execution() {
    let output = run_example("stdlib_libraries/time_limits_no_parens.rb");
    assert_eq!(output, TIME_LIMITS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/ftp_transfer` variants, which
/// differ only in whether the calls are written with parentheses.
const FTP_TRANSFER_OUTPUT: &str = concat!(
    "[\"one.rb\", \"two.rb\"]\n",
    "[\"first line\", \"second line\"]\n",
    "1998-07-05 13:23:16 UTC\n",
    "nil\n",
);

#[test]
fn test_stdlib_libraries_ftp_transfer_execution() {
    let output = run_example("stdlib_libraries/ftp_transfer.rb");
    assert_eq!(output, FTP_TRANSFER_OUTPUT);
}

#[test]
fn test_stdlib_libraries_ftp_transfer_no_parens_execution() {
    let output = run_example("stdlib_libraries/ftp_transfer_no_parens.rb");
    assert_eq!(output, FTP_TRANSFER_OUTPUT);
}

/// The expected output of both `stdlib_libraries/waiting_on_a_stream` variants,
/// which differ only in whether the calls are written with parentheses.
const WAITING_ON_A_STREAM_OUTPUT: &str = concat!(
    "[\"prompt>\"]\n",
    "[\" helloready>\", \"rea\", \">\"]\n",
    "nil\n",
    "#<Encoding:UTF-8>\n",
    "#<Encoding:UTF-16LE>\n",
);

#[test]
fn test_stdlib_libraries_waiting_on_a_stream_execution() {
    let output = run_example("stdlib_libraries/waiting_on_a_stream.rb");
    assert_eq!(output, WAITING_ON_A_STREAM_OUTPUT);
}

#[test]
fn test_stdlib_libraries_waiting_on_a_stream_no_parens_execution() {
    let output = run_example("stdlib_libraries/waiting_on_a_stream_no_parens.rb");
    assert_eq!(output, WAITING_ON_A_STREAM_OUTPUT);
}

/// The expected output of both `stdlib_libraries/reading_compressed_streams` variants, which differ only in whether the
/// calls are written with parentheses.
const READING_COMPRESSED_STREAMS_OUTPUT: &str = concat!(
    "foo\n",
    "foo-and more\n",
    "[16384, 16384, 16384, 16384, 16384, 16384, 1696]\n",
);

#[test]
fn test_stdlib_libraries_reading_compressed_streams_execution() {
    let output = run_example("stdlib_libraries/reading_compressed_streams.rb");
    assert_eq!(output, READING_COMPRESSED_STREAMS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_reading_compressed_streams_parens_execution() {
    let output = run_example("stdlib_libraries/reading_compressed_streams_parens.rb");
    assert_eq!(output, READING_COMPRESSED_STREAMS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/coverage_counts` variants, which differ only in whether the
/// calls are written with parentheses.
const COVERAGE_COUNTS_OUTPUT: &str = concat!(
    "true\n",
    "false\n",
    "true\n",
    "[1, 1, 1, nil, nil, 1, 0, nil, nil, nil, 1]\n",
    "false\n",
    "[1, 1, 1, nil, nil, 1, 0, nil, nil, nil, 1]\n",
    "{}\n",
    "[[:unused, 6, 8, 0], [:used, 2, 4, 1]]\n",
);

#[test]
fn test_stdlib_libraries_coverage_counts_execution() {
    let output = run_example("stdlib_libraries/coverage_counts.rb");
    assert_eq!(output, COVERAGE_COUNTS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_coverage_counts_parens_execution() {
    let output = run_example("stdlib_libraries/coverage_counts_parens.rb");
    assert_eq!(output, COVERAGE_COUNTS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/open3_streams` variants.
/// The first pipeline's `sort` writes to the program's own output.
const OPEN3_STREAMS_OUTPUT: &str = concat!(
    "\"to the command\"\n",
    "\"err\\n\"\n",
    "true\n",
    "true\n",
    "\"SHOUT\"\n",
    "[\"kept\", \"\", 4]\n",
    "[\"err\", \"out\"]\n",
    "a\n",
    "b\n",
    "[true, true]\n",
    "\"2\"\n",
    "2\n",
    "\"apple\\n\"\n"
);

#[test]
fn test_stdlib_libraries_open3_streams_execution() {
    let output = run_example("stdlib_libraries/open3_streams.rb");
    assert_eq!(output, OPEN3_STREAMS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_open3_streams_parens_execution() {
    let output = run_example("stdlib_libraries/open3_streams_parens.rb");
    assert_eq!(output, OPEN3_STREAMS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/io_wait_forms` variants.
const IO_WAIT_FORMS_OUTPUT: &str = "[1, 2, 4]\nnil\n4\n1\ntrue\ntrue\ntrue\n[ArgumentError, \"Events must be positive integer!\"]\n[ArgumentError, \"Events must be positive integer!\"]\n[ArgumentError, \"unsupported mode: sideways\"]\n[ArgumentError, \"timeout given more than once\"]\n[ArgumentError, \"time interval must not be negative\"]\n[TypeError, \"no implicit conversion from nil to integer\"]\n[IOError, \"closed stream\"]\n[IOError, \"closed stream\"]\n";

#[test]
fn test_stdlib_libraries_io_wait_forms_execution() {
    let output = run_example("stdlib_libraries/io_wait_forms.rb");
    assert_eq!(output, IO_WAIT_FORMS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_io_wait_forms_no_parens_execution() {
    let output = run_example("stdlib_libraries/io_wait_forms_no_parens.rb");
    assert_eq!(output, IO_WAIT_FORMS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/irb_session` variants.
const IRB_SESSION_OUTPUT: &str = "total ** 2\n100\ntotal = 4\n4\ndef twice(number) = number * 2\n:twice\ntwice(total)\n8\n[1,\n  2]\n[1, 2]\nmissing_name\n(irb):7:in '<main>': undefined local variable or method 'missing_name' for main (NameError)\n\nexit\n[:after, 4]\n";

#[test]
fn test_stdlib_libraries_irb_session_execution() {
    let output = run_example("stdlib_libraries/irb_session.rb");
    assert_eq!(output, IRB_SESSION_OUTPUT);
}

#[test]
fn test_stdlib_libraries_irb_session_no_parens_execution() {
    let output = run_example("stdlib_libraries/irb_session_no_parens.rb");
    assert_eq!(output, IRB_SESSION_OUTPUT);
}

/// `stdlib_libraries/irb_source_window` prints its own source, so each
/// variant has an expected output of its own.
const IRB_SOURCE_WINDOW_OUTPUT: &str = "\nFrom: here @ line 16 :\n\n    11:   end\n    12:   written.sub(/From: \\S+/, \"From: here\")\n    13: end\n    14: \n    15: if ARGV.first == \"early\"\n => 16:   binding.irb\n    17: elsif ARGV.first == \"late\"\n    18:   binding.irb\n    19: else\n    20:   puts(header_of(\"early\"))\n    21:   puts(header_of(\"late\"))\n\nSwitch to inspect mode.\n\n\nFrom: here @ line 18 :\n\n    13: end\n    14: \n    15: if ARGV.first == \"early\"\n    16:   binding.irb\n    17: elsif ARGV.first == \"late\"\n => 18:   binding.irb\n    19: else\n    20:   puts(header_of(\"early\"))\n    21:   puts(header_of(\"late\"))\n    22: end\n\nSwitch to inspect mode.\n\n";

const IRB_SOURCE_WINDOW_NO_PARENS_OUTPUT: &str = "\nFrom: here @ line 16 :\n\n    11:   end\n    12:   written.sub(/From: \\S+/, \"From: here\")\n    13: end\n    14: \n    15: if ARGV.first == \"early\"\n => 16:   binding.irb\n    17: elsif ARGV.first == \"late\"\n    18:   binding.irb\n    19: else\n    20:   puts header_of \"early\"\n    21:   puts header_of \"late\"\n\nSwitch to inspect mode.\n\n\nFrom: here @ line 18 :\n\n    13: end\n    14: \n    15: if ARGV.first == \"early\"\n    16:   binding.irb\n    17: elsif ARGV.first == \"late\"\n => 18:   binding.irb\n    19: else\n    20:   puts header_of \"early\"\n    21:   puts header_of \"late\"\n    22: end\n\nSwitch to inspect mode.\n\n";

#[test]
fn test_stdlib_libraries_irb_source_window_execution() {
    let output = run_example("stdlib_libraries/irb_source_window.rb");
    assert_eq!(output, IRB_SOURCE_WINDOW_OUTPUT);
}

#[test]
fn test_stdlib_libraries_irb_source_window_no_parens_execution() {
    let output = run_example("stdlib_libraries/irb_source_window_no_parens.rb");
    assert_eq!(output, IRB_SOURCE_WINDOW_NO_PARENS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/drb_remote_calls` variants.
const DRB_REMOTE_CALLS_OUTPUT: &str = "6\n[3, 5]\nDRb::DRbObject\n[6, 6]\n[ArgumentError, \"bad input\"]\n\"private method 'hidden' called\"\n";

#[test]
fn test_stdlib_libraries_drb_remote_calls_execution() {
    let output = run_example("stdlib_libraries/drb_remote_calls.rb");
    assert_eq!(output, DRB_REMOTE_CALLS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_drb_remote_calls_no_parens_execution() {
    let output = run_example("stdlib_libraries/drb_remote_calls_no_parens.rb");
    assert_eq!(output, DRB_REMOTE_CALLS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/fiddle_handles` variants.
const FIDDLE_HANDLES_OUTPUT: &str = "[Fiddle::DLError, Fiddle::Error, StandardError]\n[true, true, true]\ntrue\n\"unknown symbol \\\"no_such_symbol_here\\\"\"\n0\n\"dlclose() called too many times\"\n\"closed handle\"\n";

#[test]
fn test_stdlib_libraries_fiddle_handles_execution() {
    let output = run_example("stdlib_libraries/fiddle_handles.rb");
    assert_eq!(output, FIDDLE_HANDLES_OUTPUT);
}

#[test]
fn test_stdlib_libraries_fiddle_handles_no_parens_execution() {
    let output = run_example("stdlib_libraries/fiddle_handles_no_parens.rb");
    assert_eq!(output, FIDDLE_HANDLES_OUTPUT);
}

/// The expected output of both `stdlib_libraries/socket_full_buffers` variants.
const SOCKET_FULL_BUFFERS_OUTPUT: &str = "[IO::EAGAINWaitWritable, \"Resource temporarily unavailable - sendmsg(2) would block\"]\n:wait_writable\n3000000\n3000000\n";

#[test]
fn test_stdlib_libraries_socket_full_buffers_execution() {
    let output = run_example("stdlib_libraries/socket_full_buffers.rb");
    assert_eq!(output, SOCKET_FULL_BUFFERS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_socket_full_buffers_no_parens_execution() {
    let output = run_example("stdlib_libraries/socket_full_buffers_no_parens.rb");
    assert_eq!(output, SOCKET_FULL_BUFFERS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/socket_options` variants.
const SOCKET_OPTIONS_OUTPUT: &str = "true\n0\ntrue\n0\n64\n10\n[Errno::EINVAL, \"Invalid argument - setsockopt(2)\"]\n\"no implicit conversion of nil into String\"\n";

#[test]
fn test_stdlib_libraries_socket_options_execution() {
    let output = run_example("stdlib_libraries/socket_options.rb");
    assert_eq!(output, SOCKET_OPTIONS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_socket_options_no_parens_execution() {
    let output = run_example("stdlib_libraries/socket_options_no_parens.rb");
    assert_eq!(output, SOCKET_OPTIONS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/socket_connect_nonblock` variants.
const SOCKET_CONNECT_NONBLOCK_OUTPUT: &str = "[IO::EINPROGRESSWaitWritable, \"Operation now in progress - connect(2) would block\"]\n0\n\"connected\"\n:wait_writable\n\"unbound IPv4 socket\"\n";

#[test]
fn test_stdlib_libraries_socket_connect_nonblock_execution() {
    let output = run_example("stdlib_libraries/socket_connect_nonblock.rb");
    assert_eq!(output, SOCKET_CONNECT_NONBLOCK_OUTPUT);
}

#[test]
fn test_stdlib_libraries_socket_connect_nonblock_no_parens_execution() {
    let output = run_example("stdlib_libraries/socket_connect_nonblock_no_parens.rb");
    assert_eq!(output, SOCKET_CONNECT_NONBLOCK_OUTPUT);
}

/// The expected output of both `stdlib_libraries/socket_recvmsg` variants.
const SOCKET_RECVMSG_OUTPUT: &str = "[\"hello\", \"127.0.0.1\", true, 0]\n\"tr\"\n:wait_readable\n[\"over a connection\", \"#<Addrinfo: empty-sockaddr SOCK_STREAM>\", 0, 0]\nErrno::ENOTCONN\nnil\n";

#[test]
fn test_stdlib_libraries_socket_recvmsg_execution() {
    let output = run_example("stdlib_libraries/socket_recvmsg.rb");
    assert_eq!(output, SOCKET_RECVMSG_OUTPUT);
}

#[test]
fn test_stdlib_libraries_socket_recvmsg_no_parens_execution() {
    let output = run_example("stdlib_libraries/socket_recvmsg_no_parens.rb");
    assert_eq!(output, SOCKET_RECVMSG_OUTPUT);
}

/// The expected output of both `stdlib_libraries/addrinfo_from_arrays` variants.
const ADDRINFO_FROM_ARRAYS_OUTPUT: &str = "#<Addrinfo: 127.0.0.1:46102 (localhost)>\n#<Addrinfo: [::1]:80 (hostname)>\n#<Addrinfo: 127.0.0.1:80 TCP>\n[true, true]\n17\nSocketError\nSocket::ResolutionError\nSocket::ResolutionError\n[\"socket\", true, 0, 0]\n";

#[test]
fn test_stdlib_libraries_addrinfo_from_arrays_execution() {
    let output = run_example("stdlib_libraries/addrinfo_from_arrays.rb");
    assert_eq!(output, ADDRINFO_FROM_ARRAYS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_addrinfo_from_arrays_no_parens_execution() {
    let output = run_example("stdlib_libraries/addrinfo_from_arrays_no_parens.rb");
    assert_eq!(output, ADDRINFO_FROM_ARRAYS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/openssl_scrypt` variants.
const OPENSSL_SCRYPT_OUTPUT: &str = "\"300420e0918b3cc46a24504429ff1f43\"\n\"300420e0\"\ntrue\n\"\"\n[{N: 3}, \"EVP_PBE_scrypt\"]\n[{r: 0}, \"EVP_PBE_scrypt\"]\n[{p: 0}, \"EVP_PBE_scrypt\"]\n\"missing keywords: :N, :r, :p, :length\"\n";

#[test]
fn test_stdlib_libraries_openssl_scrypt_execution() {
    let output = run_example("stdlib_libraries/openssl_scrypt.rb");
    assert_eq!(output, OPENSSL_SCRYPT_OUTPUT);
}

#[test]
fn test_stdlib_libraries_openssl_scrypt_no_parens_execution() {
    let output = run_example("stdlib_libraries/openssl_scrypt_no_parens.rb");
    assert_eq!(output, OPENSSL_SCRYPT_OUTPUT);
}

/// The expected output of both `stdlib_libraries/x509_names` variants.
const X509_NAMES_OUTPUT: &str = "\"/DC=org/DC=ruby-lang/CN=www.ruby-lang.org\"\n[[\"DC\", \"org\", 22], [\"DC\", \"ruby-lang\", 22], [\"CN\", \"www.ruby-lang.org\", 12]]\n\"CN=www.ruby-lang.org,DC=ruby-lang,DC=org\"\n\"DC = org, DC = ruby-lang, CN = www.ruby-lang.org\"\n\"#<OpenSSL::X509::Name CN=www.ruby-lang.org,DC=ruby-lang,DC=org>\"\n[[\"C\", \"US\", 19], [\"O\", \"Acme\", 12], [\"CN\", \"x\", 12]]\n\"/CN=a\\\\+b/O=x,y\"\n\"O=x\\\\,y,CN=a\\\\+b\"\n\"CN = \\\"a+b\\\", O = \\\"x,y\\\"\"\n\"commonName                = a+b\\norganizationName          = x,y\"\ntrue\n-1\n\"3019310b3009060355040613025553310a300806035504030c0161\"\n[TypeError, \"no implicit conversion of nil into String\"]\n[OpenSSL::X509::NameError, \"X509_NAME_add_entry_by_txt: invalid field name (name=hello)\"]\n";

#[test]
fn test_stdlib_libraries_x509_names_execution() {
    let output = run_example("stdlib_libraries/x509_names.rb");
    assert_eq!(output, X509_NAMES_OUTPUT);
}

#[test]
fn test_stdlib_libraries_x509_names_no_parens_execution() {
    let output = run_example("stdlib_libraries/x509_names_no_parens.rb");
    assert_eq!(output, X509_NAMES_OUTPUT);
}

/// The expected output of both `stdlib_libraries/x509_certificates` variants.
const X509_CERTIFICATES_OUTPUT: &str = "[\"sha256WithRSAEncryption\", 1, \"/CN=Example Root\"]\n[\"basicConstraints\", \"keyUsage\", \"subjectKeyIdentifier\"]\n\"keyUsage = critical, Digital Signature\"\n[true, false]\n[false, \"unable to get local issuer certificate\"]\n[true, \"ok\", 2]\n[false, 10, \"certificate has expired\"]\n";

#[test]
fn test_stdlib_libraries_x509_certificates_execution() {
    let output = run_example("stdlib_libraries/x509_certificates.rb");
    assert_eq!(output, X509_CERTIFICATES_OUTPUT);
}

#[test]
fn test_stdlib_libraries_x509_certificates_no_parens_execution() {
    let output = run_example("stdlib_libraries/x509_certificates_no_parens.rb");
    assert_eq!(output, X509_CERTIFICATES_OUTPUT);
}

/// The expected output of both `stdlib_libraries/ripper_events` variants.
const RIPPER_EVENTS_OUTPUT: &str = concat!(
    "[\"def\", \" \", \"area\", \"(\", \"width\", \",\", \" \", \"height\", \" \", \"=\", \" \", \"2\", \")\", \" \", \"=\", \" \", \"width\", \" \", \"*\", \" \", \"height\", \"\\n\", \"puts\", \" \", \"area\", \"(\", \"3\", \")\", \"\\n\"]\n",
    "[[1, 0], :on_ident, \"total\", \"CMDARG\"]\n",
    "[[1, 5], :on_sp, \" \", \"CMDARG\"]\n",
    "[[1, 6], :on_op, \"=\", \"BEG\"]\n",
    "[[1, 7], :on_sp, \" \", \"BEG\"]\n",
    "[[1, 8], :on_ident, \"price\", \"ARG\"]\n",
    "[[1, 13], :on_sp, \" \", \"ARG\"]\n",
    "[[1, 14], :on_op, \"*\", \"BEG\"]\n",
    "[[1, 15], :on_sp, \" \", \"BEG\"]\n",
    "[[1, 16], :on_int, \"2\", \"END\"]\n",
    "[:program, [[:assign, [:var_field, [:@ident, \"total\", [1, 0]]], [:binary, [:vcall, [:@ident, \"price\", [1, 8]]], :*, [:@int, \"2\", [1, 16]]]]]]\n",
    "[:program, [:stmts_add, [:stmts_new], [:array, [:args_add_star, [:args_add, [:args_new], [:@int, \"1\", [1, 1]]], [:vcall, [:@ident, \"rest\", [1, 5]]]]]]]\n",
    "[:program, [[:def, [:@ident, \"area\", [1, 4]], [:paren, [:params, [[:@ident, \"width\", [1, 9]]], [[[:@ident, \"height\", [1, 16]], [:@int, \"2\", [1, 25]]]], nil, nil, nil, nil, nil]], [:bodystmt, [:binary, [:var_ref, [:@ident, \"width\", [1, 30]]], :*, [:var_ref, [:@ident, \"height\", [1, 38]]]], nil, nil, nil]], [:command, [:@ident, \"puts\", [2, 0]], [:args_add_block, [[:method_add_arg, [:fcall, [:@ident, \"area\", [2, 5]]], [:arg_paren, [:args_add_block, [[:@int, \"3\", [2, 10]]], false]]]], false]]]]\n",
    "nil\n",
    "\"BEG|LABEL\"\n",
    "[:program, [[:assign, [:var_field, [:@ident, \"message\", [1, 0]]], [:string_literal, [:string_content, [:@tstring_content, \"Deploy finished\\n\", [2, 4]], [:@tstring_content, \"  on schedule\\n\", [3, 4]]]]]]]\n",
    "[[:on_ident, \"message\"], [:on_sp, \" \"], [:on_op, \"=\"], [:on_sp, \" \"], [:on_heredoc_beg, \"<<~TEXT\"], [:on_nl, \"\\n\"], [:on_ignored_sp, \"    \"], [:on_tstring_content, \"Deploy finished\\n\"], [:on_ignored_sp, \"    \"], [:on_tstring_content, \"  on schedule\\n\"], [:on_heredoc_end, \"TEXT\\n\"]]\n",
    "[:program, [[:case, [:vcall, [:@ident, \"reading\", [1, 5]]], [:in, [:hshptn, nil, [[[:@label, \"status:\", [2, 4]], [:@int, \"200\", [2, 12]]], [[:@label, \"body:\", [2, 17]], nil]], nil], [[:var_ref, [:@ident, \"body\", [2, 29]]]], [:in, [:aryptn, nil, [[:binary, [:var_ref, [:@const, \"Integer\", [3, 4]]], :\"=>\", [:var_field, [:@ident, \"code\", [3, 15]]]]], [:var_field, nil], nil], [[:var_ref, [:@ident, \"code\", [3, 29]]]], nil]]]]]\n",
    "[\"start\", \"stop\"]\n",
    "2\n",
    "\"print\"\n",
    "2\n",
);

#[test]
fn test_stdlib_libraries_ripper_events_execution() {
    let output = run_example("stdlib_libraries/ripper_events.rb");
    assert_eq!(output, RIPPER_EVENTS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_ripper_events_no_parens_execution() {
    let output = run_example("stdlib_libraries/ripper_events_no_parens.rb");
    assert_eq!(output, RIPPER_EVENTS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/ripper_grammar_forms` variants.
const RIPPER_GRAMMAR_FORMS_OUTPUT: &str = concat!(
    "[:program, [[:if, [:vcall, [:@ident, \"a\", [1, 3]]], [[:void_stmt], [:vcall, [:@ident, \"b\", [2, 1]]]], nil]]]\n",
    "[:program, [[:if, [:vcall, [:@ident, \"a\", [1, 3]]], [[:void_stmt], [:vcall, [:@ident, \"b\", [1, 6]]]], nil]]]\n",
    "[:program, [[:if, [:vcall, [:@ident, \"a\", [1, 3]]], [[:void_stmt], [:vcall, [:@ident, \"b\", [2, 3]]]], nil]]]\n",
    "[:program, [[:if, [:vcall, [:@ident, \"a\", [1, 3]]], [[:vcall, [:@ident, \"b\", [3, 0]]]], nil]]]\n",
    "[:program, [[:if, [:vcall, [:@ident, \"a\", [1, 3]]], [[:vcall, [:@ident, \"b\", [2, 0]]]], nil]]]\n",
    "[:program, [[:if, [:vcall, [:@ident, \"a\", [1, 3]]], [[:vcall, [:@ident, \"b\", [2, 5]]]], nil]]]\n",
    "[:program, [[:if, [:vcall, [:@ident, \"a\", [1, 3]]], [[:vcall, [:@ident, \"b\", [1, 11]]]], nil]]]\n",
    "[:program, [[:while, [:vcall, [:@ident, \"a\", [1, 6]]], [[:void_stmt], [:vcall, [:@ident, \"b\", [2, 1]]]]]]]\n",
    "[:program, [[:while, [:vcall, [:@ident, \"a\", [1, 6]]], [[:void_stmt], [:vcall, [:@ident, \"b\", [1, 9]]]]]]]\n",
    "[:program, [[:until, [:vcall, [:@ident, \"a\", [1, 6]]], [[:vcall, [:@ident, \"b\", [3, 0]]]]]]]\n",
    "[:program, [[:for, [:var_field, [:@ident, \"x\", [1, 4]]], [:vcall, [:@ident, \"a\", [1, 9]]], [[:void_stmt], [:vcall, [:@ident, \"b\", [2, 1]]]]]]]\n",
    "[:program, [[:for, [:var_field, [:@ident, \"x\", [1, 4]]], [:vcall, [:@ident, \"a\", [1, 9]]], [[:vcall, [:@ident, \"b\", [1, 11]]]]]]]\n",
    "[:program, [[:unless, [:vcall, [:@ident, \"a\", [1, 7]]], [[:void_stmt], [:vcall, [:@ident, \"b\", [2, 1]]]], nil]]]\n",
    "[:program, [[:if, [:vcall, [:@ident, \"a\", [1, 3]]], [[:void_stmt]], [:elsif, [:vcall, [:@ident, \"c\", [2, 6]]], [[:void_stmt], [:vcall, [:@ident, \"b\", [3, 1]]]], nil]]]]\n",
    "[:program, [[:case, [:vcall, [:@ident, \"a\", [1, 5]]], [:when, [[:@int, \"1\", [2, 5]]], [[:void_stmt], [:vcall, [:@ident, \"b\", [3, 1]]]], nil]]]]\n",
    "[:program, [[:case, [:vcall, [:@ident, \"a\", [1, 5]]], [:when, [[:@int, \"1\", [2, 5]]], [[:void_stmt], [:vcall, [:@ident, \"b\", [2, 9]]]], nil]]]]\n",
    "[:program, [[:begin, [:bodystmt, [[:void_stmt], [:vcall, [:@ident, \"a\", [2, 1]]]], [:rescue, nil, nil, [[:void_stmt], [:vcall, [:@ident, \"b\", [4, 1]]]], nil], nil, nil]]]]\n",
    "[:program, [[:while, [:vcall, [:@ident, \"a\", [1, 6]]], [[:void_stmt], [:vcall, [:@ident, \"b\", [1, 12]]]]]]]\n",
    "[:program, [[:method_add_arg, [:fcall, [:@ident, \"p\", [1, 0]]], [:arg_paren, [[:command_call, [:vcall, [:@ident, \"a\", [1, 2]]], [:@period, \".\", [1, 3]], [:@ident, \"b\", [1, 4]], [:args_add_block, [[:@int, \"1\", [1, 6]]], false]]]]]]]\n",
    "[:program, [[:method_add_arg, [:fcall, [:@ident, \"p\", [1, 0]]], [:arg_paren, [[:command, [:@ident, \"bar\", [1, 2]], [:args_add_block, [[:array, [[:@int, \"2\", [1, 7]]]]], false]]]]]]]\n",
    "[:program, [[:method_add_arg, [:fcall, [:@ident, \"p\", [1, 0]]], [:arg_paren, [[:command, [:@ident, \"gather\", [1, 2]], [:args_add_block, [[:paren, [[:void_stmt]]]], false]]]]]]]\n",
    "[:program, [[:method_add_arg, [:fcall, [:@ident, \"p\", [1, 0]]], [:arg_paren, [[:command_call, [:vcall, [:@ident, \"a\", [1, 2]]], [:@period, \".\", [1, 3]], [:@ident, \"b\", [1, 4]], [:args_add_block, [[:@int, \"1\", [1, 6]], [:@int, \"2\", [1, 9]]], false]]]]]]]\n",
    "[:program, [[:method_add_arg, [:fcall, [:@ident, \"p\", [1, 0]]], [:arg_paren, [[:command, [:@ident, \"foo\", [1, 2]], [:args_add_block, [[:@int, \"1\", [1, 6]]], false]]]]]]]\n",
    "[:program, [[:method_add_arg, [:fcall, [:@ident, \"p\", [1, 0]]], [:arg_paren, [[:command, [:@ident, \"foo\", [1, 2]], [:args_add_block, [[:@int, \"1\", [1, 6]], [:@int, \"2\", [1, 9]]], false]]]]]]]\n",
    "nil\n",
    "nil\n",
    "[:program, [[:method_add_block, [:call, [:method_add_block, [:method_add_arg, [:fcall, [:@ident, \"foo\", [1, 0]]], []], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]], [:@period, \".\", [2, 3]], [:@ident, \"bar\", [2, 4]]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]]]]\n",
    "[:program, [[:method_add_block, [:command_call, [:method_add_block, [:method_add_arg, [:fcall, [:@ident, \"foo\", [1, 0]]], []], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]], [:@period, \".\", [1, 10]], [:@ident, \"bar\", [1, 11]], [:args_add_block, [[:@int, \"1\", [1, 15]]], false]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]]]]\n",
    "nil\n",
    "nil\n",
    "[:program, [[:method_add_arg, [:fcall, [:@ident, \"foo\", [1, 0]]], [:arg_paren, [[:command_call, [:vcall, [:@ident, \"a\", [1, 4]]], [:@period, \".\", [1, 5]], [:@ident, \"b\", [1, 6]], [:args_add_block, [[:@int, \"1\", [1, 8]]], false]]]]]]]\n",
    "[:program, [[:method_add_arg, [:fcall, [:@ident, \"p\", [1, 0]]], [:arg_paren, [[:command, [:@ident, \"foo\", [1, 2]], [:args_add_block, [[:bare_assoc_hash, [[:assoc_new, [:@label, \"a:\", [1, 6]], [:@int, \"1\", [1, 9]]]]]], false]]]]]]]\n",
    "nil\n",
    "[:program, [[:method_add_arg, [:fcall, [:@ident, \"p\", [1, 0]]], [:arg_paren, [:args_add_block, [[:paren, [[:method_add_block, [:command, [:@ident, \"foo\", [1, 3]], [:args_add_block, [[:@int, \"1\", [1, 7]]], false]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]]]]], false]]]]]\n",
    "[:program, [[:method_add_arg, [:fcall, [:@ident, \"p\", [1, 0]]], [:arg_paren, [[:command, [:@ident, \"foo\", [1, 2]], [:args_add_block, [[:method_add_block, [:method_add_arg, [:fcall, [:@ident, \"bar\", [1, 6]]], []], [:brace_block, nil, [[:method_add_block, [:command, [:@ident, \"baz\", [1, 12]], [:args_add_block, [[:@int, \"1\", [1, 16]]], false]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]]]]]], false]]]]]]]\n",
    "[:program, [[:method_add_block, [:command, [:@ident, \"foo\", [1, 0]], [:args_add_block, [[:@int, \"1\", [1, 4]]], false]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]]]]\n",
    "[:program, [[:method_add_block, [:command_call, [:method_add_block, [:command, [:@ident, \"foo\", [1, 0]], [:args_add_block, [[:@int, \"1\", [1, 4]]], false]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]], [:@period, \".\", [1, 12]], [:@ident, \"bar\", [1, 13]], nil], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]]]]\n",
    "[:program, [[:call, [:method_add_block, [:command, [:@ident, \"foo\", [1, 0]], [:args_add_block, [[:@int, \"1\", [1, 4]]], false]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]], [:@period, \".\", [1, 12]], [:@ident, \"bar\", [1, 13]]]]]\n",
    "[:program, [[:method_add_arg, [:call, [:method_add_block, [:command, [:@ident, \"foo\", [1, 0]], [:args_add_block, [[:@int, \"1\", [1, 4]]], false]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]], [:@period, \".\", [1, 12]], [:@ident, \"bar\", [1, 13]]], [:args_add_block, [[:@int, \"2\", [1, 17]]], false]]]]\n",
    "[:program, [[:method_add_block, [:command_call, [:method_add_block, [:command, [:@ident, \"foo\", [1, 0]], [:args_add_block, [[:@int, \"1\", [1, 4]]], false]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]], [:@period, \".\", [1, 12]], [:@ident, \"bar\", [1, 13]], nil], [:brace_block, nil, [[:void_stmt]]]]]]\n",
    "[:program, [[:method_add_block, [:command_call, [:method_add_block, [:command, [:@ident, \"foo\", [1, 0]], [:args_add_block, [[:@int, \"1\", [1, 4]]], false]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]], [:@period, \".\", [1, 12]], [:@ident, \"bar\", [1, 13]], [:arg_paren, [:args_add_block, [[:@int, \"2\", [1, 17]]], false]]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]]]]\n",
    "[:program, [[:def, [:@ident, \"m\", [1, 4]], [:params, nil, nil, nil, nil, nil, nil, nil], [:bodystmt, [[:sclass, [:var_ref, [:@kw, \"self\", [1, 16]]], [:bodystmt, [[:return, [:args_add_block, [[:@int, \"1\", [1, 29]]], false]]], nil, nil, nil]]], nil, nil, nil]]]]\n",
    "nil\n",
    "[:program, [[:def, [:@ident, \"m\", [1, 4]], [:params, nil, nil, nil, nil, nil, nil, nil], [:bodystmt, [[:method_add_block, [:method_add_arg, [:fcall, [:@ident, \"foo\", [1, 7]]], []], [:brace_block, nil, [[:sclass, [:var_ref, [:@kw, \"self\", [1, 22]]], [:bodystmt, [[:return, [:args_add_block, [[:@int, \"1\", [1, 35]]], false]]], nil, nil, nil]]]]]], nil, nil, nil]]]]\n",
    "[:program, [[:method_add_block, [:command_call, [:method_add_block, [:command_call, [:var_ref, [:@const, \"TracePoint\", [1, 0]]], [:@period, \".\", [1, 10]], [:@ident, \"new\", [1, 11]], [:args_add_block, [[:symbol_literal, [:symbol, [:@ident, \"call\", [1, 16]]]]], false]], [:do_block, [:block_var, [:params, [[:@ident, \"tp\", [1, 25]]], nil, nil, nil, nil, nil, nil], false], [:bodystmt, [[:void_stmt]], nil, nil, nil]]], [:@period, \".\", [2, 3]], [:@ident, \"enable\", [2, 4]], nil], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]]]]\n",
    "[:program, [[:call, [:method_add_block, [:command_call, [:method_add_block, [:command_call, [:vcall, [:@ident, \"a\", [1, 0]]], [:@period, \".\", [1, 1]], [:@ident, \"b\", [1, 2]], [:args_add_block, [[:@int, \"1\", [1, 4]]], false]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]], [:@period, \".\", [1, 12]], [:@ident, \"c\", [1, 13]], nil], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]], [:@period, \".\", [1, 21]], [:@ident, \"d\", [1, 22]]]]]\n",
    "[:program, [[:method_add_arg, [:call, [:method_add_block, [:command, [:@ident, \"foo\", [1, 0]], [:args_add_block, [[:@int, \"1\", [1, 4]]], false]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]], [:@period, \".\", [1, 12]], [:@ident, \"bar\", [1, 13]]], [:arg_paren, [:args_add_block, [[:@int, \"2\", [1, 17]]], false]]]]]\n",
    "[:program, [[:call, [:call, [:method_add_block, [:command, [:@ident, \"foo\", [1, 0]], [:args_add_block, [[:@int, \"1\", [1, 4]]], false]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]], [:@period, \".\", [1, 12]], [:@ident, \"bar\", [1, 13]]], [:@period, \".\", [1, 16]], [:@ident, \"baz\", [1, 17]]]]]\n",
);

#[test]
fn test_stdlib_libraries_ripper_grammar_forms_execution() {
    let output = run_example("stdlib_libraries/ripper_grammar_forms.rb");
    assert_eq!(output, RIPPER_GRAMMAR_FORMS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_ripper_grammar_forms_no_parens_execution() {
    let output = run_example("stdlib_libraries/ripper_grammar_forms_no_parens.rb");
    assert_eq!(output, RIPPER_GRAMMAR_FORMS_OUTPUT);
}

const FORWARDING: &str = concat!(
    "2\n",
    "3\n",
    "5\n",
    "2\n",
    "3\n",
    "\"5\"\n",
    "[10, 18]\n",
    "false\n",
    "Line\n",
    "true\n",
    "true\n",
    "-1\n",
    ":count\n",
    "[:min, :max]\n",
    "tests/_examples/stdlib_libraries/forwarding.rb:49: warning: Secretive#hidden at tests/_examples/stdlib_libraries/forwarding.rb:47 forwarding to private method Secretive::Inner#hidden\n",
    ":reached\n",
    "[\"a\"]\n",
    "through stdout\n",
    "\"1.4.0\"\n",
    "nil\n",
);

const FORWARDING_NO_PARENS: &str = concat!(
    "2\n",
    "3\n",
    "5\n",
    "2\n",
    "3\n",
    "\"5\"\n",
    "[10, 18]\n",
    "false\n",
    "Line\n",
    "true\n",
    "true\n",
    "-1\n",
    ":count\n",
    "[:min, :max]\n",
    "tests/_examples/stdlib_libraries/forwarding_no_parens.rb:49: warning: Secretive#hidden at tests/_examples/stdlib_libraries/forwarding_no_parens.rb:47 forwarding to private method Secretive::Inner#hidden\n",
    ":reached\n",
    "[\"a\"]\n",
    "through stdout\n",
    "\"1.4.0\"\n",
    "nil\n",
);

#[test]
fn test_stdlib_libraries_forwarding_execution() {
    let output = run_example("stdlib_libraries/forwarding.rb");
    assert_eq!(output, FORWARDING);
}

#[test]
fn test_stdlib_libraries_forwarding_no_parens_execution() {
    let output = run_example("stdlib_libraries/forwarding_no_parens.rb");
    assert_eq!(output, FORWARDING_NO_PARENS);
}
