/// The Files diff surface: a structured per-file view of the same `Base...HEAD`
/// patch Raw diff shows as text. This file is the module's only interface — the
/// tables, the gap arithmetic, the Prism adapter, and the render context stay
/// private to it.

export { expandGap, jumpToFile, setAllFiles, setCompare, setLayout, toggleFile, toggleNavigator } from "./actions.ts";
export { ensureGrammars, languageFor } from "./highlight.ts";
export { loadFileDiffs, resetFileDiffs } from "./load.ts";
export { layoutToggle, singleFileDiff } from "./single.ts";
export { filesDiffView } from "./view.ts";
export { createDiffView } from "./wire.ts";
export type { DiffCompare, DiffViewState, FileDiff } from "./wire.ts";
