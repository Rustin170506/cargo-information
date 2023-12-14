#[test]
// You can use `TRYCMD=overwrite` to overwrite the expected output.
fn readme() {
    let t = trycmd::TestCases::new();
    let cargo_info = trycmd::cargo::cargo_bin("cargo-info");
    t.register_bin("cargo", cargo_info);
    t.case("README.md");
}
