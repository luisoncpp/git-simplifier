# Unified-patch parsing has two rules that look optional and are not

Both of these read as style choices in a diff parser. Both are correctness.

## Split on `'\n'`, never `str::lines()`

`str::lines()` strips a trailing `'\r'` as well as the `'\n'`. On a CRLF-checked-in
file that silently deletes the last byte of *every* context line, so the parsed
diff disagrees with the raw patch text — and the disagreement is invisible until
someone compares them side by side. Split on `'\n'` alone and drop only the single
empty trailing element the patch's final newline produces.

## A hunk ends at its declared counts, not at a sentinel prefix

The obvious loop is "read lines until one starts with `diff --git` or `@@`". That
is wrong, because a hunk's *content* is arbitrary source text: a diff of a diff, a
test fixture, or a doc about patches all contain lines that look structural. The
`@@` header already declares how many old-side and new-side lines follow, so
consume until both counts are satisfied and nothing in the file body can
desynchronize the state machine.

The same reasoning covers the sibling traps:

- `\ No newline at end of file` consumes **neither** counter, can appear twice in
  one hunk, and arrives *after* the counts are already satisfied when it belongs to
  the hunk's last line — so it needs a post-loop pass, and it must be matched on the
  leading backslash, never on the English text.
- An empty context line is a lone space, and the section heading after the closing
  `@@` can itself contain `@@`, so read the ranges first and take everything after
  the *first* `" @@"` as the heading.
- `GIT binary patch` payload lines are base85 and never contain a space, which is
  what makes a `diff --git ` guard safe as a secondary terminator alongside Git's
  actual one, the empty line.

## Related

`git diff` has no infinite-context flag, and `INT_MAX` is not a safe stand-in:
xdiff computes a hunk's end as `start + change + context` in `int` *before*
clamping it to the record count, so a near-`INT_MAX` context overflows to a
negative end and the clamp never fires. Use a bounded large value.
