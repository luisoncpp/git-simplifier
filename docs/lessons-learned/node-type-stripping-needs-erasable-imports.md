# Node type stripping needs erasable imports

The workbench tests (`node --test`) import the UI's `.ts` sources directly: Node strips type annotations itself, without consulting `tsc` or reading other files. That has two consequences that only surface at *runtime*:

- `import { AppState } from "./types.ts"` survives stripping and then crashes because `AppState` has no runtime export. Every type-only import must be written `import type { ... }` (or `import { type X }`) so the erasure is explicit.
- Relative imports need the real `.ts` extension — Node resolves files, not module specifiers. Non-erasable syntax (enums, parameter properties, namespaces) is likewise forbidden because stripping cannot rewrite it.

The effective guard is the tsconfig pair `verbatimModuleSyntax: true` + `erasableSyntaxOnly: true` (TS ≥ 5.8): they make `tsc --noEmit` reject exactly the constructs the stripper cannot handle, so the lint fails before the test runner ever sees them. Vite accepts the same `.ts`-extension imports unchanged, so one import style serves both the bundler and Node.
