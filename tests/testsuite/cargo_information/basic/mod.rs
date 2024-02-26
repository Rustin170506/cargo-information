use cargo_test_macro::cargo_test;
use cargo_test_support::file;

use super::{cargo_info_with_color, init_registry_without_token};

#[cargo_test]
fn case() {
    init_registry_without_token();
    cargo_test_support::registry::Package::new("my-package", "0.1.0")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "my-package"
            version = "0.1.0"
            description = "A package for testing"
            repository = "https://github.com/hi-rustin/cargo-infromation"
            documentation = "https://docs.rs/my-package/0.1.0"
            license = "MIT"
            edition = "2018"
            rust-version = "1.50.0"
            keywords = ["foo", "bar", "baz"]

            [features]
            default = ["feature1"]
            feature1 = []
            feature2 = []

            [dependencies]
            foo = "0.1.0"
            bar = "0.2.0"
            baz = { version = "0.3.0", optional = true }

            [[bin]]
            name = "my_bin"

            [lib]
            name = "my_lib"
            "#,
        )
        .file("src/bin/my_bin.rs", "")
        .file("src/lib.rs", "")
        .publish();
    cargo_info_with_color()
        .arg("my-package")
        .arg("--registry=dummy-registry")
        .assert()
        .success()
        .stdout_matches(file!["stdout.term.svg"])
        .stderr_matches(file!["stderr.term.svg"]);
}
