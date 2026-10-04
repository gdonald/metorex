// 3.14 is a sample float throughout these tests, not an approximation of pi.
#![allow(clippy::approx_constant)]

pub mod common;

mod vm {
    mod allocation_tracing_tests;
    mod asking_about_a_file_tests;
    mod errno_message_tests;
    mod feature_resolution_tests;
    mod file_facts_tests;
    mod io_wait_tests;
    mod loading_under_threads_tests;
    mod naming_files_tests;
    mod null_byte_path_tests;
    mod reading_forms_tests;
    mod reading_switches_tests;
    mod reading_the_user_database_tests;
    mod thread_class_names_tests;
    mod walking_a_directory_tests;
}
