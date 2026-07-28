# Architecture Docs

Canonical technical guides — the single source of truth for each subsystem's design, data model, and behavior rules.

Covers only what's already implemented. For architecture docs of not implemented yet, check `docs/plans`

| File | Subsystem | Notes |
|------|-----------|-------|
| [git-core.md](./git-core.md) | Rust Git runner and rewrite engine | Backend-only first vertical slice |
| [workbench-ui.md](./workbench-ui.md) | Vanilla-JS workbench deep module | State rules, rendering, and the review surface |
| [packaging.md](./packaging.md) | Windows installer bundling | NSIS per-user setup; `npm run installer` |
