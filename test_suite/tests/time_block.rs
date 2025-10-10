#[test]
fn test_time_block() {
    let t = trybuild::TestCases::new();
    // ensures compiling works with profiling turned on
    t.pass("tests/ui/happy_path.rs");
    t.compile_fail("tests/ui/closure.rs");
    t.compile_fail("tests/ui/struct.rs");
}
