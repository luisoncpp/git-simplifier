mod support;

use std::ffi::OsString;
use std::fs;

use git_helper_core::{ExcludeSubmoduleRequest, ExclusionError, GitCommand, RepoPath};
use support::exclusion::{config_value, hook_path};
use support::fixture_repo::FixtureRepo;

#[test]
fn exclusion_plans_exact_config_hook_and_staging_guidance() {
    let fixture = fixture_with_submodule();
    let plan = fixture
        .repo
        .plan_exclude_submodule(request(
            "Modules/Engine",
            /*install_hook=*/ true,
            /*disable_recurse=*/ true,
        ))
        .unwrap();

    assert_eq!(
        plan.config_lines,
        vec![
            "git config --local --replace-all 'submodule.Modules/Engine.ignore' all",
            "git config --local --replace-all submodule.recurse false",
        ]
    );
    assert!(plan.hook_preview.contains("--ignore-submodules=none"));
    assert!(plan.hook_preview.contains("MERGE_HEAD"));
    assert_eq!(plan.staging_command, "git add -u -- ':!Modules/Engine'");
}

#[test]
fn exclusion_sets_local_config_and_installs_guard() {
    let fixture = fixture_with_submodule();
    let plan = fixture
        .repo
        .plan_exclude_submodule(request(
            "Modules/Engine",
            /*install_hook=*/ true,
            /*disable_recurse=*/ true,
        ))
        .unwrap();

    let result = fixture.repo.apply_exclude_submodule(&plan).unwrap();

    assert!(result.config_changed);
    assert!(result.hook_changed);
    assert_eq!(
        config_value(&fixture, "submodule.Modules/Engine.ignore"),
        "all"
    );
    assert_eq!(config_value(&fixture, "submodule.recurse"), "false");
    let hook = fs::read(hook_path(&fixture)).unwrap();
    assert!(hook.starts_with(b"#!/bin/sh\n"));
    assert!(hook
        .windows(plan.hook_preview.len())
        .any(|window| { window == plan.hook_preview.as_bytes() }));
    let log = fs::read_to_string(fixture.root.path().join(".git/githelper/oplog.json")).unwrap();
    assert!(log.contains("exclude-submodule"));
}

#[test]
fn existing_hook_is_preserved_and_guard_is_appended() {
    let fixture = fixture_with_submodule();
    let path = hook_path(&fixture);
    fs::write(&path, b"#!/bin/sh\ncustom-check\n").unwrap();
    let plan = fixture
        .repo
        .plan_exclude_submodule(request(
            "Modules/Engine",
            /*install_hook=*/ true,
            /*disable_recurse=*/ false,
        ))
        .unwrap();

    fixture.repo.apply_exclude_submodule(&plan).unwrap();

    let hook = fs::read(path).unwrap();
    assert!(hook.starts_with(b"#!/bin/sh\ncustom-check\n"));
    assert!(hook.ends_with(plan.hook_preview.as_bytes()));
}

#[test]
fn guard_rejects_a_staged_submodule_pointer_even_when_status_ignores_it() {
    let fixture = fixture_with_submodule();
    let child = FixtureRepo::new();
    let new_child_head = child.head();
    let plan = fixture
        .repo
        .plan_exclude_submodule(request(
            "Modules/Engine",
            /*install_hook=*/ true,
            /*disable_recurse=*/ false,
        ))
        .unwrap();
    fixture.repo.apply_exclude_submodule(&plan).unwrap();
    run_owned(
        &fixture,
        vec![
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            new_child_head.as_str(),
            "Modules/Engine",
        ],
    );

    let result = fixture.repo.run(GitCommand::write(args(&[
        "commit",
        "-m",
        "should be blocked",
    ])));

    assert!(result.is_err());
    assert!(fixture
        .status()
        .windows("Modules/Engine".len())
        .any(|window| { window == b"Modules/Engine" }));
}

#[test]
fn exclusion_rejects_a_non_submodule_path() {
    let fixture = FixtureRepo::new();
    let result = fixture.repo.plan_exclude_submodule(request(
        "README.md",
        /*install_hook=*/ false,
        /*disable_recurse=*/ false,
    ));

    assert!(matches!(
        result,
        Err(ExclusionError::InvalidState(message)) if message.contains("not a submodule")
    ));
}

#[test]
fn exclusion_rejects_a_plan_when_local_config_changed() {
    let fixture = fixture_with_submodule();
    let plan = fixture
        .repo
        .plan_exclude_submodule(request(
            "Modules/Engine",
            /*install_hook=*/ true,
            /*disable_recurse=*/ false,
        ))
        .unwrap();
    run_owned(
        &fixture,
        vec![
            "config",
            "--local",
            "submodule.Modules/Engine.ignore",
            "dirty",
        ],
    );

    let result = fixture.repo.apply_exclude_submodule(&plan);

    assert!(matches!(result, Err(ExclusionError::StalePlan)));
}

fn fixture_with_submodule() -> FixtureRepo {
    let fixture = FixtureRepo::new();
    let child = FixtureRepo::new();
    fixture.add_gitlink("Modules/Engine", &child.head(), "add submodule pointer");
    fixture
}

fn request(path: &str, install_hook: bool, disable_recurse: bool) -> ExcludeSubmoduleRequest {
    ExcludeSubmoduleRequest {
        path: RepoPath::new(path.to_string()).unwrap(),
        install_hook,
        disable_recurse,
    }
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}

fn run_owned(fixture: &FixtureRepo, values: Vec<&str>) {
    fixture.repo.run(GitCommand::write(args(&values))).unwrap();
}
