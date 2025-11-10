#[test]
fn test_time_block() {
    let t = trybuild::TestCases::new();
    // ensures compiling works with profiling turned on
    t.pass("tests/ui/happy_path.rs");
    t.pass("tests/ui/function_with_a_really_long_name.rs");
    t.pass("tests/ui/bandwidth.rs");
    // ensures compile works when main function returns someting
    t.pass("tests/ui/anyhow.rs");

    t.compile_fail("tests/ui/closure.rs");
    t.compile_fail("tests/ui/struct.rs");
    t.compile_fail("tests/ui/invalid_bandwidth.rs");
}
