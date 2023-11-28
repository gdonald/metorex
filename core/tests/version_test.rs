use metorex_core::core_version;

#[test]
fn test_core_version() {
    assert_eq!(core_version(), "0.1.0");
}
