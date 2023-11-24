use cargo_test_macro::cargo_test;
use cargo_test_support::{compare::assert_ui, curr_dir, Project, TestEnv};

#[cargo_test]
fn case() {
    cargo_test_support::registry::init();
    let project = Project::from_template(curr_dir!().join("in"));
    let project_root = project.root();
    let cwd = &project_root;

    snapbox::cmd::Command::new(snapbox::cmd::cargo_bin("cargo-info"))
        .with_assert(assert_ui())
        .test_env()
        .arg("unknown")
        .arg("--color=never")
        .current_dir(cwd)
        .assert()
        .failure()
        .stdout_matches_path(curr_dir!().join("stdout.log"))
        .stderr_matches_path(curr_dir!().join("stderr.log"));

    assert_ui().subset_matches(curr_dir!().join("out"), &project_root);
}
