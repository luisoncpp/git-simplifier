# `git rev-parse --git-path hooks` returns a directory

Git's `rev-parse --git-path hooks` result is the hooks directory, not a hook file. Append the hook filename (for example, `pre-commit`) before reading or writing. Treating the result as a file produces a Windows permission error when the code tries to read the directory.
