use crate::rewrite::RepoPath;

pub(crate) fn block(path: &RepoPath) -> String {
    let pathspec = shell_quote(&format!(":(literal){}", path.as_str()));
    let message = shell_quote(&format!(
        "git-helper: excluded submodule is staged: {}",
        path.as_str()
    ));
    format!(
        "# git-helper excluded submodule guard\nif ! git diff --cached --quiet --ignore-submodules=none -- {pathspec}; then\n    echo {message} >&2\n    exit 1\nfi\n",
    )
}

pub(crate) fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
