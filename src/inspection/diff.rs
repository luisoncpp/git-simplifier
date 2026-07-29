use crate::git::{GitCommand, GitRunner};
use crate::rewrite::RefName;

use super::errors::InspectionError;

pub(crate) fn branch_diff(runner: &GitRunner, base: &RefName) -> Result<String, InspectionError> {
    super::queries::ensure_remote_base(base)?;
    let range = format!("{}...HEAD", base.as_str());
    let output = runner.run(GitCommand::read(
        [
            "-c",
            "diff.noprefix=false",
            "diff",
            "--binary",
            "--no-color",
            "--no-ext-diff",
            "--no-textconv",
            "--no-relative",
            "--no-renames",
            "--ignore-submodules=none",
            "--src-prefix=a/",
            "--dst-prefix=b/",
            "--unified=3",
            &range,
            "--",
        ]
        .into_iter()
        .map(Into::into)
        .collect(),
    ))?;
    String::from_utf8(output.stdout)
        .map_err(|_| InspectionError::Parse("Branch diff was not UTF-8".to_string()))
}
