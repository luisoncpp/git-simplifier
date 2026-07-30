mod support;

use git_helper_core::{DiffCompare, DiffLineKind, FileDiff, FileDiffStatus, RefName, RepoPath};
use support::fixture_repo::FixtureRepo;

fn base() -> RefName {
    RefName::new("refs/remotes/origin/base".to_string()).unwrap()
}

fn numbered(lines: usize) -> String {
    (1..=lines)
        .map(|line| format!("line {line}\n"))
        .collect::<String>()
}

fn replace_line(content: &str, line: usize, text: &str) -> String {
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    lines[line - 1] = text.to_string();
    lines
        .iter()
        .map(|line| format!("{line}\n"))
        .collect::<String>()
}

fn only(files: Vec<FileDiff>) -> FileDiff {
    assert_eq!(files.len(), 1, "{files:?}");
    files.into_iter().next().unwrap()
}

#[test]
fn files_diff_numbers_context_add_and_delete_lines_of_a_modified_file() {
    let fixture = FixtureRepo::new();
    let seeded = numbered(12);
    fixture.commit_base_file("src/app.ts", &seeded);
    fixture.commit_file(
        "src/app.ts",
        &replace_line(&seeded, 6, "changed"),
        "edit line 6",
    );

    let file = only(fixture.repo.files_diff(base(), DiffCompare::Head).unwrap());

    assert_eq!(file.path.as_str(), "src/app.ts");
    assert_eq!(file.status, FileDiffStatus::Modified);
    assert!(!file.binary);
    assert!(!file.complete);
    assert_eq!(file.old_mode.as_deref(), Some("100644"));
    assert_eq!(file.new_mode.as_deref(), Some("100644"));
    assert_eq!(file.hunks.len(), 1);
    let hunk = &file.hunks[0];
    assert_eq!(
        (
            hunk.old_start,
            hunk.old_lines,
            hunk.new_start,
            hunk.new_lines
        ),
        (3, 7, 3, 7)
    );
    let removed = hunk
        .lines
        .iter()
        .find(|line| line.kind == DiffLineKind::Del)
        .unwrap();
    assert_eq!(removed.text, "line 6");
    assert_eq!(removed.old_line, Some(6));
    assert_eq!(removed.new_line, None);
    let added = hunk
        .lines
        .iter()
        .find(|line| line.kind == DiffLineKind::Add)
        .unwrap();
    assert_eq!(added.text, "changed");
    assert_eq!(added.old_line, None);
    assert_eq!(added.new_line, Some(6));
    let last = hunk.lines.last().unwrap();
    assert_eq!((last.old_line, last.new_line), (Some(9), Some(9)));
}

#[test]
fn files_diff_separates_two_hunks_that_are_far_apart() {
    let fixture = FixtureRepo::new();
    let seeded = numbered(40);
    fixture.commit_base_file("wide.txt", &seeded);
    let edited = replace_line(&replace_line(&seeded, 5, "near top"), 30, "near bottom");
    fixture.commit_file("wide.txt", &edited, "edit both ends");

    let file = only(fixture.repo.files_diff(base(), DiffCompare::Head).unwrap());

    assert_eq!(file.hunks.len(), 2);
    let first = &file.hunks[0];
    let second = &file.hunks[1];
    assert!(
        second.new_start > first.new_start + first.new_lines,
        "the two hunks must leave a gap a viewer can offer to expand: {file:?}"
    );
}

#[test]
fn files_diff_reports_an_added_file_with_no_old_line_numbers() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("added.txt", "first\nsecond\n", "add a file");

    let file = only(fixture.repo.files_diff(base(), DiffCompare::Head).unwrap());

    assert_eq!(file.status, FileDiffStatus::Added);
    assert_eq!(file.old_mode, None);
    assert_eq!(file.new_mode.as_deref(), Some("100644"));
    let hunk = &file.hunks[0];
    assert_eq!(
        (
            hunk.old_start,
            hunk.old_lines,
            hunk.new_start,
            hunk.new_lines
        ),
        (0, 0, 1, 2)
    );
    assert!(hunk
        .lines
        .iter()
        .all(|line| line.kind == DiffLineKind::Add && line.old_line.is_none()));
}

