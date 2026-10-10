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
const HTTP_RESPONSE_OUTPUT: &str = "Net::HTTPOK\n\"200\"\n\"OK\"\n\"1.1\"\n\"text/plain\"\n{\"content-type\" => [\"text/plain\"], \"x-trace\" => [\"alpha\"]}\nNet::HTTPOK\nNet::HTTPError\ntrue\nfalse\n\"#<Net::HTTPOK 200 OK readbody=false>\"\n\"maintenance window moves to 02:00\\n\"\n\"#<Net::HTTPOK 200 OK readbody=true>\"\nnil\nNet::HTTPClientException\n\"404 \\\"Not Found\\\"\"\ntrue\nNet::HTTPRetriableError\n";

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
const HTTP_REQUEST_EXEC_OUTPUT: &str = "[\"POST /orders HTTP/1.1\", \"Content-Type: text/plain\", \"Accept-Encoding: gzip;q=1.0,deflate;q=0.6,identity;q=0.3\", \"Accept: */*\", \"User-Agent: Ruby\", \"Content-Length: 33\", \"\", \"maintenance window moves to 02:00\"]\n[\"PUT /orders HTTP/1.0\", \"Content-Type: text/plain\", \"Transfer-Encoding: chunked\", \"Accept-Encoding: gzip;q=1.0,deflate;q=0.6,identity;q=0.3\", \"Accept: */*\", \"User-Agent: Ruby\", \"\", \"8\", \"chunk me\", \"0\"]\nArgumentError\ntrue\ntrue\n\"#<Net::HTTP::Trace TRACE>\"\n";

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
const BUILD_SETTINGS_OUTPUT: &str = "true\ntrue\ntrue\ntrue\nnil\n64\n[4, 8]\ntrue\n[-32768, 32767]\n\"::1\"\ntrue\n[\"10.0.0.5\"]\nResolv::ResolvError\n{verbose: true, require: \"optparse\"}\n[\"leftover\"]\ntrue\n";

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

const TOPOLOGICAL_SORT: &str = concat!(
    "[\"fetch\", \"build\", \"test\", \"deploy\"]\n",
    "[[\"fetch\"], [\"build\"], [\"test\"], [\"deploy\"]]\n",
    "[\"fetch\"]\n",
    "[\"build\"]\n",
    "[\"test\"]\n",
    "[\"deploy\"]\n",
    "[\"fetch\", \"build\", \"test\", \"deploy\"]\n",
    "[[\"fetch\"], [\"build\"], [\"test\"]]\n",
    "[[2, 3], [1], [4]]\n",
    "\"topological sort failed: [2, 3]\"\n",
    "StandardError\n",
    "[4, 2, 3, 1]\n",
    "[[4], [2], [3], [1]]\n",
    "Enumerator\n",
    "[4]\n",
    "[2]\n",
    "[3]\n",
    "[1]\n",
    "\"0.2.0\"\n",
    "NotImplementedError\n",
);

#[test]
fn test_stdlib_libraries_topological_sort_execution() {
    let output = run_example("stdlib_libraries/topological_sort.rb");
    assert_eq!(output, TOPOLOGICAL_SORT);
}

#[test]
fn test_stdlib_libraries_topological_sort_no_parens_execution() {
    let output = run_example("stdlib_libraries/topological_sort_no_parens.rb");
    assert_eq!(output, TOPOLOGICAL_SORT);
}

const IO_WAIT_LIBRARY: &str = concat!(
    "TrueClass\n",
    "false\n",
    "nil\n",
    "nil\n",
    "nil\n",
    "true\n",
    "true\n",
    "1\n",
    "true\n",
    "4\n",
    "true\n",
    "true\n",
    "true\n",
    "true\n",
    "true\n",
    "2\n",
    "nil\n",
    "true\n",
    "\"time interval must not be negative\"\n",
    "nil\n",
    "nil\n",
    "true\n",
    "true\n",
    "true\n",
);

#[test]
fn test_stdlib_libraries_io_wait_library_execution() {
    let output = run_example("stdlib_libraries/io_wait_library.rb");
    assert_eq!(output, IO_WAIT_LIBRARY);
}

#[test]
fn test_stdlib_libraries_io_wait_library_no_parens_execution() {
    let output = run_example("stdlib_libraries/io_wait_library_no_parens.rb");
    assert_eq!(output, IO_WAIT_LIBRARY);
}

const PSEUDO_TERMINALS: &str = concat!(
    "[:check, :getpty, :open, :spawn]\n",
    "RuntimeError\n",
    "[File, File, Integer]\n",
    "false\n",
    "\"hello\\r\\n\"\n",
    "3\n",
    "nil\n",
    "[IO, File]\n",
    "true\n",
    "true\n",
    "true\n",
    "\"typed\\r\\n\"\n",
    "\"back\\n\"\n",
    "[IO, File]\n",
    ":answer\n",
    "nil\n",
    "Process::Status\n",
    "true\n",
    "\"block form\\r\\n\"\n",
    "\"hi\\r\\n\"\n",
    "7\n",
    "true\n",
);

#[test]
fn test_stdlib_libraries_pseudo_terminals_execution() {
    let output = run_example("stdlib_libraries/pseudo_terminals.rb");
    assert_eq!(output, PSEUDO_TERMINALS);
}

#[test]
fn test_stdlib_libraries_pseudo_terminals_no_parens_execution() {
    let output = run_example("stdlib_libraries/pseudo_terminals_no_parens.rb");
    assert_eq!(output, PSEUDO_TERMINALS);
}

const CONTINUATIONS: &str = concat!(
    "true\n",
    "3\n",
    "10\n",
    "nil\n",
    "[1, 2]\n",
    "5\n",
    "[Continuation, true]\n",
    "4\n",
    "nil\n",
    "200\n",
    "[:[], :call]\n",
    "NoMethodError\n",
    "true\n",
);

#[test]
fn test_stdlib_libraries_continuations_execution() {
    let output = run_example("stdlib_libraries/continuations.rb");
    assert_eq!(output, CONTINUATIONS);
}

#[test]
fn test_stdlib_libraries_continuations_no_parens_execution() {
    let output = run_example("stdlib_libraries/continuations_no_parens.rb");
    assert_eq!(output, CONTINUATIONS);
}

/// The expected output of both `stdlib_libraries/continuation_reentry`
/// variants.
const CONTINUATION_REENTRY: &str = concat!(
    "0\n",
    "1\n",
    "2\n",
    "in method 10\n",
    "in method 11\n",
    "in method 12\n",
    ":done\n",
);

#[test]
fn test_stdlib_libraries_continuation_reentry_execution() {
    let output = run_example("stdlib_libraries/continuation_reentry.rb");
    assert_eq!(output, CONTINUATION_REENTRY);
}

#[test]
fn test_stdlib_libraries_continuation_reentry_no_parens_execution() {
    let output = run_example("stdlib_libraries/continuation_reentry_no_parens.rb");
    assert_eq!(output, CONTINUATION_REENTRY);
}

/// Run `code` with `-e`, answering what it wrote to stderr.
fn stderr_of(code: &str) -> String {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_metorex"))
        .arg("-e")
        .arg(code)
        .output()
        .expect("failed to run metorex");
    String::from_utf8(output.stderr).expect("stderr was not utf8")
}

