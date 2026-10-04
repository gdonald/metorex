// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod vm {
    mod drb_tests;
    mod fiddle_tests;
    mod internet_address_tests;
    mod maps_that_hold_weakly_tests;
    mod ripper_tests;
    mod scrypt_tests;
    mod socket_option_tests;
    mod socket_write_tests;
    mod uniform_resource_tests;
    mod weak_reference_tests;
    mod x509_certificate_tests;
    mod x509_name_tests;
}
