use crate::rewrite::RepoPath;

use super::super::errors::InspectionError;

const DEV_NULL: &str = "/dev/null";

pub(super) fn repo_path(value: String) -> Result<RepoPath, InspectionError> {
    RepoPath::new(value).map_err(InspectionError::Parse)
}

pub(super) fn is_dev_null(operand: &str) -> bool {
    trim_tab(operand) == DEV_NULL
}

/// The new side of a `diff --git ` operand pair.
pub(super) fn header_new_path(rest: &str) -> Result<String, InspectionError> {
    let second = split_operands(rest).ok_or_else(|| unreadable(rest))?;
    operand_name(second)
}

/// A `diff --git`, `---`, or `+++` operand: optionally quoted, optionally
/// followed by a tab, and carrying an `a/` or `b/` side prefix.
pub(super) fn operand_name(operand: &str) -> Result<String, InspectionError> {
    let trimmed = trim_tab(operand);
    let name = match trimmed.strip_prefix('"') {
        Some(inner) => unquote(inner.strip_suffix('"').unwrap_or(inner))?,
        None => trimmed.to_string(),
    };
    Ok(strip_side_prefix(name))
}

/// The two operands are space separated and a tracked name may itself contain
/// spaces, so tokenizing cannot settle the split. Three recoverable shapes, in
/// order: a quoted first operand ends at its closing quote; two identical names
/// (the only shape `--no-renames` produces) split exactly in the middle;
/// otherwise take the first space that leaves `b/` on the right.
fn split_operands(rest: &str) -> Option<&str> {
    if rest.starts_with('"') {
        return rest.get(closing_quote(rest)? + 2..);
    }
    if let Some(second) = middle_split(rest) {
        return Some(second);
    }
    rest.match_indices(" b/")
        .map(|(at, _)| &rest[at + 1..])
        .next()
}

fn closing_quote(rest: &str) -> Option<usize> {
    let bytes = rest.as_bytes();
    let mut at = 1;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at += 2,
            b'"' => return Some(at),
            _ => at += 1,
        }
    }
    None
}

/// `a/NAME b/NAME` is `2 + n + 1 + 2 + n` bytes long, so the name length falls
/// out of the total and both halves can be verified against each other.
fn middle_split(rest: &str) -> Option<&str> {
    if rest.len() < 5 || !(rest.len() - 5).is_multiple_of(2) || !rest.starts_with("a/") {
        return None;
    }
    let name_length = (rest.len() - 5) / 2;
    if rest.as_bytes().get(2 + name_length) != Some(&b' ') {
        return None;
    }
    let left = rest.get(2..2 + name_length)?;
    let right = rest.get(3 + name_length..)?;
    if right.strip_prefix("b/")? != left {
        return None;
    }
    Some(right)
}

fn strip_side_prefix(name: String) -> String {
    for prefix in ["a/", "b/"] {
        if let Some(stripped) = name.strip_prefix(prefix) {
            return stripped.to_string();
        }
    }
    name
}

fn trim_tab(operand: &str) -> &str {
    operand.strip_suffix('\t').unwrap_or(operand)
}

/// C-style unquoting. `core.quotepath` (on by default) escapes a non-ASCII name
/// one byte at a time, so octal escapes must accumulate as bytes and be decoded
/// once at the end; decoding per escape produces mojibake.
fn unquote(inner: &str) -> Result<String, InspectionError> {
    let bytes = inner.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] != b'\\' {
            out.push(bytes[at]);
            at += 1;
            continue;
        }
        let marker = *bytes.get(at + 1).ok_or_else(|| unreadable(inner))?;
        at += 2;
        if let Some(byte) = simple_escape(marker) {
            out.push(byte);
            continue;
        }
        let (byte, used) = octal_escape(marker, &bytes[at..]).ok_or_else(|| unreadable(inner))?;
        out.push(byte);
        at += used;
    }
    String::from_utf8(out).map_err(|_| unreadable(inner))
}

fn simple_escape(marker: u8) -> Option<u8> {
    match marker {
        b'\\' => Some(b'\\'),
        b'"' => Some(b'"'),
        b'a' => Some(0x07),
        b'b' => Some(0x08),
        b'f' => Some(0x0c),
        b'n' => Some(b'\n'),
        b'r' => Some(b'\r'),
        b't' => Some(b'\t'),
        b'v' => Some(0x0b),
        _ => None,
    }
}

/// Git prints three octal digits, but accept one or two so a hand-written patch
/// cannot make the whole file unreadable.
fn octal_escape(first: u8, rest: &[u8]) -> Option<(u8, usize)> {
    if !(b'0'..=b'7').contains(&first) {
        return None;
    }
    let mut value = u32::from(first - b'0');
    let mut used = 0;
    while used < 2 {
        let Some(digit) = rest.get(used).filter(|byte| (b'0'..=b'7').contains(byte)) else {
            break;
        };
        value = value * 8 + u32::from(digit - b'0');
        used += 1;
    }
    u8::try_from(value).ok().map(|byte| (byte, used))
}

fn unreadable(rest: &str) -> InspectionError {
    InspectionError::Parse(format!("unreadable patch path: {rest}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_symmetric_header_with_a_space_in_the_name_splits_in_the_middle() {
        assert_eq!(
            header_new_path("a/my file.txt b/my file.txt").unwrap(),
            "my file.txt"
        );
    }

    #[test]
    fn an_asymmetric_header_falls_back_to_the_b_side() {
        assert_eq!(header_new_path("a/old.txt b/new.txt").unwrap(), "new.txt");
    }

    #[test]
    fn a_quoted_header_is_unquoted_as_bytes_before_decoding() {
        let header = "\"a/caf\\303\\251.txt\" \"b/caf\\303\\251.txt\"";
        assert_eq!(header_new_path(header).unwrap(), "café.txt");
    }

    #[test]
    fn simple_escapes_survive_unquoting() {
        assert_eq!(
            operand_name("\"b/say \\\"hi\\\".txt\"").unwrap(),
            "say \"hi\".txt"
        );
        assert_eq!(operand_name("\"b/back\\\\slash\"").unwrap(), "back\\slash");
    }

    #[test]
    fn an_operand_loses_its_side_prefix_and_trailing_tab() {
        assert_eq!(operand_name("b/src/app.ts\t").unwrap(), "src/app.ts");
        assert!(is_dev_null("/dev/null\t"));
        assert!(!is_dev_null("b/dev/null"));
    }
}