#[test]
fn test_continuation_warning_names_the_library_file() {
    let written = stderr_of("require 'continuation'");
    assert_eq!(
        written,
        "<metorex>/continuation.rb: warning: callcc is obsolete; use Fiber instead\n"
    );
}

const RIPEMD_DIGESTS: &str = concat!(
    "\"9c1185a5c5e9fc54612808977ee8f548b2258d31\"\n",
    "\"8eb208f7e05d987a9b044a8e98c6b087f15a0bfc\"\n",
    "\"aa69deee9a8922e92f8105e007f76110f381e9cf\"\n",
    "\"XQaJ70nS+uVyuIGxI6hf+iFZXzY=\"\n",
    "\"8eb208f7e05d987a9b044a8e98c6b087f15a0bfc\"\n",
    "20\n",
    "64\n",
    "\"8eb208f7e05d987a9b044a8e98c6b087f15a0bfc\"\n",
    "20\n",
    "#<Encoding:BINARY (ASCII-8BIT)>\n",
    "\"8eb208f7e05d987a9b044a8e98c6b087f15a0bfc\"\n",
    "Digest::Base\n",
);

#[test]
fn test_stdlib_libraries_ripemd_digests_execution() {
    let output = run_example("stdlib_libraries/ripemd_digests.rb");
    assert_eq!(output, RIPEMD_DIGESTS);
}

#[test]
fn test_stdlib_libraries_ripemd_digests_no_parens_execution() {
    let output = run_example("stdlib_libraries/ripemd_digests_no_parens.rb");
    assert_eq!(output, RIPEMD_DIGESTS);
}

const PERSISTENT_STORES: &str = concat!(
    "true\n",
    ":answered\n",
    "true\n",
    "[[1, 2], [:list, \"name\"], [:list, \"name\"], true, false, \"x\", 5]\n",
    "[PStore::Error, \"undefined key 'missing'\"]\n",
    "nil\n",
    "[1, 2]\n",
    "nil\n",
    "[1, 2, 4]\n",
    "[1, 2, 4]\n",
    "nil\n",
    "[\"name\"]\n",
    "[PStore::Error, \"not in transaction\"]\n",
    "[PStore::Error, \"in read-only transaction\"]\n",
    "[PStore::Error, \"nested transaction\"]\n",
    "[PStore::Error, \"in read-only transaction\"]\n",
    "StandardError\n",
    "false\n",
    "{\"name\" => \"x\"}\n",
    "false\n",
    "\"x\"\n",
    "[:[], :[]=, :abort, :commit, :delete, :fetch, :key?, :keys, :path, :root?, :roots, :transaction, :ultra_safe, :ultra_safe=]\n",
    "[]\n",
    "false\n",
);

#[test]
fn test_stdlib_libraries_persistent_stores_execution() {
    let output = run_example("stdlib_libraries/persistent_stores.rb");
    assert_eq!(output, PERSISTENT_STORES);
}

#[test]
fn test_stdlib_libraries_persistent_stores_no_parens_execution() {
    let output = run_example("stdlib_libraries/persistent_stores_no_parens.rb");
    assert_eq!(output, PERSISTENT_STORES);
}

const BENCHMARKS: &str = concat!(
    "1.9375\n",
    "\"fixed\"\n",
    "[\"fixed\", 1.5, 0.25, 0.125, 0.0625, 2.0]\n",
    "{label: \"fixed\", utime: 1.5, stime: 0.25, cutime: 0.125, cstime: 0.0625, real: 2.0}\n",
    "  N   N   N (  N)\n",
    "fixed:  1.50| 0.25| 0.12| 0.06| 1.94|( 2.00) extra\n",
    "[\"\", 2.5, 1.25, 1.125, 1.0625, 3.0]\n",
    "4.0\n",
    "0.75\n",
    "0.0\n",
    "\"\"\n",
    "\"      user     system      total        real\\n\"\n",
    "\"%10.6u %10.6y %10.6t %10.6r\\n\"\n",
    "\"0.5.0\"\n",
    "Benchmark::Tms\n",
    "\"work\"\n",
    "Float\n",
    "true\n",
    "Float\n",
    "Float\n",
    "\"\"\n",
    "              user     system      total        real\n",
    "first:    N   N   N (  N)\n",
    "second:   N   N   N (  N)\n",
    ">total:   N   N   N (  N)\n",
    "2\n",
    "[\"first:\", \"second:\"]\n",
    "Rehearsal ------------------------------------------------\n",
    "one            N   N   N (  N)\n",
    "longer label   N   N   N (  N)\n",
    "--------------------------------------- total: Nsec\n",
    "\n",
    "                   user     system      total        real\n",
    "one            N   N   N (  N)\n",
    "longer label   N   N   N (  N)\n",
    "[\"one\", \"longer label\"]\n",
    "Benchmark::Tms\n",
    "true\n",
    "true\n",
    "\"no block\"\n",
    "[:benchmark, :bm, :bmbm, :measure, :ms, :realtime]\n",
);

#[test]
fn test_stdlib_libraries_benchmarks_execution() {
    let output = run_example("stdlib_libraries/benchmarks.rb");
    assert_eq!(output, BENCHMARKS);
}

#[test]
fn test_stdlib_libraries_benchmarks_no_parens_execution() {
    let output = run_example("stdlib_libraries/benchmarks_no_parens.rb");
    assert_eq!(output, BENCHMARKS);
}

