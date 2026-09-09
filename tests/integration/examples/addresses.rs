// Examples covering URI

use super::run_example;

/// The expected output of both `addresses/uniform_resources` variants, which
/// differ only in whether the calls are written with parentheses.
const UNIFORM_RESOURCES_OUTPUT: &str = "URI::HTTP\n\"http\"\n\"user:pass\"\n\"user\"\n\"pass\"\n\"example.com\"\n8080\n\"/path/to/page\"\n\"query=val\"\n\"top\"\n\"/path/to/page?query=val\"\n\"http://user:pass@example.com:8080/path/to/page?query=val#top\"\ntrue\n[\"http\", \"example.com\", 8080]\n443\n80\nURI::FTP\n\"pub/ruby.tar.gz\"\n\"i\"\nURI::MailTo\n\"spam@example.com\"\n[[\"subject\", \"Hello\"]]\n\"o=Example,c=US\"\n\"comp.lang.ruby\"\n\"http://a/b/c/g\"\n\"http://a/b/g\"\n\"http://a/g\"\n\"http://a/b/c/d;p?y\"\n\"g\"\n\"http://localhost/a/e/g/i\"\n\"a%20b&c\"\n\"a b&c\"\n\"a+b%26c\"\n\"name=ruby&age=30\"\n[[\"name\", \"ruby\"], [\"age\", \"30\"]]\n[\"http://example.com/x\", \"mailto:a@b.c\"]\ntrue\ntrue\nfalse\n\"http://example.com/\"\n";

#[test]
fn test_addresses_uniform_resources_execution() {
    let output = run_example("addresses/uniform_resources.rb");
    assert_eq!(output, UNIFORM_RESOURCES_OUTPUT);
}

#[test]
fn test_addresses_uniform_resources_no_parens_execution() {
    let output = run_example("addresses/uniform_resources_no_parens.rb");
    assert_eq!(output, UNIFORM_RESOURCES_OUTPUT);
}

/// The expected output of both `addresses/internet_addresses` variants, which
/// differ only in whether the calls are written with parentheses.
const INTERNET_ADDRESSES_OUTPUT: &str = "\"192.168.1.0\"\n\"192.168.1.0\"\n\"#<IPAddr: IPv4:192.168.1.0/255.255.255.0>\"\ntrue\nfalse\ntrue\n24\n3232235776\n\"0.1.168.192.in-addr.arpa\"\n\"3ffe:505:2::1\"\n\"3ffe:0505:0002:0000:0000:0000:0000:0001\"\n\"::\"\n\"0:0:0:1::\"\n\"3ffe:505:2::\"\n\"3ffe:505:2::\"\n\"f.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.2.0.0.0.5.0.5.0.e.f.f.3.ip6.arpa\"\n\"::192.168.1.2\"\n\"::ffff:192.168.1.2\"\ntrue\n\"192.168.1.2\"\n\"::192.168.1.2\"\ntrue\nfalse\ntrue\nfalse\n\"3ffe:505:2:1::\"\n\"3ffe:505::\"\n\"0:3ffe:505:2::\"\n\"ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff\"\n\"3ffe:505::\"\nIPAddr::InvalidAddressError\n";

#[test]
fn test_addresses_internet_addresses_execution() {
    let output = run_example("addresses/internet_addresses.rb");
    assert_eq!(output, INTERNET_ADDRESSES_OUTPUT);
}

#[test]
fn test_addresses_internet_addresses_no_parens_execution() {
    let output = run_example("addresses/internet_addresses_no_parens.rb");
    assert_eq!(output, INTERNET_ADDRESSES_OUTPUT);
}
