use cargo_test_macro::cargo_test;
use cargo_test_support::{basic_manifest, git, project};
use snapbox::file;

use super::{cargo_info, init_registry_without_token};

#[cargo_test]
fn case() {
    init_registry_without_token();
    let baz = git::new("baz", |project| {
        project
            .file("Cargo.toml", &basic_manifest("baz", "0.1.0"))
            .file("src/lib.rs", "")
    });

    let foo = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "foo"
                    version = "0.1.0"

                    [dependencies]
                    baz = {{ git = '{}' }}
                "#,
                baz.url()
            ),
        )
        .file("src/lib.rs", &format!(""))
        .build();

    let project_root = foo.root();
    let cwd = &project_root;

    cargo_info()
        .arg("foo")
        .current_dir(cwd)
        .assert()
        .success()
        .stdout_matches(file!["stdout.log"])
        .stderr_matches(file!["stderr.log"]);
}
