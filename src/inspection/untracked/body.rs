use std::fs;

use crate::git::GitRunner;
use crate::rewrite::RepoPath;

use super::super::errors::InspectionError;
use super::super::model::{
    DiffHunk, DiffLine, DiffLineKind, FileDiff, FileDiffStatus, UntrackedAnnotations,
};

pub(super) fn should_stub(annotations: &UntrackedAnnotations) -> bool {
    annotations.gitignored || annotations.in_node_modules
}

pub(super) fn stub(path: RepoPath, untracked: UntrackedAnnotations) -> FileDiff {
    FileDiff {
        path,
        previous_path: None,
        status: FileDiffStatus::Added,
        old_mode: None,
        new_mode: Some("100644".to_string()),
        binary: false,
        complete: false,
        hunks: Vec::new(),
        untracked: Some(untracked),
    }
}

pub(super) fn synthesize(
    runner: &GitRunner,
    path: RepoPath,
    untracked: UntrackedAnnotations,
) -> Result<FileDiff, InspectionError> {
    let worktree = runner.repo_path().join(path.as_str());
    let bytes = fs::read(&worktree).map_err(|error| InspectionError::Parse(error.to_string()))?;
    let (binary, hunks) = if bytes.contains(&0) || std::str::from_utf8(&bytes).is_err() {
        (true, Vec::new())
    } else {
        let text = String::from_utf8(bytes).expect("validated above");
        (false, vec![text_hunk(diff_lines_from_text(&text))])
    };
    Ok(FileDiff {
        path,
        previous_path: None,
        status: FileDiffStatus::Added,
        old_mode: None,
        new_mode: Some("100644".to_string()),
        binary,
        complete: true,
        hunks,
        untracked: Some(untracked),
    })
}

fn text_hunk(lines: Vec<DiffLine>) -> DiffHunk {
    let count = lines.len() as u32;
    DiffHunk {
        old_start: 0,
        old_lines: 0,
        new_start: if count == 0 { 0 } else { 1 },
        new_lines: count,
        heading: String::new(),
        lines,
    }
}

fn diff_lines_from_text(text: &str) -> Vec<DiffLine> {
    if text.is_empty() {
        return Vec::new();
    }
    let ends_with_newline = text.ends_with('\n');
    let body = if ends_with_newline {
        &text[..text.len() - 1]
    } else {
        text
    };
    let mut lines = Vec::new();
    let mut new_line = 1u32;
    for part in body.split('\n') {
        lines.push(DiffLine {
            kind: DiffLineKind::Add,
            old_line: None,
            new_line: Some(new_line),
            text: part.to_string(),
            no_newline: false,
        });
        new_line += 1;
    }
    if !ends_with_newline {
        if let Some(last) = lines.last_mut() {
            last.no_newline = true;
        }
    }
    lines
}
