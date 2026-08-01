# `%(refname:strip=N)` counts the branch name too

`git for-each-ref --format=%(refname:strip=N)` strips `N` slash-separated components **from the whole refname**, including the part you want to keep.

`refs/githelper/wip/feature` has exactly four components. `strip=4` therefore returns an **empty string**, not `feature`:

```
$ git for-each-ref --format='[%(refname:strip=4)]' refs/githelper/wip
[]                 # refs/githelper/wip/spike
[thing]            # refs/githelper/wip/team/thing  -- lost "team/"
```

Two failure modes, and the first is the dangerous one:

- A simple branch name yields `""`. Combined with the near-universal `.filter(|line| !line.is_empty())` guard on `for-each-ref` output, the record is then **dropped silently** — the list comes back empty and looks like "there is nothing here" rather than like a parse failure. This shipped in `src/inspection/queries.rs`, where it made `LocalBranchChoice::saved_work` permanently `false` and the branch picker's "has Saved work" marker unreachable.
- A slashed branch name loses its first segment, producing a name that looks plausible and matches nothing.

Use `%(refname)` and `strip_prefix("refs/githelper/wip/")` in code instead. It is correct for any number of slashes in the branch name, and a ref that does not match the prefix is visibly skipped rather than silently becoming an empty string.

More generally: any `for-each-ref` parser that filters out empty records can turn a format bug into a silent empty list. If a listing is a *safety* input — Cleanup's Saved-work exclusion is — prefer a parse that can fail loudly over one that can under-report.
