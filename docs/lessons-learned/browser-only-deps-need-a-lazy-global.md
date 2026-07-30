# A browser-only dependency needs a lazy import and a published global

`npm test` loads the UI's `.ts` sources directly, with no bundler. That makes any
static `import` of a browser library a cost every test run pays — and a hard
failure if the package is missing. Prism is the first such dependency here, so the
pattern it needs is worth reusing:

- **Dynamic `import()` behind a `globalThis.document` check.** Without a document
  nothing is imported at all and the feature degrades (here: to escaped plain
  text). As a bonus an `import()` expression sidesteps `verbatimModuleSyntax` and
  `erasableSyntaxOnly` entirely, and works with `esModuleInterop` off, which a
  default import against an `export =` declaration would not.
- **Publish the core to `globalThis` before loading anything that extends it.**
  Prism's grammar files bind a free `Prism` identifier through the global rather
  than importing it, so a grammar loaded before `globalThis.Prism` is set fails.
  Setting `globalThis.Prism = { manual: true }` *first* also suppresses Prism's
  automatic document scan on load.
- **Wrap the whole thing in try/catch.** Decoration must never fail the feature it
  decorates.
- **Guard the arrangement with a test**, not a comment: assert the rendered markup
  contains escaped text and no `class="token`. If someone converts the adapter to a
  static import, that assertion breaks (or the suite fails to import at all)
  instead of the regression landing quietly.

Two related traps in the same area:

- `@types/prismjs` declares the package root only. Deep component paths need a
  local `declare module "prismjs/components/*" {}`; they are imported for their
  side effect, so they need a name tsc accepts, not a shape.
- Prism's output is **already HTML-escaped**. Passing it through `esc` again
  renders tags as visible text. Keep escaping inside the adapter so every branch —
  including the fallback — returns safe markup, and let callers concatenate raw.