const FILE_UTILITIES: &str = concat!(
    "mkdir a\n",
    "mkdir => [\"a\"]\n",
    "mkdir -m 750 b c\n",
    "mkdir mode => [\"b\", \"c\"]\n",
    "mkdir -p a/b/c/\n",
    "mkdir_p => [\"a/b/c/\"]\n",
    "mkdir -p x/y z\n",
    "mkdir_p noop => [\"x/y\", \"z\"]\n",
    "mkdir exists !! Errno::EEXIST: File exists @ dir_s_mkdir - a\n",
    "touch t1 t2\n",
    "touch => [\"t1\", \"t2\"]\n",
    "touch -c missing\n",
    "touch nocreate !! Errno::ENOENT: No such file or directory @ apply2files - missing\n",
    "touch -t 200109090146.40 t1\n",
    "touch mtime => [\"t1\"]\n",
    "1000000000\n",
    "cp a/one.txt copy.txt\n",
    "cp => nil\n",
    "cp a/one.txt t1 c\n",
    "cp into dir => [\"a/one.txt\", \"t1\"]\n",
    "cp same !! ArgumentError: same file: copy.txt and copy.txt\n",
    "cp -r a a2\n",
    "cp_r => nil\n",
    "cp -rp a c\n",
    "cp_r preserve => 1\n",
    "cp -lr a a3\n",
    "cp_lr => nil\n",
    "ln copy.txt hard.txt\n",
    "ln => 0\n",
    "ln -f t2 hard.txt\n",
    "ln force => 0\n",
    "ln -s copy.txt soft.txt\n",
    "ln_s => 0\n",
    "ln_s exists !! Errno::EEXIST: File exists @ syserr_fail2_in - soft.txt\n",
    "ln -sf t1 soft.txt\n",
    "ln_sf => 0\n",
    "ln -s ../a/b/two.txt c/two.txt\n",
    "ln_sr => 0\n",
    "ln -s ../../a/one.txt a2/b/rel.txt\n",
    "ln_s relative => 0\n",
    "mv t2 moved\n",
    "mv => 0\n",
    "mv moved copy.txt b\n",
    "mv into dir => [\"moved\", \"copy.txt\"]\n",
    "mv onto dir => 0\n",
    "mv -f nope nowhere\n",
    "mv missing force => nil\n",
    "chmod 600 t1\n",
    "chmod => [\"t1\"]\n",
    "chmod u+x,go=r b/moved\n",
    "chmod symbolic => [\"b/moved\"]\n",
    "chmod a-x,a+X a2\n",
    "chmod X => [\"a2\"]\n",
    "chmod bad !! ArgumentError: invalid 'who' symbol in file mode: q\n",
    "chmod -R g+w a2\n",
    "chmod_R => [\"a2\"]\n",
    "install -c -m 640 t1 inst/deep/t1\n",
    "install => nil\n",
    "install -c -p -m u+x t1 inst/deep/t1\n",
    "install again => nil\n",
    "install -c t1 inst2/\n",
    "install dir => nil\n",
    "compare_file => [true, false, true]\n",
    "uptodate? => [true, false, false]\n",
    "copy_entry => nil\n",
    "copy_file => nil\n",
    "copy_stream => 3\n",
    "chown : t1\n",
    "chown self => [\"t1\"]\n",
    "a/ 755\n",
    "a/b/ 755\n",
    "a/b/c/ 755\n",
    "a/b/two.txt 664 \"two\" links=2\n",
    "a/one.txt 664 \"one\" links=2\n",
    "a2/ 775\n",
    "a2/a3/ 775\n",
    "a2/a3/b/ 775\n",
    "a2/a3/b/c/ 775\n",
    "a2/a3/b/two.txt 664 \"two\" links=2\n",
    "a2/a3/one.txt 664 \"one\" links=2\n",
    "a2/b/ 775\n",
    "a2/b/c/ 775\n",
    "a2/b/rel.txt -> ../../a/one.txt\n",
    "a2/b/two.txt 664 \"two\" links=1\n",
    "a2/one.txt 664 \"one\" links=1\n",
    "b/ 750\n",
    "b/copy.txt 644 \"one\" links=1\n",
    "b/moved 744 \"\" links=2\n",
    "c/ 750\n",
    "c/a/ 755\n",
    "c/a/b/ 755\n",
    "c/a/b/c/ 755\n",
    "c/a/b/two.txt 644 \"two\" links=1\n",
    "c/a/one.txt 644 \"one\" links=1\n",
    "c/one.txt 644 \"one\" links=1\n",
    "c/t1 644 \"\" links=1\n",
    "c/two.txt -> ../a/b/two.txt\n",
    "entry/ 755\n",
    "entry/b/ 755\n",
    "entry/b/c/ 755\n",
    "entry/b/two.txt 644 \"two\" links=1\n",
    "entry/one.txt 644 \"one\" links=1\n",
    "file_copy 644 \"one\" links=1\n",
    "hard.txt 744 \"\" links=2\n",
    "inst/ 755\n",
    "inst/deep/ 755\n",
    "inst/deep/t1 640 \"\" links=1\n",
    "inst2/ 755\n",
    "inst2/t1 600 \"\" links=1\n",
    "soft.txt -> t1\n",
    "streamed 644 \"one\" links=1\n",
    "t1 600 \"\" links=1\n",
    "rm streamed file_copy\n",
    "rm => [\"streamed\", \"file_copy\"]\n",
    "rm missing !! Errno::ENOENT: No such file or directory @ apply2files - nope\n",
    "rm -f nope t1\n",
    "rm_f => [\"nope\", \"t1\"]\n",
    "rm -r entry\n",
    "rm_r => [\"entry\"]\n",
    "rm -rf a3 a2 none\n",
    "rm_rf => [\"a3\", \"a2\", \"none\"]\n",
    "rmdir c\n",
    "rmdir !! Errno::ENOTEMPTY: Directory not empty @ dir_s_rmdir - c\n",
    "rmdir -p p/q/r\n",
    "rmdir parents => [\"p/q/r\"]\n",
    "remove_dir => 0\n",
    "remove_dir file !! Errno::ENOTDIR: Not a directory - soft.txt\n",
    "remove_entry_secure => nil\n",
    "remove_file => 1\n",
    "cd a\n",
    "cd -\n",
    "cd => true\n",
    "a/ 755\n",
    "a/b/ 755\n",
    "a/b/c/ 755\n",
    "a/b/two.txt 664 \"two\" links=1\n",
    "a/one.txt 664 \"one\" links=1\n",
    "b/ 750\n",
    "b/copy.txt 644 \"one\" links=1\n",
    "b/moved 744 \"\" links=1\n",
    "c/ 750\n",
    "c/a/ 755\n",
    "c/a/b/ 755\n",
    "c/a/b/c/ 755\n",
    "c/a/b/two.txt 644 \"two\" links=1\n",
    "c/a/one.txt 644 \"one\" links=1\n",
    "c/one.txt 644 \"one\" links=1\n",
    "c/t1 644 \"\" links=1\n",
    "c/two.txt -> ../a/b/two.txt\n",
    "soft.txt -> t1\n",
    "touch v1\n",
    "Verbose => [\"v1\"]\n",
    "NoWrite => nil\n",
    "rm -rf a\n",
    "DryRun => nil\n",
    "cp v1 v2\n",
    "DryRun cp => nil\n",
    "true\n",
    "false\n",
    "true\n",
    "NoWrite pwd => nil\n",
    "step: touch labelled\n",
    "label => [\"labelled\"]\n",
    "included => [\"inc2/one\", \"inc2/one/file\"]\n",
    "private => false\n",
);

#[test]
fn test_stdlib_libraries_file_utilities_execution() {
    let output = run_example("stdlib_libraries/file_utilities.rb");
    assert_eq!(output, FILE_UTILITIES);
}

#[test]
fn test_stdlib_libraries_file_utilities_no_parens_execution() {
    let output = run_example("stdlib_libraries/file_utilities_no_parens.rb");
    assert_eq!(output, FILE_UTILITIES);
}

