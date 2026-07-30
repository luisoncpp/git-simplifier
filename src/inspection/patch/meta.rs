use super::super::errors::InspectionError;
use super::super::model::{FileDiff, FileDiffStatus};
use super::paths;

/// `(prefix, sets the new side, implied status)`. `new mode` is never a prefix of
/// `new file mode`, so the order of this table does not matter.
const MODES: [(&str, bool, Option<FileDiffStatus>); 4] = [
    ("new file mode ", true, Some(FileDiffStatus::Added)),
    ("deleted file mode ", false, Some(FileDiffStatus::Deleted)),
    ("old mode ", false, None),
    ("new mode ", true, None),
];

/// The `diff --git` line names both sides, and under `--no-renames` they are
/// always equal, so the new side alone identifies the file. A later `+++` line
/// overrides it when Git prints one.
pub(super) fn start(new_path: String) -> Result<FileDiff, InspectionError> {
    Ok(FileDiff {
        path: paths::repo_path(new_path)?,
        previous_path: None,
        status: FileDiffStatus::Modified,
        old_mode: None,
        new_mode: None,
        binary: false,
        complete: false,
        hunks: Vec::new(),
    })
}

/// One metadata line of a file's header block. An unrecognized line is ignored
/// rather than fatal, so a header line added by a future Git cannot break the
/// whole viewer.
pub(super) fn apply(file: &mut FileDiff, line: &str) -> Result<(), InspectionError> {
    if apply_mode(file, line) {
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("index ") {
        apply_index_mode(file, rest);
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("rename from ") {
        file.status = FileDiffStatus::Renamed;
        file.previous_path = Some(paths::repo_path(paths::operand_name(rest)?)?);
        return Ok(());
    }
    apply_side(file, line)
}

fn apply_mode(file: &mut FileDiff, line: &str) -> bool {
    for (prefix, is_new_side, status) in MODES {
        let Some(mode) = line.strip_prefix(prefix) else {
            continue;
        };
        if let Some(status) = status {
            file.status = status;
        }
        let slot = if is_new_side {
            &mut file.new_mode
        } else {
            &mut file.old_mode
        };
        *slot = Some(mode.trim().to_string());
        return true;
    }
    false
}

/// `index <old>..<new> <mode>` carries the mode only when it did *not* change,
/// which makes it the sole source for the ordinary modified-file case — and the
/// easy half to miss, because the `old mode`/`new mode` pair covers only chmods.
fn apply_index_mode(file: &mut FileDiff, rest: &str) {
    let Some((_, mode)) = rest.rsplit_once(' ') else {
        return;
    };
    if mode.len() != 6 || !mode.bytes().all(|byte| byte.is_ascii_digit()) {
        return;
    }
    file.old_mode = file.old_mode.take().or_else(|| Some(mode.to_string()));
    file.new_mode = file.new_mode.take().or_else(|| Some(mode.to_string()));
}

/// The `---`/`+++` operands run to end of line, so they are the unambiguous path
/// source; `/dev/null` on either side also settles the status.
fn apply_side(file: &mut FileDiff, line: &str) -> Result<(), InspectionError> {
    if let Some(rest) = line.strip_prefix("--- ") {
        if paths::is_dev_null(rest) {
            file.status = FileDiffStatus::Added;
        }
        return Ok(());
    }
    let Some(rest) = line.strip_prefix("+++ ") else {
        return Ok(());
    };
    if paths::is_dev_null(rest) {
        file.status = FileDiffStatus::Deleted;
        return Ok(());
    }
    file.path = paths::repo_path(paths::operand_name(rest)?)?;
    Ok(())
}
