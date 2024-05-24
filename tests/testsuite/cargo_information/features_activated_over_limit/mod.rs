use cargo_test_macro::cargo_test;
use cargo_test_support::file;

use super::{cargo_info, init_registry_without_token};

#[cargo_test]
fn case() {
    const MANY_FEATURES_COUNT: usize = 200;
    const DEFAULT_FEATURES_COUNT: usize = 100;

    init_registry_without_token();
    let mut test_package =
        cargo_test_support::registry::Package::new("your-face", "99999.0.0+my-package");
    let features = (0..MANY_FEATURES_COUNT)
        .map(|i| format!("eyes{i:03}"))
        .collect::<Vec<_>>();
    for name in &features {
        test_package.feature(name.as_str(), &[]);
    }
    let default_features = features
        .iter()
        .take(DEFAULT_FEATURES_COUNT)
        .map(|s| s.as_str())
        .collect::<Vec<_>>();
    test_package.feature("default", &default_features);
    test_package.publish();

    cargo_info()
        .arg("your-face")
        .arg("--registry=dummy-registry")
        .assert()
        .success()
        .stdout_eq_(file!["stdout.log"])
        .stderr_eq_(file!["stderr.log"]);
}