const RUN_COMMANDS: &str = concat!(
    "$ mkdir -- -v a\n",
    "mkdir a\n",
    "exit 0\n",
    "$ mkdir -- -p -v deep/er/est\n",
    "mkdir -p deep/er/est\n",
    "exit 0\n",
    "$ mkdir -- a\n",
    "File exists @ dir_s_mkdir - a (Errno::EEXIST)\n",
    "exit 1\n",
    "$ touch -- -v t1 t2\n",
    "touch t1 t2\n",
    "exit 0\n",
    "$ cp -- -v one.txt copy.txt\n",
    "cp one.txt copy.txt\n",
    "exit 0\n",
    "$ cp -- -rv deep a\n",
    "cp -r deep a\n",
    "exit 0\n",
    "$ cp -- -p one.txt kept.txt\n",
    "\n",
    "exit 0\n",
    "$ cp -- -l -v deep linked\n",
    "cp -lr deep linked\n",
    "exit 0\n",
    "$ ln -- -v one.txt hard.txt\n",
    "ln one.txt hard.txt\n",
    "exit 0\n",
    "$ ln -- -s -v one.txt soft.txt\n",
    "ln -s one.txt soft.txt\n",
    "exit 0\n",
    "$ ln -- -sf -v t1 soft.txt\n",
    "ln -sf t1 soft.txt\n",
    "exit 0\n",
    "$ mv -- -v t2 moved.txt\n",
    "mv t2 moved.txt\n",
    "exit 0\n",
    "$ mv -- one.txt t1 a\n",
    "\n",
    "exit 0\n",
    "$ chmod -- -v 600 copy.txt\n",
    "chmod 600 copy.txt\n",
    "exit 0\n",
    "$ chmod -- -v u+x,go-r kept.txt\n",
    "chmod u+x,go-r kept.txt\n",
    "exit 0\n",
    "$ install -- -v -m 640 copy.txt inst/copy.txt\n",
    "install -c -m 640 copy.txt inst/copy.txt\n",
    "exit 0\n",
    "$ install -- -p -v copy.txt inst2/\n",
    "install -c -p copy.txt inst2/\n",
    "exit 0\n",
    "$ rm -- -v hard.txt\n",
    "rm hard.txt\n",
    "exit 0\n",
    "$ rm -- nope\n",
    "No such file or directory @ apply2files - nope (Errno::ENOENT)\n",
    "exit 1\n",
    "$ rm -- -f -v nope\n",
    "rm -f nope\n",
    "exit 0\n",
    "$ rm -- -r -v linked\n",
    "rm -r linked\n",
    "exit 0\n",
    "$ rmdir -- -p -v deep/er/est\n",
    "rmdir -p deep/er/est\n",
    "exit 0\n",
    "$ rm -- -v glob*\n",
    "rm globa globb\n",
    "exit 0\n",
    "$ wait_writable -- -v -n 1 moved.txt\n",
    "\n",
    "exit 0\n",
    "$ wait_writable -- missing\n",
    "\n",
    "exit 0\n",
    "$ help cp mv\n",
    "Copy SOURCE to DEST, or multiple SOURCE(s) to DIRECTORY\n",
    "\n",
    "  ruby -run -e cp -- [OPTION] SOURCE DEST\n",
    "\n",
    "  -p          preserve file attributes if possible\n",
    "  -r          copy recursively\n",
    "  -l          make hard link instead of copying (implies -r)\n",
    "  -v          verbose\n",
    "\n",
    "\n",
    "Rename SOURCE to DEST, or move SOURCE(s) to DIRECTORY.\n",
    "\n",
    "  ruby -run -e mv -- [OPTION] SOURCE DEST\n",
    "\n",
    "  -v          verbose\n",
    "\n",
    "\n",
    "exit 0\n",
    "$ help nonesuch\n",
    "\n",
    "exit 0\n",
    "$ cp -- --help\n",
    "\n",
    "exit 0\n",
    "$ httpd\n",
    "webrick is not found. You may need to `gem install webrick` to install webrick.\n",
    "exit 1\n",
    "a/ 755\n",
    "a/deep/ 755\n",
    "a/deep/er/ 755\n",
    "a/deep/er/est/ 755\n",
    "a/one.txt 644 \"one\\n\"\n",
    "a/t1 644 \"\"\n",
    "copy.txt 600 \"one\\n\"\n",
    "inst/ 755\n",
    "inst/copy.txt 640 \"one\\n\"\n",
    "inst2/ 755\n",
    "inst2/copy.txt 600 \"one\\n\"\n",
    "kept.txt 700 \"one\\n\"\n",
    "moved.txt 644 \"\"\n",
    "soft.txt -> t1\n",
);

#[test]
fn test_stdlib_libraries_run_commands_execution() {
    let output = run_example("stdlib_libraries/run_commands.rb");
    assert_eq!(output, RUN_COMMANDS);
}

#[test]
fn test_stdlib_libraries_run_commands_no_parens_execution() {
    let output = run_example("stdlib_libraries/run_commands_no_parens.rb");
    assert_eq!(output, RUN_COMMANDS);
}

const OPTION_PARSER_ORDERS: &str = concat!(
    "[:p, true]\n",
    "[:m, \"644\"]\n",
    "[:n, \"5\"]\n",
    "[:p, true]\n",
    "[:r, true]\n",
    "[:r, true]\n",
    "[:n, \"7\"]\n",
    "[:port, \"80\"]\n",
    "[:port, \"81\"]\n",
    "[:color, false]\n",
    "[\"x\", \"y\"]\n",
    "[]\n",
    "[:p, true]\n",
    "[\"x\", \"-p\"]\n",
    "[:p, true]\n",
    "[\"x\", \"y\", \"-m\", \"z\"]\n",
    "[:p, true]\n",
    "[\"a\", \"b\"]\n",
    "[:p, true]\n",
    "[]\n",
    "[\"-m\"]\n",
    "[OptionParser::InvalidOption, \"invalid option: -z\"]\n",
    "[OptionParser::MissingArgument, \"missing argument: -n\"]\n",
    "[OptionParser::InvalidOption, \"invalid option: --nope\"]\n",
);

#[test]
fn test_stdlib_libraries_option_parser_orders_execution() {
    let output = run_example("stdlib_libraries/option_parser_orders.rb");
    assert_eq!(output, OPTION_PARSER_ORDERS);
}

#[test]
fn test_stdlib_libraries_option_parser_orders_no_parens_execution() {
    let output = run_example("stdlib_libraries/option_parser_orders_no_parens.rb");
    assert_eq!(output, OPTION_PARSER_ORDERS);
}

const CONFIGURE_CHECKS: &str = concat!(
    "checking for stdio.h... yes\n",
    "true\n",
    "checking for no_such_header.h... no\n",
    "false\n",
    "checking for sqrt() in -lm... yes\n",
    "true\n",
    "checking for nothing() in -lno_such_library_xyz... no\n",
    "false\n",
    "checking for printf() in stdio.h... yes\n",
    "true\n",
    "checking for no_such_function_xyz()... no\n",
    "false\n",
    "checking for strlen() in stdio.h,string.h... yes\n",
    "true\n",
    "checking for errno in errno.h... yes\n",
    "true\n",
    "checking for no_such_variable_xyz in stdio.h... no\n",
    "false\n",
    "checking for size_t in stddef.h... yes\n",
    "true\n",
    "checking for struct no_such_type in stdio.h... no\n",
    "false\n",
    "checking for EOF in stdio.h... yes\n",
    "true\n",
    "checking for NO_SUCH_MACRO_XYZ in stdio.h... no\n",
    "false\n",
    "checking for EOF in stdio.h... yes\n",
    "true\n",
    "checking for NO_SUCH_CONST_XYZ in stdio.h... no\n",
    "false\n",
    "[nil, nil]\n",
    "[\"/def/inc\", \"/def/lib\"]\n",
    "[\"-DHAVE_STDIO_H\", \"-DHAVE_PRINTF\", \"-DHAVE_STRLEN\", \"-DHAVE_ERRNO\", \"-DHAVE_TYPE_SIZE_T\", \"-DHAVE_CONST_EOF\"]\n",
    "\"-lm \"\n",
    "[\"-I/def/inc\"]\n",
    "[\"/def/lib\"]\n",
    "\"MY_LIB_H\"\n",
    "\"foo()\"\n",
    "\"foo\"\n",
    "creating Makefile\n",
    "[\"-DHAVE_CONST_EOF\", \"-DHAVE_ERRNO\", \"-DHAVE_PRINTF\", \"-DHAVE_STDIO_H\", \"-DHAVE_STRLEN\", \"-DHAVE_TYPE_SIZE_T\", \"-lm\"]\n",
    "checking for stdio.h... yes\n",
    "checking for no_such.h... no\n",
    "checking for sqrt() in -lm... yes\n",
    "checking for printf() in stdio.h... yes\n",
    "checking for size_t in stddef.h... yes\n",
    "checking for EOF in stdio.h... yes\n",
    "creating Makefile\n",
    "[\"-DHAVE_CONST_EOF\", \"-DHAVE_PRINTF\", \"-DHAVE_STDIO_H\", \"-DHAVE_TYPE_SIZE_T\", \"-lm\"]\n",
);

