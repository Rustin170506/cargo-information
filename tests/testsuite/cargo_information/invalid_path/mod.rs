use cargo_test_macro::cargo_test;
use cargo_test_support::curr_dir;

use super::{cargo_info, init_registry_without_token};

#[cargo_test]
fn case() {
    init_registry_without_token();

    cargo_info()
        .arg("my-package")
        .arg("--registry=dummy-registry")
        .arg("--path=./crate1")
        .current_dir(curr_dir!())
        .assert()
        .code(101)
        .stdout_matches_path(curr_dir!().join("stdout.log"))
        .stderr_matches_path(curr_dir!().join("stderr.log"));
}
