use crate::rewrite::RepoPath;

pub(crate) fn block(path: &RepoPath) -> String {
    let pathspec = shell_quote(&format!(":(literal){}", path.as_str()));
    let message = shell_quote(&format!(
        "git-helper: excluded submodule is staged: {}",
        path.as_str()
    ));
    format!(
        "# git-helper excluded submodule guard\n{}",
        guard_body(&pathspec, &message)
    )
}

fn guard_body(pathspec: &str, message: &str) -> String {
    format!(
        "if ! git diff --cached --quiet --ignore-submodules=none -- {pathspec}; then\n\
         if git rev-parse --verify --quiet MERGE_HEAD >/dev/null \\\n\
         && git diff --cached --quiet --ignore-submodules=none MERGE_HEAD -- {pathspec}; then\n\
         :\n\
         else\n\
         echo {message} >&2\n\
         exit 1\n\
         fi\n\
         fi\n"
    )
}

pub(crate) fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
