# Git log format separators keep line terminators

**Date:** 2026-07-27

Adding a custom record separator such as `%x1e` to `git log --format` does not suppress Git's own line termination. Splitting on the custom byte therefore leaves a final whitespace-only fragment, which looks like a malformed record unless the parser explicitly discards it.

Test pretty-format parsers through a real fixture repository rather than only with handcrafted bytes. The fixture captures both the requested separators and Git's surrounding output behavior.
