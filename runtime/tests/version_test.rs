use metorex_runtime::runtime_version;

#[test]
fn test_runtime_version() {
    assert_eq!(runtime_version(), "0.1.0");
}
