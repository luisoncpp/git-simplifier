use super::super::errors::InspectionError;
use super::super::model::{DiffHunk, DiffLine, DiffLineKind};
use super::PatchReader;

pub(super) fn parse_hunk(reader: &mut PatchReader<'_>) -> Result<DiffHunk, InspectionError> {
    let header = reader.next().unwrap_or_default();
    let mut hunk = parse_header(header)?;
    read_lines(reader, &mut hunk)?;
    consume_trailing_marker(reader, &mut hunk);
    Ok(hunk)
}

/// `@@ -old +new @@ heading`. The heading is arbitrary source text and can itself
/// contain `@@`, so the ranges are read first and the heading is whatever
/// follows the first closing `@@`.
fn parse_header(line: &str) -> Result<DiffHunk, InspectionError> {
    let rest = line.strip_prefix("@@ -").ok_or_else(|| unreadable(line))?;
    let (old, rest) = rest.split_once(" +").ok_or_else(|| unreadable(line))?;
    let (new, heading) = rest.split_once(" @@").ok_or_else(|| unreadable(line))?;
    let (old_start, old_lines) = parse_range(old).ok_or_else(|| unreadable(line))?;
    let (new_start, new_lines) = parse_range(new).ok_or_else(|| unreadable(line))?;
    Ok(DiffHunk {
        old_start,
        old_lines,
        new_start,
        new_lines,
        heading: heading.strip_prefix(' ').unwrap_or(heading).to_string(),
        lines: Vec::new(),
    })
}

/// Git omits the count when the range is a single line.
fn parse_range(text: &str) -> Option<(u32, u32)> {
    match text.split_once(',') {
        Some((start, count)) => Some((start.parse().ok()?, count.parse().ok()?)),
        None => Some((text.parse().ok()?, 1)),
    }
}

/// Termination is driven by the hunk's declared counts, never by a sentinel
/// prefix: a content line can read `diff --git …` or `@@ …`, and an empty
/// context line arrives as a lone space or, from a whitespace-stripping tool, as
/// an empty string.
fn read_lines(reader: &mut PatchReader<'_>, hunk: &mut DiffHunk) -> Result<(), InspectionError> {
    let mut numbers = LineNumbers {
        old: hunk.old_start,
        new: hunk.new_start,
    };
    let (mut old, mut new) = (0, 0);
    while old < hunk.old_lines || new < hunk.new_lines {
        let line = reader.next().ok_or_else(|| truncated(hunk))?;
        if line.starts_with('\\') {
            mark_no_newline(hunk);
            continue;
        }
        let kind = kind_of(line).ok_or_else(|| unreadable(line))?;
        match kind {
            DiffLineKind::Context => {
                old += 1;
                new += 1;
            }
            DiffLineKind::Add => new += 1,
            DiffLineKind::Del => old += 1,
        }
        hunk.lines
            .push(numbers.take(kind, line.get(1..).unwrap_or("")));
    }
    Ok(())
}

/// The marker for a hunk's final line arrives after the declared counts are
/// already satisfied, so `read_lines` cannot see it.
fn consume_trailing_marker(reader: &mut PatchReader<'_>, hunk: &mut DiffHunk) {
    while reader.peek().is_some_and(|line| line.starts_with('\\')) {
        reader.next();
        mark_no_newline(hunk);
    }
}

fn mark_no_newline(hunk: &mut DiffHunk) {
    if let Some(last) = hunk.lines.last_mut() {
        last.no_newline = true;
    }
}

fn kind_of(line: &str) -> Option<DiffLineKind> {
    match line.as_bytes().first() {
        None | Some(b' ') => Some(DiffLineKind::Context),
        Some(b'+') => Some(DiffLineKind::Add),
        Some(b'-') => Some(DiffLineKind::Del),
        _ => None,
    }
}

struct LineNumbers {
    old: u32,
    new: u32,
}

impl LineNumbers {
    fn take(&mut self, kind: DiffLineKind, text: &str) -> DiffLine {
        let (old_line, new_line) = match kind {
            DiffLineKind::Context => (Some(self.old), Some(self.new)),
            DiffLineKind::Add => (None, Some(self.new)),
            DiffLineKind::Del => (Some(self.old), None),
        };
        if old_line.is_some() {
            self.old += 1;
        }
        if new_line.is_some() {
            self.new += 1;
        }
        DiffLine {
            kind,
            old_line,
            new_line,
            text: text.to_string(),
            no_newline: false,
        }
    }
}

fn unreadable(line: &str) -> InspectionError {
    InspectionError::Parse(format!("unreadable patch hunk: {line}"))
}

fn truncated(hunk: &DiffHunk) -> InspectionError {
    InspectionError::Parse(format!(
        "patch hunk at line {} ended before its declared line counts",
        hunk.new_start
    ))
}