#[test]
fn test_stdlib_libraries_configure_checks_execution() {
    let output = run_example("stdlib_libraries/configure_checks.rb");
    assert_eq!(output, CONFIGURE_CHECKS);
}

#[test]
fn test_stdlib_libraries_configure_checks_no_parens_execution() {
    let output = run_example("stdlib_libraries/configure_checks_no_parens.rb");
    assert_eq!(output, CONFIGURE_CHECKS);
}

const COLORIZED_CODE: &str = concat!(
    "\"\\e[34m\\e[1m# A comment\\e[0m\\n\"\n",
    "\"\\e[36mrequire\\e[0m \\e[31m\\e[1m\\\"\\e[0m\\e[31mset\\e[0m\\e[31m\\e[1m\\\"\\e[0m\\n\"\n",
    "\"\\e[32mmodule\\e[0m \\e[34m\\e[1m\\e[4mShipping\\e[0m\\n\"\n",
    "\"  \\e[32mclass\\e[0m \\e[34m\\e[1m\\e[4mCrate\\e[0m < \\e[34m\\e[1m\\e[4mStruct\\e[0m.\\e[36mnew\\e[0m(\\e[33m:\\e[0m\\e[33mwidth\\e[0m, \\e[33m:\\e[0m\\e[33mheight\\e[0m)\\n\"\n",
    "\"    \\e[34m\\e[1m\\e[4mRATE\\e[0m = \\e[35m\\e[1m1.5\\e[0m\\n\"\n",
    "\"    \\e[32mdef\\e[0m \\e[36m\\e[1mvolume\\e[0m(depth = \\e[34m\\e[1m2\\e[0m) = \\e[36mwidth\\e[0m * \\e[36mheight\\e[0m * depth\\n\"\n",
    "\"    \\e[32mdef\\e[0m \\e[36m\\e[1mlabel\\e[0m\\n\"\n",
    "\"      \\e[31m\\e[1m\\\"\\e[0m\\e[31m\\#{\\e[0m\\e[36mwidth\\e[0m\\e[31m}\\e[0m\\e[31mx\\e[0m\\e[31m\\#{\\e[0m\\e[36mheight\\e[0m\\e[31m}\\e[0m\\e[31m \\e[0m\\e[31m#\\e[0m@name\\e[31m $0 \\e[0m\\e[31m#\\e[0m\\e[32m\\e[1m$1\\e[0m\\e[31m\\e[1m\\\"\\e[0m\\n\"\n",
    "\"    \\e[32mend\\e[0m\\n\"\n",
    "\"    \\e[32malias\\e[0m \\e[36m\\e[1msize\\e[0m \\e[36m\\e[1mvolume\\e[0m\\n\"\n",
    "\"    \\e[32mundef\\e[0m \\e[33mlabel\\e[0m\\n\"\n",
    "\"  \\e[32mend\\e[0m\\n\"\n",
    "\"\\e[32mend\\e[0m\\n\"\n",
    "\"crate = \\e[34m\\e[1m\\e[4mShipping\\e[0m::\\e[34m\\e[1m\\e[4mCrate\\e[0m.\\e[36mnew\\e[0m \\e[34m\\e[1m3\\e[0m, \\e[34m\\e[1m4r\\e[0m\\n\"\n",
    "\"\\e[36mp\\e[0m crate.\\e[36mvolume\\e[0m, \\e[33m:\\e[0m\\e[33msym\\e[0m, \\e[33m:\\\"\\e[0m\\e[33mquoted \\e[0m\\e[33m\\#{\\e[0m\\e[34m\\e[1m1\\e[0m\\e[33m}\\e[0m\\e[33m\\\"\\e[0m, \\e[33m%i[\\e[0m\\e[33ma\\e[0m \\e[33mb\\e[0m\\e[33m]\\e[0m, \\e[31m\\e[1m%w[\\e[0m\\e[31mc\\e[0m \\e[31md\\e[0m\\e[31m\\e[1m]\\e[0m, {\\e[35mkey:\\e[0m \\e[34m\\e[1m1\\e[0m, \\e[35m\\\"str\\\":\\e[0m \\e[34m\\e[1m2\\e[0m}\\n\"\n",
    "\"\\e[36mp\\e[0m \\e[36m\\e[1mnil\\e[0m, \\e[36m\\e[1mtrue\\e[0m, \\e[36m\\e[1mfalse\\e[0m, \\e[36m\\e[1mself\\e[0m, \\e[36m\\e[1m__FILE__\\e[0m, \\e[36m\\e[1m__LINE__\\e[0m, \\e[34m\\e[1m?a\\e[0m, \\e[34m\\e[1m2i\\e[0m, \\e[34m\\e[1m0x1f\\e[0m, \\e[35m\\e[1m1e3\\e[0m, \\e[32m\\e[1m$stdout\\e[0m, \\e[32m\\e[1m$~\\e[0m\\n\"\n",
    "\"\\e[36mputs\\e[0m \\e[31m\\e[1m`\\e[0m\\e[31mecho hi\\e[0m\\e[31m\\e[1m`\\e[0m \\e[32mif\\e[0m \\e[32mdefined?\\e[0m(crate) && !\\e[36m\\e[1mfalse\\e[0m \\e[32mor\\e[0m \\e[32mnot\\e[0m \\e[36m\\e[1mtrue\\e[0m\\n\"\n",
    "\"x = \\e[31m\\e[1m/\\e[0m\\e[31mab+c\\e[0m\\e[31m\\e[1m/i\\e[0m =~ \\e[31m\\e[1m\\\"\\e[0m\\e[31mabbc\\e[0m\\e[31m\\e[1m\\\"\\e[0m\\n\"\n",
    "\"value = [\\e[34m\\e[1m1\\e[0m, \\e[34m\\e[1m2\\e[0m].\\e[36mmap\\e[0m { it ** \\e[34m\\e[1m2\\e[0m }.\\e[36msum\\e[0m \\e[32mrescue\\e[0m \\e[34m\\e[1m0\\e[0m\\n\"\n",
    "\"first, crate.width = \\e[34m\\e[1m1\\e[0m, \\e[34m\\e[1m2\\e[0m\\n\"\n",
    "\"\\e[32mcase\\e[0m value \\e[32mwhen\\e[0m \\e[34m\\e[1m5\\e[0m \\e[32mthen\\e[0m \\e[36mp\\e[0m \\e[34m\\e[1m1\\e[0m \\e[32melse\\e[0m \\e[36mp\\e[0m \\e[34m\\e[1m2\\e[0m \\e[32mend\\e[0m\\n\"\n",
    "\"y = \\e[31m<<~TEXT\\e[0m\\n\"\n",
    "\"\\e[31m  heredoc \\e[0m\\e[31m\\#{\\e[0mvalue\\e[31m}\\e[0m\\e[31m\\e[0m\\n\"\n",
    "\"\\e[31mTEXT\\e[0m\\n\"\n",
    "\"\\e[34m\\e[1m=begin\\e[0m\\n\"\n",
    "\"\\e[34m\\e[1mdoc\\e[0m\\n\"\n",
    "\"\\e[34m\\e[1m=end\\e[0m\\n\"\n",
    "\"\\e[32m__END__\\e[0m\\n\"\n",
    "\"trailing\\n\"\n",
    "true\n",
    "\"width + \\e[36mdepth\\e[0m\\n\"\n",
    "\"def (\\n  x^A\"\n",
    "\"\u{feff}\\e[34m\\e[1m# marked\\e[0m\\n\"\n",
    "\"\\e[31m\\e[1mtext\\e[0m\"\n",
    "\"text\"\n",
    "\"\\e[0m\"\n",
    "\"\"\n",
    "true\n",
    "false\n",
    "false\n",
    "p :piped\n",
);

