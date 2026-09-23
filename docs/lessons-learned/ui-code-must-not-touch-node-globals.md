# UI code must not touch Node globals

The workbench tests run the UI's `.ts` sources under Node, where `process`, `Buffer`, etc. exist. The webview has none of them. A leftover `process.stderr.write` debug line therefore passes every test and `tsc` (Node types are in scope), yet throws a `ReferenceError` in the app.

Where it throws decides how it looks. Inside `render()` (e.g. `highlightLines`), the exception escapes `controller.run`'s `finally`, so the DOM stays on the **busy frame** rendered before the work started. Files diff then shows `0 file(s) +0 −0` while Raw diff over the same patch works — it reads like a parser bug, but the backend is fine.

- Debug UI code with `console.*`, and remove it before committing; never use `process.*`.
- `test/ui-highlight-webview.test.mjs` hides `globalThis.process` around highlighting to guard the render path.
- If a view shows its empty/loading shape with data you know exists, check the webview console for a render exception before debugging the backend.

## Prism under the test stub

`ensureGrammars` only imports Prism when `globalThis.document` exists. The `prismjs` main entry bundles the file-highlight plugin, which reads `Element.prototype` at import time. A stub needs an `Element` too, or the import fails, `ensureGrammars` swallows the error, and highlighting tests fail with plain text. Use `test/support/prism-dom.mjs`.
