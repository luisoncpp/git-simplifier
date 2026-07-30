mod hunks;
mod meta;
mod paths;

use super::errors::InspectionError;
use super::model::FileDiff;

const FILE_HEADER: &str = "diff --git ";
const BINARY_HEADER: &str = "GIT binary patch";
const HUNK_HEADER: &str = "@@";

/// A cursor over a patch's lines.
///
/// Lines are split on `'\n'` alone: `str::lines()` also strips a trailing
/// `'\r'`, which silently corrupts every context line of a CRLF-checked-in file
/// and would make the structured diff disagree with the copyable patch text.
pub(super) struct PatchReader<'a> {
    lines: Vec<&'a str>,
    index: usize,
}

impl<'a> PatchReader<'a> {
    fn new(text: &'a str) -> Self {
        let mut lines: Vec<&'a str> = text.split('\n').collect();
        if lines.last() == Some(&"") {
            lines.pop();
        }
        Self { lines, index: 0 }
    }

    fn peek(&self) -> Option<&'a str> {
        self.lines.get(self.index).copied()
    }

    fn next(&mut self) -> Option<&'a str> {
        let line = self.peek()?;
        self.index += 1;
        Some(line)
    }
}

pub(super) fn parse_patch(text: &str) -> Result<Vec<FileDiff>, InspectionError> {
    let mut reader = PatchReader::new(text);
    let mut files = Vec::new();
    while let Some(line) = reader.peek() {
        if !line.starts_with(FILE_HEADER) {
            reader.next();
            continue;
        }
        files.push(parse_file(&mut reader)?);
    }
    Ok(files)
}

fn parse_file(reader: &mut PatchReader<'_>) -> Result<FileDiff, InspectionError> {
    let header = reader.next().unwrap_or_default();
    let mut file = meta::start(paths::header_new_path(&header[FILE_HEADER.len()..])?)?;
    read_metadata(reader, &mut file)?;
    if reader.peek() == Some(BINARY_HEADER) {
        reader.next();
        file.binary = true;
        skip_binary_block(reader);
        return Ok(file);
    }
    while reader
        .peek()
        .is_some_and(|line| line.starts_with(HUNK_HEADER))
    {
        file.hunks.push(hunks::parse_hunk(reader)?);
    }
    Ok(file)
}

fn read_metadata(reader: &mut PatchReader<'_>, file: &mut FileDiff) -> Result<(), InspectionError> {
    while let Some(line) = reader.peek() {
        if line.starts_with(FILE_HEADER) || line.starts_with(HUNK_HEADER) || line == BINARY_HEADER {
            return Ok(());
        }
        reader.next();
        meta::apply(file, line)?;
    }
    Ok(())
}

/// Git terminates the base85 payload with an empty line. Payload lines never
/// contain a space, so the file-header guard cannot match one by accident.
fn skip_binary_block(reader: &mut PatchReader<'_>) {
    while let Some(line) = reader.peek() {
        if line.starts_with(FILE_HEADER) {
            return;
        }
        reader.next();
        if line.is_empty() {
            return;
        }
    }
}