#[test]
fn test_stdlib_libraries_colorized_code_execution() {
    let output = run_example("stdlib_libraries/colorized_code.rb");
    assert_eq!(output, COLORIZED_CODE);
}

#[test]
fn test_stdlib_libraries_colorized_code_no_parens_execution() {
    let output = run_example("stdlib_libraries/colorized_code_no_parens.rb");
    assert_eq!(output, COLORIZED_CODE);
}

const PRISM_PARSING_OUTPUT: &str = concat!(
    "\"1.9.0\"\n",
    "true\n",
    "[:def_node, :call_node]\n",
    ":total\n",
    "[:items]\n",
    "[1, 54]\n",
    "\"total\"\n",
    "[:sum, :*, :price, :puts, :total]\n",
    "[:sum]\n",
    "[[:IDENTIFIER, \"rate\"], [:EQUAL, \"=\"], [:INTEGER, \"7\"], [:EOF, \"\"]]\n",
    "[[:def_params_term, 9], [:unexpected_token_close_context, 9], [:def_term, 0]]\n",
    "true\n",
    "true\n",
    "[\"# shipping\", \"# per pound\"]\n",
    "4\n",
    "[10, 11]\n",
    "[:fee, :rate]\n",
    "32\n",
    "2\n",
    "104\n",
    "[:program, [[:binary, [:vcall, [:@ident, \"rate\", [1, 0]]], :+, [:@int, \"1\", [1, 7]]]]]\n",
    "true\n",
    "true\n",
);

#[test]
fn test_stdlib_prism_parsing_execution() {
    let output = run_example("stdlib/prism_parsing.rb");
    assert_eq!(output, PRISM_PARSING_OUTPUT);
}

#[test]
fn test_stdlib_prism_parsing_no_parens_execution() {
    let output = run_example("stdlib/prism_parsing_no_parens.rb");
    assert_eq!(output, PRISM_PARSING_OUTPUT);
}

const BUNDLED_GEMS_OUTPUT: &str = concat!(
    "[[:posted], false]\n",
    "true\n",
    "#<Encoding:Shift_JIS>\n",
    "\"日本\"\n",
    "[147, 250, 150, 123]\n",
    "\"日本\"\n",
    "\"2.1.5 (2018-12-15)\"\n",
    "9\n",
    "\"1.8.1\"\n",
    "\"127.0.0.1\"\n",
    "2\n",
    "[:invoice, 1043, 15]\n",
    "[[:invoice, 1042, 99]]\n",
    "7647\n",
    "[198, 252, 203, 220]\n",
    "#<Encoding:Shift_JIS>\n",
);

#[test]
fn test_stdlib_bundled_gems_execution() {
    let output = run_example("stdlib/bundled_gems.rb");
    assert_eq!(output, BUNDLED_GEMS_OUTPUT);
}

#[test]
fn test_stdlib_bundled_gems_no_parens_execution() {
    let output = run_example("stdlib/bundled_gems_no_parens.rb");
    assert_eq!(output, BUNDLED_GEMS_OUTPUT);
}

const IRB_TERMINAL_OUTPUT: &str = concat!(
    "irb(main):001> irb(main):001> rate = 7\n",
    "=> 7\n",
    "irb(main):002> irb(main):002> rate * 2\n",
    "=> 14\n",
    "irb(main):003> irb(main):003> [1, 2].first\n",
    "=> 1\n",
    "irb(main):004> irb(main):004> exit\n",
    "0\n",
);

#[test]
fn test_stdlib_libraries_irb_terminal_execution() {
    let output = run_example("stdlib_libraries/irb_terminal.rb");
    assert_eq!(output, IRB_TERMINAL_OUTPUT);
}

#[test]
fn test_stdlib_libraries_irb_terminal_no_parens_execution() {
    let output = run_example("stdlib_libraries/irb_terminal_no_parens.rb");
    assert_eq!(output, IRB_TERMINAL_OUTPUT);
}

/// The expected output of both `stdlib_libraries/ripper_brace_after_command`
/// variants.
const RIPPER_BRACE_AFTER_COMMAND_OUTPUT: &str = concat!(
    "foo 1 { }      refused\n",
    "p(foo 1 { })   refused\n",
    "a.b 1 { }      refused\n",
    "super 1 { }    refused\n",
    "foo(1) { }     read\n",
    "foo (1) { }    read\n",
    "foo a { }      read\n",
    "a.b c { }      read\n",
    "super(1) { }   read\n",
);

#[test]
fn test_stdlib_libraries_ripper_brace_after_command_execution() {
    let output = run_example("stdlib_libraries/ripper_brace_after_command.rb");
    assert_eq!(output, RIPPER_BRACE_AFTER_COMMAND_OUTPUT);
}

#[test]
fn test_stdlib_libraries_ripper_brace_after_command_no_parens_execution() {
    let output = run_example("stdlib_libraries/ripper_brace_after_command_no_parens.rb");
    assert_eq!(output, RIPPER_BRACE_AFTER_COMMAND_OUTPUT);
}