#[test]
fn files_diff_reports_a_deleted_file_with_no_new_line_numbers() {
    let fixture = FixtureRepo::new();
    fixture.remove_file("README.md", "drop the readme");

    let file = only(fixture.repo.files_diff(base(), DiffCompare::Head).unwrap());

    assert_eq!(file.path.as_str(), "README.md");
    assert_eq!(file.status, FileDiffStatus::Deleted);
    assert_eq!(file.old_mode.as_deref(), Some("100644"));
    assert_eq!(file.new_mode, None);
    let hunk = &file.hunks[0];
    assert_eq!((hunk.new_start, hunk.new_lines), (0, 0));
    assert!(hunk
        .lines
        .iter()
        .all(|line| line.kind == DiffLineKind::Del && line.new_line.is_none()));
}

#[test]
fn files_diff_marks_the_last_line_when_the_file_has_no_trailing_newline() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("tail.txt", "only line", "add a file with no final newline");

    let file = only(fixture.repo.files_diff(base(), DiffCompare::Head).unwrap());

    let lines = &file.hunks[0].lines;
    assert_eq!(lines.len(), 1, "the marker is not itself a line: {lines:?}");
    assert_eq!(lines[0].text, "only line");
    assert!(lines[0].no_newline);
}

#[test]
fn files_diff_skips_a_binary_payload_without_losing_the_next_file() {
    let fixture = FixtureRepo::new();
    fixture.commit_bytes(
        "logo.png",
        &[0x89, b'P', 0x00, 0x01, 0x02, 0xff, 0x00],
        "add an image",
    );
    fixture.commit_file("after.txt", "text\n", "add text after the image");

    let files = fixture.repo.files_diff(base(), DiffCompare::Head).unwrap();

    assert_eq!(files.len(), 2, "{files:?}");
    let image = files
        .iter()
        .find(|file| file.path.as_str() == "logo.png")
        .unwrap();
    assert!(image.binary);
    assert!(image.hunks.is_empty());
    let text = files
        .iter()
        .find(|file| file.path.as_str() == "after.txt")
        .unwrap();
    assert!(!text.binary);
    assert_eq!(text.hunks[0].lines[0].text, "text");
    let leaked = files
        .iter()
        .flat_map(|file| file.hunks.iter())
        .flat_map(|hunk| hunk.lines.iter())
        .any(|line| line.text.contains("GIT binary patch") || line.text.contains("literal "));
    assert!(!leaked, "the base85 payload must not reach a rendered line");
}

#[test]
fn files_diff_reports_a_mode_change_with_no_hunks() {
    let fixture = FixtureRepo::new();
    fixture.chmod_executable("README.md", "make the readme executable");

    let file = only(fixture.repo.files_diff(base(), DiffCompare::Head).unwrap());

    assert_eq!(file.status, FileDiffStatus::Modified);
    assert_eq!(file.old_mode.as_deref(), Some("100644"));
    assert_eq!(file.new_mode.as_deref(), Some("100755"));
    assert!(file.hunks.is_empty());
    assert!(!file.binary);
}

#[test]
fn files_diff_preserves_carriage_returns_in_content() {
    let fixture = FixtureRepo::new();
    fixture.set_config("core.autocrlf", "false");
    fixture.commit_file("crlf.txt", "alpha\r\nbeta\r\n", "add a CRLF file");

    let file = only(fixture.repo.files_diff(base(), DiffCompare::Head).unwrap());

    let texts: Vec<&str> = file.hunks[0]
        .lines
        .iter()
        .map(|line| line.text.as_str())
        .collect();
    assert_eq!(
        texts,
        vec!["alpha\r", "beta\r"],
        "str::lines() would have eaten the \\r"
    );
}

#[test]
fn files_diff_parses_a_header_path_that_contains_a_space() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("spaced name.txt", "content\n", "add a spaced path");

    let file = only(fixture.repo.files_diff(base(), DiffCompare::Head).unwrap());

    assert_eq!(file.path.as_str(), "spaced name.txt");
}

#[test]
fn files_diff_parses_an_octal_escaped_non_ascii_path() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("café.txt", "content\n", "add a non-ASCII path");

    let file = only(fixture.repo.files_diff(base(), DiffCompare::Head).unwrap());

    assert_eq!(file.path.as_str(), "café.txt");
}

