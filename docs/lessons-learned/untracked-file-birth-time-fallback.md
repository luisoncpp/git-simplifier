# Untracked file birth time falls back to mtime

When annotating Local untracked files for the "creation after HEAD" filter, `Metadata::created()` is preferred and `modified()` is the fallback when birth time is unavailable (common on Linux).

Compare file seconds strictly before HEAD commit seconds (`%ct`), not `<=`, so a file created in the same second as HEAD still counts as "after HEAD" for filtering.

The maximal untracked set unions `git ls-files -o --exclude-standard` with `git ls-files -o -i --exclude-standard`; `--no-exclude-standard` is not portable across Git versions.
