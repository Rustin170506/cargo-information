use cargo_test_macro::cargo_test;
use cargo_test_support::{compare::assert_ui, registry::RegistryBuilder, Project};
use cargo_test_support::{current_dir, file};

use super::cargo_info;

#[cargo_test]
fn case() {
    let _ = RegistryBuilder::new()
        .alternative()
        .no_configure_token()
        .build();
    cargo_test_support::registry::Package::new("my-package", "99999.0.0-alpha.1+my-package")
        .alternative(true)
        .publish();

    let project = Project::from_template(current_dir!().join("in"));
    let project_root = project.root();
    let cwd = &project_root;

    cargo_info()
        .arg("my-package")
        .arg("--registry=alternative")
        .current_dir(cwd)
        .assert()
        .success()
        .stdout_matches(file!["stdout.log"])
        .stderr_matches(file!["stderr.log"]);

    assert_ui().subset_matches(current_dir!().join("out"), &project_root);
}