/// The expected output of both `stdlib_libraries/ripper_error_events` variants.
const RIPPER_ERROR_EVENTS_OUTPUT: &str = concat!(
    "\"class A; return; end\"\n",
    "  parse_error: Invalid return in class/module body\n",
    "  | Invalid return in class/module body\n",
    "  | class A; return; end\n",
    "  |          ^~~~~~\n",
    "\"def f; class A; end; end\"\n",
    "  parse_error: class definition in method body\n",
    "  | class definition in method body\n",
    "  | def f; class A; end; end\n",
    "  |        ^~~~~~~\n",
    "\"def f; module M; end; end\"\n",
    "  parse_error: module definition in method body\n",
    "  | module definition in method body\n",
    "  | def f; module M; end; end\n",
    "  |        ^~~~~~~~\n",
    "\"def f; A = 1; end\"\n",
    "  assign_error: dynamic constant assignment\n",
    "  | dynamic constant assignment\n",
    "  | def f; A = 1; end\n",
    "  |        ^\n",
    "\"alias $a $1\"\n",
    "  alias_error: can't make alias for the number variables\n",
    "  | can't make alias for the number variables\n",
    "  | alias $a $1\n",
    "  |          ^~\n",
    "\"def f(a, a); end\"\n",
    "  parse_error: duplicated argument name\n",
    "  | duplicated argument name\n",
    "  | def f(a, a); end\n",
    "  |           ^\n",
    "\"foo { |a, a| }\"\n",
    "  parse_error: duplicated argument name\n",
    "  | duplicated argument name\n",
    "  | foo { |a, a| }\n",
    "  |            ^\n",
    "\"->(a, a) {}\"\n",
    "  parse_error: duplicated argument name\n",
    "  | duplicated argument name\n",
    "  | ->(a, a) {}\n",
    "  |        ^\n",
    "\"def f(a:, a:); end\"\n",
    "  parse_error: duplicated argument name\n",
    "  | duplicated argument name\n",
    "  | def f(a:, a:); end\n",
    "  |             ^\n",
    "\"case 1; in [x, x]; end\"\n",
    "  parse_error: duplicated variable name\n",
    "  | duplicated variable name\n",
    "  | case 1; in [x, x]; end\n",
    "  |                ^\n",
    "\"case a; in x | 1; end\"\n",
    "  parse_error: alternative pattern after variable capture\n",
    "  | alternative pattern after variable capture\n",
    "  | case a; in x | 1; end\n",
    "  |              ^\n",
    "\"case a; in 1 | x; end\"\n",
    "  parse_error: variable capture in alternative pattern\n",
    "  | variable capture in alternative pattern\n",
    "  | case a; in 1 | x; end\n",
    "  |                ^\n",
    "\"[1].each { _1; it }\"\n",
    "  compile_error: 'it' is not allowed when a numbered parameter is already used\n",
    "(ripper):1: numbered parameter is already used here\n",
    "  | 'it' is not allowed when a numbered parameter is already used\n",
    "  | (none):1: numbered parameter is already used here\n",
    "  | [1].each { _1; it }\n",
    "  |            ^~\n",
    "\"[1].each { it; _1 }\"\n",
    "  compile_error: numbered parameters are not allowed when 'it' is already used\n",
    "(ripper):1: 'it' is already used here\n",
    "  | numbered parameters are not allowed when 'it' is already used\n",
    "  | (none):1: 'it' is already used here\n",
    "  | [1].each { it; _1 }\n",
    "  |            ^~\n",
    "\"[1].each { _1; [2].each { _1 } }\"\n",
    "  compile_error: numbered parameter is already used in outer block\n",
    "(ripper):1: numbered parameter is already used here\n",
    "  | numbered parameter is already used in outer block\n",
    "  | (none):1: numbered parameter is already used here\n",
    "  | [1].each { _1; [2].each { _1 } }\n",
    "  |            ^~\n",
    "\"_1 = 1\"\n",
    "  compile_error: _1 is reserved for numbered parameter\n",
    "  | _1 is reserved for numbered parameter\n",
    "\"self = 1\"\n",
    "  assign_error: Can't change the value of self\n",
    "  | Can't change the value of self\n",
    "  | self = 1\n",
    "  | ^~~~\n",
    "\"$1 = 1\"\n",
    "  assign_error: Can't set variable $1\n",
    "  | Can't set variable $1\n",
    "\"class foo; end\"\n",
    "  class_name_error: class/module name must be CONSTANT\n",
    "  | class/module name must be CONSTANT\n",
    "  | class foo; end\n",
    "  |       ^~~\n",
    "\"x = return\"\n",
    "  parse_error: void value expression\n",
    "  | void value expression\n",
    "  | x = return\n",
    "  |     ^~~~~~\n",
    "\"next next\"\n",
    "  parse_error: void value expression\n",
    "  parse_error: Invalid next\n",
    "  parse_error: Invalid next\n",
    "  | void value expression\n",
    "  | next next\n",
    "  |      ^~~~\n",
    "  | Invalid next\n",
    "  | next next\n",
    "  |      ^~~~\n",
    "  | Invalid next\n",
    "  | next next\n",
    "  | ^~~~~~~~~\n",
    "\"def foo=() = 1\"\n",
    "  parse_error: setter method cannot be defined in an endless method definition\n",
    "  | setter method cannot be defined in an endless method definition\n",
    "  | def foo=() = 1\n",
    "  | ^~~~~~~~\n",
    "\"def f; BEGIN {}; end\"\n",
    "  parse_error: BEGIN is permitted only at toplevel\n",
    "  | BEGIN is permitted only at toplevel\n",
    "  | def f; BEGIN {}; end\n",
    "  |        ^~~~~\n",
    "\"x = 1; x = *\"\n",
    "  compile_error: no anonymous rest parameter\n",
    "  parse_error: syntax error, unexpected end-of-input, expecting ','\n",
    "  | no anonymous rest parameter\n",
    "  | syntax error, unexpected end-of-input, expecting ','\n",
    "  | x = 1; x = *\n",
    "  |             ^\n",
    "\"\\\"abc\"\n",
    "  compile_error: unterminated string meets end of file\n",
    "  | unterminated string meets end of file\n",
    "\"0o8\"\n",
    "  parse_error: Invalid octal digit\n",
    "  | Invalid octal digit\n",
);

