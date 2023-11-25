use cargo_test_macro::cargo_test;
use cargo_test_support::curr_dir;

use super::cargo_info;

#[cargo_test]
fn case() {
    cargo_info()
        .arg("--help")
        .arg("--registry=dummy-registry")
        .assert()
        .success()
        .stdout_matches_path(curr_dir!().join("stdout.log"))
        .stderr_matches_path(curr_dir!().join("stderr.log"));
}
