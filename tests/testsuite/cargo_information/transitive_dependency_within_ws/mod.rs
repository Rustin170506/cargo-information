use cargo_test_macro::cargo_test;
use cargo_test_support::{compare::assert_ui, current_dir, file, Project};

use super::{cargo_info, init_registry_without_token};

#[cargo_test]
fn case() {
    init_registry_without_token();
    for ver in ["1.0.0", "2.0.0", "3.0.0"] {
        cargo_test_support::registry::Package::new("my-package", ver).publish();
    }
    // Dep1 depends on 3.0.0, Dep2 depends on 2.0.0, Dep3 depends on 1.0.0
    cargo_test_support::registry::Package::new("dep1", "1.0.0")
        .dep("my-package", "1.0.0")
        .publish();
    cargo_test_support::registry::Package::new("dep2", "1.0.0")
        .dep("my-package", "2.0.0")
        .publish();
    cargo_test_support::registry::Package::new("dep3", "1.0.0")
        .dep("my-package", "3.0.0")
        .publish();

    let project = Project::from_template(current_dir!().join("in"));
    let project_root = project.root();
    let crate1_root = project_root.join("crates/crate1");
    let crate2_root = project_root.join("crates/crate2");
    let root_directory = &project_root;
    let crate1_directory = &crate1_root;
    let crate2_directory = &crate2_root;

    cargo_info()
        .arg("my-package")
        .arg("--registry=dummy-registry")
        .current_dir(root_directory)
        .assert()
        .stdout_matches(file!["root.stdout.log"])
        .stderr_matches(file!["root.stderr.log"]);
    cargo_info()
        .arg("my-package")
        .arg("--registry=dummy-registry")
        .current_dir(crate1_directory)
        .assert()
        .stdout_matches(file!["crate1.stdout.log"])
        .stderr_matches(file!["crate1.stderr.log"]);
    cargo_info()
        .arg("my-package")
        .arg("--registry=dummy-registry")
        .current_dir(crate2_directory)
        .assert()
        .stdout_matches(file!["crate2.stdout.log"])
        .stderr_matches(file!["crate2.stderr.log"]);

    assert_ui().subset_matches(current_dir!().join("out"), &project_root);
}
