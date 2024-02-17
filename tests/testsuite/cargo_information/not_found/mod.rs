use cargo_test_macro::cargo_test;
use cargo_test_support::{compare::assert_ui, current_dir, file, Project};

use super::{cargo_info, init_registry_without_token};

#[cargo_test]
fn case() {
    init_registry_without_token();
    let project = Project::from_template(current_dir!().join("in"));
    let project_root = project.root();
    let cwd = &project_root;

    cargo_info()
        .arg("unknown")
        .arg("--registry=dummy-registry")
        .current_dir(cwd)
        .assert()
        .failure()
        .stdout_matches(file!["stdout.log"])
        .stderr_matches(file!["stderr.log"]);

    assert_ui().subset_matches(current_dir!().join("out"), &project_root);
}
