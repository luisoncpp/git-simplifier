use std::ffi::OsString;
use std::path::PathBuf;

use git_helper_core::GitCommand;

use super::fixture_repo::FixtureRepo;

pub fn config_value(fixture: &FixtureRepo, key: &str) -> String {
    String::from_utf8(read(fixture, vec!["config", "--local", "--get", key]))
        .unwrap()
        .trim()
        .to_string()
}

pub fn hook_path(fixture: &FixtureRepo) -> PathBuf {
    let output = read(fixture, vec!["rev-parse", "--git-path", "hooks"]);
    let path = PathBuf::from(String::from_utf8(output).unwrap().trim());
    let hooks = if path.is_absolute() {
        path
    } else {
        fixture.root.path().join(path)
    };
    hooks.join("pre-commit")
}

fn read(fixture: &FixtureRepo, values: Vec<&str>) -> Vec<u8> {
    fixture
        .repo
        .run(GitCommand::read(args(&values)))
        .unwrap()
        .stdout
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