#[test]
fn files_diff_describes_the_same_changes_as_the_copyable_patch() {
    let fixture = FixtureRepo::new();
    let seeded = numbered(20);
    fixture.commit_base_file("src/app.ts", &seeded);
    fixture.commit_file("src/app.ts", &replace_line(&seeded, 10, "changed"), "edit");
    fixture.commit_file("added.txt", "one\ntwo\n", "add");
    fixture.remove_file("README.md", "remove");
    fixture.set_config("color.ui", "always");
    fixture.set_config("diff.noprefix", "true");

    let patch = fixture.repo.branch_diff(base(), DiffCompare::Head).unwrap();
    let files = fixture.repo.files_diff(base(), DiffCompare::Head).unwrap();

    let added_text = patch
        .lines()
        .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
        .count();
    let removed_text = patch
        .lines()
        .filter(|line| line.starts_with('-') && !line.starts_with("---"))
        .count();
    let lines = || files.iter().flat_map(hunk_lines);
    assert_eq!(
        added_text,
        lines().filter(|kind| *kind == DiffLineKind::Add).count()
    );
    assert_eq!(
        removed_text,
        lines().filter(|kind| *kind == DiffLineKind::Del).count()
    );
    let mut paths: Vec<&str> = files.iter().map(|file| file.path.as_str()).collect();
    paths.sort_unstable();
    assert_eq!(paths, vec!["README.md", "added.txt", "src/app.ts"]);
}

fn hunk_lines(file: &FileDiff) -> impl Iterator<Item = DiffLineKind> + '_ {
    file.hunks
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .map(|line| line.kind)
}

#[test]
fn full_file_diff_returns_every_line_of_one_file() {
    let fixture = FixtureRepo::new();
    let seeded = numbered(40);
    fixture.commit_base_file("wide.txt", &seeded);
    fixture.commit_file(
        "wide.txt",
        &replace_line(&seeded, 30, "changed"),
        "edit line 30",
    );
    let path = RepoPath::new("wide.txt".to_string()).unwrap();

    let file = fixture.repo.full_file_diff(base(), path, DiffCompare::Head).unwrap().unwrap();

    assert!(file.complete);
    assert_eq!(file.hunks.len(), 1);
    let hunk = &file.hunks[0];
    assert_eq!((hunk.old_start, hunk.new_start), (1, 1));
    assert_eq!(hunk.new_lines, 40);
    let first = &hunk.lines[0];
    assert_eq!(first.kind, DiffLineKind::Context);
    assert_eq!(first.new_line, Some(1));
}

#[test]
fn full_file_diff_ignores_other_changed_files() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("one.txt", "one\n", "add one");
    fixture.commit_file("two.txt", "two\n", "add two");
    let path = RepoPath::new("two.txt".to_string()).unwrap();

    let file = fixture.repo.full_file_diff(base(), path, DiffCompare::Head).unwrap().unwrap();

    assert_eq!(file.path.as_str(), "two.txt");
}

#[test]
fn full_file_diff_pins_its_pathspec_to_the_repository_root() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("sub/nested.txt", "nested\n", "add a nested file");
    fixture.set_config("diff.relative", "true");
    let nested = fixture.reopen_at("sub");
    let path = RepoPath::new("sub/nested.txt".to_string()).unwrap();

    let file = nested.full_file_diff(base(), path, DiffCompare::Head).unwrap().unwrap();

    assert_eq!(file.path.as_str(), "sub/nested.txt");
}

#[test]
fn full_file_diff_returns_nothing_for_a_path_with_no_changes() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("changed.txt", "changed\n", "add a file");
    let path = RepoPath::new("README.md".to_string()).unwrap();

    let file = fixture.repo.full_file_diff(base(), path, DiffCompare::Head).unwrap();

    assert!(
        file.is_none(),
        "an unchanged path is a refresh, not an error: {file:?}"
    );
}

#[test]
fn files_diff_rejects_a_base_that_is_not_remote_tracking() {
    let fixture = FixtureRepo::new();
    let local = RefName::new("refs/heads/base".to_string()).unwrap();
    let path = RepoPath::new("README.md".to_string()).unwrap();

    let listed = fixture.repo.files_diff(local.clone(), DiffCompare::Head).unwrap_err();
    let expanded = fixture.repo.full_file_diff(local, path, DiffCompare::Head).unwrap_err();

    assert!(listed.to_string().contains("remote-tracking"), "{listed}");
    assert!(
        expanded.to_string().contains("remote-tracking"),
        "{expanded}"
    );
}
