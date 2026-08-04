# Untracked filters must constrain ls-files, not post-filter it

Local Files diff used to return a maximal untracked set (ordinary plus every gitignored path) and let the UI hide rows. That copied Cleanup's "annotate once, toggle free" pattern, but ignored trees like `target/` make the Git walk itself the cost — filtering afterward still paid for the scan.

Respect gitignore, exclude `node_modules`, and exclude root-dot paths belong in the `ls-files` pathspecs / flags. Age and unknown-type checks run before body reads. Flipping a toggle reloads with the new query; it does not re-walk a maximal ignored set that was never needed for the default view.
