use std::env;

#[test]
#[ignore = "failing in nightly due to github.com/rust-lang/rust/issues/89795"]
fn ui() {
    let t = trybuild::TestCases::new();

    // Due to the way trybuild compiles the input files, `no_std`
    // doesn't work. So, since -Zbuild-std is enabled in the cargo
    // config file in the root of the crate we need to also build the
    // std crate for these tests. This wrapper script adds the necessary
    // argument when trybuild invokes cargo.
    let cargo_wrapper = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("../../../../uefi-macros/tests/cargo_wrapper");
    env::set_var("CARGO", cargo_wrapper);

    t.compile_fail("tests/ui/*.rs");
}
