# Explicit diff prefixes do not disable `diff.noprefix`

Passing `--src-prefix=a/ --dst-prefix=b/` is not enough to stabilize patch paths when a repository has `diff.noprefix=true`: Git still emits paths without either prefix.

Override the config for the command with `-c diff.noprefix=false`, then pass the explicit prefixes. `--default-prefix` also overrides the config in newer Git, but it is absent from the project's minimum Git 2.38 documentation.

Test stable patch producers under `diff.noprefix=true`; the output remains syntactically valid without prefixes, so this drift is easy to miss.