#[test]
fn test_stdlib_libraries_ripper_error_events_execution() {
    let output = run_example("stdlib_libraries/ripper_error_events.rb");
    assert_eq!(output, RIPPER_ERROR_EVENTS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_ripper_error_events_no_parens_execution() {
    let output = run_example("stdlib_libraries/ripper_error_events_no_parens.rb");
    assert_eq!(output, RIPPER_ERROR_EVENTS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/scanner_errors` variants.
const SCANNER_ERRORS_OUTPUT: &str = concat!(
    "\"\\\"\\\\xg\\\"\"\n",
    "  parse_error: invalid hex escape\n",
    "  | invalid hex escape\n",
    "  | \"\\xg\"\n",
    "  |  ^~\n",
    "\"\\\"\\\\u{zz}\\\"\"\n",
    "  parse_error: invalid Unicode escape\n",
    "  parse_error: unterminated Unicode escape\n",
    "  | invalid Unicode escape\n",
    "  | \"\\u{zz}\"\n",
    "  |     ^\n",
    "  | unterminated Unicode escape\n",
    "  | \"\\u{zz}\"\n",
    "  |     ^\n",
    "\"\\\"\\\\u{61\\\"\"\n",
    "  parse_error: unterminated Unicode escape\n",
    "  | unterminated Unicode escape\n",
    "  | \"\\u{61\"\n",
    "  |       ^\n",
    "\"\\\"\\\\u12\\\"\"\n",
    "  parse_error: invalid Unicode escape\n",
    "  | invalid Unicode escape\n",
    "  | \"\\u12\"\n",
    "  |  ^~~~\n",
    "\"x = 0x + 1\"\n",
    "  parse_error: numeric literal without digits\n",
    "  | numeric literal without digits\n",
    "  | x = 0x + 1\n",
    "  |     ^~\n",
    "\"0xg\"\n",
    "  parse_error: numeric literal without digits\n",
    "  parse_error: syntax error, unexpected local variable or method, expecting end-of-input\n",
    "  | numeric literal without digits\n",
    "  | syntax error, unexpected local variable or method, expecting end-of-input\n",
    "\"0b2\"\n",
    "  parse_error: numeric literal without digits\n",
    "  parse_error: syntax error, unexpected integer literal, expecting end-of-input\n",
    "  | numeric literal without digits\n",
    "  | syntax error, unexpected integer literal, expecting end-of-input\n",
    "\"@1\"\n",
    "  compile_error: '@1' is not allowed as an instance variable name\n",
    "  parse_error: syntax error, unexpected integer literal, expecting end-of-input\n",
    "  | '@1' is not allowed as an instance variable name\n",
    "  | syntax error, unexpected integer literal, expecting end-of-input\n",
    "\"@@1a\"\n",
    "  compile_error: '@@1' is not allowed as a class variable name\n",
    "  parse_error: syntax error, unexpected integer literal, expecting end-of-input\n",
    "  | '@@1' is not allowed as a class variable name\n",
    "  | syntax error, unexpected integer literal, expecting end-of-input\n",
    "\"$-\"\n",
    "  parse_error: syntax error, unexpected invalid token\n",
    "  | syntax error, unexpected invalid token\n",
    "\"?\\\\xg\"\n",
    "  parse_error: invalid hex escape\n",
    "  parse_error: syntax error, unexpected local variable or method, expecting end-of-input\n",
    "  | invalid hex escape\n",
    "  | syntax error, unexpected local variable or method, expecting end-of-input\n",
    "\"%(\\\\xg)\"\n",
    "  parse_error: invalid hex escape\n",
    "  | invalid hex escape\n",
    "  | %(\\xg)\n",
    "  |   ^~\n",
    "\"/\\\\xg/\"\n",
    "  parse_error: invalid hex escape\n",
    "  | invalid hex escape\n",
    "  | /\\xg/\n",
    "  |  ^~\n",
    "\"'\\\\xg'\"\n",
    "[[:on_tstring_beg, \"\\\"\"], [:on_tstring_content, \"a\\\\u{\"], [:on_tstring_content, \"zz}b\"], [:on_tstring_end, \"\\\"\"]]\n",
    "[[:on_ivar, \"@\"], [:on_int, \"1\"]]\n",
);

#[test]
fn test_stdlib_libraries_scanner_errors_execution() {
    let output = run_example("stdlib_libraries/scanner_errors.rb");
    assert_eq!(output, SCANNER_ERRORS_OUTPUT);
}

#[test]
fn test_stdlib_libraries_scanner_errors_no_parens_execution() {
    let output = run_example("stdlib_libraries/scanner_errors_no_parens.rb");
    assert_eq!(output, SCANNER_ERRORS_OUTPUT);
}

/// The expected output of both `stdlib_libraries/address_lookups` variants.
const ADDRESS_LOOKUPS: &str = concat!(
    "true\n",
    "true\n",
    "true\n",
    "Socket::ResolutionError\n",
    "[#<Addrinfo: 0.0.0.0:80 TCP ()>]\n",
    "[#<Addrinfo: 0.0.0.0:80 TCP (<any>)>]\n",
    "[#<Addrinfo: 0.0.0.0:80 TCP>]\n",
    "[#<Addrinfo: 127.0.0.1:80 TCP>, #<Addrinfo: 127.0.0.1:80 UDP>, #<Addrinfo: 127.0.0.1:80 SOCK_RAW>]\n",
    "[#<Addrinfo: ::1 UDP>]\n",
    "#<Addrinfo: 127.0.0.1:80 TCP (:http)>\n",
    "[\"#<Addrinfo: 127.0.0.1:80 TCP (localhost:http)>\"]\n",
    "#<Addrinfo: 127.0.0.1:80 SOCK_RAW>\n",
    "#<Addrinfo: 127.0.0.1:80 SOCK_STREAM IPPROTO_UDP>\n",
    "#<Addrinfo: 127.0.0.1:80 SOCK_RAW IPPROTO_TCP>\n",
);

#[test]
fn test_stdlib_libraries_address_lookups_execution() {
    let output = run_example("stdlib_libraries/address_lookups.rb");
    assert_eq!(output, ADDRESS_LOOKUPS);
}

#[test]
fn test_stdlib_libraries_address_lookups_no_parens_execution() {
    let output = run_example("stdlib_libraries/address_lookups_no_parens.rb");
    assert_eq!(output, ADDRESS_LOOKUPS);
}

/// Run `code` with `-e`, answering what it wrote to stdout.
fn stdout_of(code: &str) -> String {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_metorex"))
        .arg("-e")
        .arg(code)
        .output()
        .expect("failed to run metorex");
    String::from_utf8(output.stdout).expect("stdout was not utf8")
}

#[test]
fn test_continuation_resume_of_a_finished_list_answers_false() {
    let written = stdout_of(
        "p __continuation_resume__(999_999, 0, 1), __continuation_resume__(), __continuation_resumed__(1, 0), __continuation_resumed__()",
    );
    assert_eq!(written, "false\nfalse\nnil\nnil\n");
}

#[test]
fn test_continuation_called_after_its_method_returned_raises() {
    let written = stderr_of(
        "require 'continuation'; def leaves = callcc { |k| $later = k; 1 }; leaves; $later.call(2)",
    );
    assert!(written.contains(
        "a continuation cannot be resumed once the code its callcc was written in has returned (NotImplementedError)"
    ));
}

#[test]
fn test_continuation_site_is_nil_outside_any_method() {
    let written = stdout_of("p __continuation_site__()");
    assert_eq!(written, "nil\n");
}

/// The expected output of both `stdlib_libraries/socket_reading_lengths` variants.
const SOCKET_READING_LENGTHS: &str = concat!(
    "\"sleep\"\n",
    "\"abcdefghij\"\n",
    "\"klm\"\n",
    "nil\n",
    "false\n",
    "\"1234\"\n",
    "\"line one\\r\\n\\r\\n\"\n",
    "\"rest\"\n",
    "true\n",
    "nil\n",
    "nil\n",
    "\"foo\"\n",
);

#[test]
fn test_stdlib_libraries_socket_reading_lengths_execution() {
    let output = run_example("stdlib_libraries/socket_reading_lengths.rb");
    assert_eq!(output, SOCKET_READING_LENGTHS);
}

#[test]
fn test_stdlib_libraries_socket_reading_lengths_no_parens_execution() {
    let output = run_example("stdlib_libraries/socket_reading_lengths_no_parens.rb");
    assert_eq!(output, SOCKET_READING_LENGTHS);
}

/// The expected output of both `stdlib_libraries/http_exchange` variants.
const HTTP_EXCHANGE: &str = concat!(
    "\"GET /zipped answered\"\n",
    "\"GET /plain answered\"\n",
    "nil\n",
    "\"POST /form answered\"\n",
);

#[test]
fn test_stdlib_libraries_http_exchange_execution() {
    let output = run_example("stdlib_libraries/http_exchange.rb");
    assert_eq!(output, HTTP_EXCHANGE);
}

#[test]
fn test_stdlib_libraries_http_exchange_no_parens_execution() {
    let output = run_example("stdlib_libraries/http_exchange_no_parens.rb");
    assert_eq!(output, HTTP_EXCHANGE);
}

/// The expected output of both `stdlib_libraries/inflating_gzip_in_pieces` variants.
const INFLATING_GZIP_IN_PIECES: &str = concat!(
    "\"m\"\n",
    "\"aintenance window moves to 02:00\"\n",
    "[\"maintenance window moves to 02:00\", true]\n",
    "[\"zlib wrapped\"]\n",
    "\"named stream\"\n",
    "[\"\", \"\", \"\"]\n",
);

#[test]
fn test_stdlib_libraries_inflating_gzip_in_pieces_execution() {
    let output = run_example("stdlib_libraries/inflating_gzip_in_pieces.rb");
    assert_eq!(output, INFLATING_GZIP_IN_PIECES);
}

#[test]
fn test_stdlib_libraries_inflating_gzip_in_pieces_no_parens_execution() {
    let output = run_example("stdlib_libraries/inflating_gzip_in_pieces_no_parens.rb");
    assert_eq!(output, INFLATING_GZIP_IN_PIECES);
}
