# Oplog timestamps are nanosecond epoch strings, and operation ids embed them

`recording::timestamp()` returns `SystemTime::now().as_nanos()` as a decimal string, and `OperationRecord::id` is `"<operation>-<nanos>-<pid>"`. Both are perfectly good as sort keys and as collision-resistant ids, and both are unreadable as UI text — the recovery panel was showing `uncommit-1753600000000000000-42` as an operation's identity and the raw nanosecond string as its time.

Anything rendering these has to format them: divide by 1e6 for a `Date`. Git's own `%aI` author dates in the same panels are ISO 8601 instead, so a single formatter that accepts both is worth having rather than two call sites that each guess.

The id is still the right value for `data-value` and for `resume`/`cancel` lookups — just not the right value to show.
