use super::command::GitOutput;
use super::error::GitError;

pub(crate) fn validate(output: &GitOutput) -> Result<(), GitError> {
    let text = String::from_utf8_lossy(&output.stdout);
    let version = text.split_whitespace().nth(2).unwrap_or_default();
    let mut parts = version.split('.');
    let major = parts.next().and_then(|value| value.parse::<u32>().ok());
    let minor = parts.next().and_then(|value| value.parse::<u32>().ok());
    if major.is_some_and(|value| value > 2) || (major == Some(2) && minor.unwrap_or(0) >= 39) {
        return Ok(());
    }
    Err(GitError::UnsupportedVersion {
        raw: output.stdout.clone(),
    })
}
