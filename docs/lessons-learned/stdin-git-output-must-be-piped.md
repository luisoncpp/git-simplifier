# Git Commands With Stdin Must Pipe Their Output

When a Git command receives stdin, spawning it directly and then calling `wait_with_output()` does not capture stdout or stderr unless both streams were explicitly set to `Stdio::piped()`. The child can succeed and print a commit SHA to the parent process while the caller receives empty output. Configure stdin, stdout, and stderr as pipes together before spawning commands such as `commit-tree`.
