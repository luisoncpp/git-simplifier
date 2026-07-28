use std::ffi::OsString;
use std::fs;

use git_helper_core::{GitCommand, GitRepository, RepositoryConfig};

use super::fixture_repo::FixtureRepo;

pub fn add_submodule(fixture: &FixtureRepo) -> GitRepository {
    let path = fixture.root.path().join("Modules/Engine");
    fs::create_dir_all(&path).unwrap();
    let child = GitRepository::open(RepositoryConfig {
        path: path.clone(),
        git_executable: "git".into(),
    })
    .unwrap();
    run(&child, &["init", "-b", "main"]);
    run(&child, &["config", "user.name", "Fixture User"]);
    run(&child, &["config", "user.email", "fixture@example.test"]);
    fs::write(path.join("README.md"), b"child\n").unwrap();
    run(&child, &["add", "--", "README.md"]);
    run(&child, &["commit", "-m", "child"]);
    fixture.add_gitlink("Modules/Engine", &head(&child), "add submodule");
    child
}

pub fn head(repo: &GitRepository) -> String {
    let output = repo
        .run(GitCommand::read(args(&["rev-parse", "HEAD"])))
        .unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

pub fn run(repo: &GitRepository, values: &[&str]) {
    repo.run(GitCommand::write(args(values))).unwrap();
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}
