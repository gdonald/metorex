// Examples covering the libraries metorex carries

use super::run_example;

/// The expected output of both `stdlib_libraries/shipped_libraries` variants,
/// which differ only in whether the calls are written with parentheses.
const SHIPPED_LIBRARIES_OUTPUT: &str = "\"Tm93IGlzIHRoZSB0aW1lIGZvciBhbGwgZ29vZCBjb2RlcnMKdG8gbGVhcm4g\\nUnVieQ==\\n\"\n\"aGVsbG8=\"\n\"hello\"\n\"hello\"\n\"aGVsbG8_\"\n\"aGVsbG8_\"\n\"hello?\"\n[\"ruby\", \"-e\", \"puts 1\", \"--name\", \"value\"]\n\"a\\\\ b\"\n\"ruby a\\\\ b\"\n[\"one\", \"two three\"]\n{\"r\" => \"ruby\", \"ru\" => \"ruby\", \"rub\" => \"ruby\", \"ruby\" => \"ruby\"}\n{\"ca\" => \"car\", \"car\" => \"car\", \"co\" => \"cone\", \"con\" => \"cone\", \"cone\" => \"cone\"}\ntrue\n0\nNoMethodError\nTypeError\n0\n1\n:moved\n0\n32\n8\n\"\"\n36\nInteger\n7\n7\n\"e\"\n1\n\"xam\"\n4\nfalse\nnil\nnil\nfalse\ntrue\ntrue\nfalse\ntrue\n\"first\\n\"\n1\n[\"second\\n\"]\n0\n[\"first\\n\", \"second\\n\"]\n[\"a\", \"b\", \"c\"]\n[97, 98, 99]\n\"a\"\n97\n\"one two!\\nthree\"\n14\n13\n0\n2\n0\n\"exam\"\nIOError\n\"Ada\"\n1843\n\"computing\"\n\"Ada\"\n\"London\"\n{name: \"Ada\", year: 1843, field: \"computing\", city: \"London\"}\n\"#<OpenStruct name=\\\"Ada\\\", year=1843, field=\\\"computing\\\", city=\\\"London\\\">\"\ntrue\nfalse\nnil\ntrue\n\"#<OpenStruct>\"\ntrue\nfalse\n[2, 3, 5, 7, 11]\n[[2, 3], [3, 2], [5, 1]]\n72\n[[-1, 1], [2, 1], [5, 1]]\ntrue\n[[2, 2], [3, 1]]\n20\n2\n3\n2\n";

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
        "\"0.33333333333333333333e0\"\n",
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
        "[\"0.2e1\", \"0.1e1\"]\n",
        "\"0.1024e4\"\n",
        "2\n",
        "3\n",
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
        "\"0.33333333333333333333e0\"\n",
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
        "[\"0.2e1\", \"0.1e1\"]\n",
        "\"0.1024e4\"\n",
        "2\n",
        "3\n",
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
const YAML_DOCUMENTS_OUTPUT: &str = "\"str\"\n:locked\n47\n[\"a\", \"b\", \"c\"]\n[\"a\", \"b\", \"c\"]\n{\"a\" => \"b\", \"c\" => 2}\n[[[\"one\", \"two\", \"three\"]]]\n{:\"user name\" => \"This is the user name.\"}\nnil\n[[\"Mark McGwire\", \"Sammy Sosa\"], [\"Chicago Cubs\"]]\n2\n\"--- :locked\\n\"\n\"--- str\\n\"\n\"--- \\na: b\\n\"\n\"--- \\n- a\\n- b\\n- c\\n\"\n\"--- foo\\n--- 20\\n--- []\\n\\n--- {}\\n\\n\"\n\"--- \\n- a: b\\n- b: c\\n\"\n\"--- !ruby/module 'Enumerable'\\n\"\n\"--- !ruby/exception:StandardError\\nmessage: foobar\\nbacktrace: \\n\"\n\"--- !ruby/range\\nbegin: 1\\nend: 3\\nexcl: false\\n\"\n\"--- .nan\\n\"\n{[\"Detroit Tigers\", \"Chicago Cubs\"] => [\"first\"]}\nPsych::SyntaxError\n";

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
const PROCESS_IDENTITY_OUTPUT: &str = "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n3\ntrue\n\"metorex-example\"\nNotImplementedError\nErrno::ESRCH\nErrno::EPERM\n";

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
