/// Quick single-file diff window. Owns its session state separately from the
/// Inspection Files diff surface so layout reveals and caches never cross.

export { QuickFileDiffApp } from "./controller.ts";
export { pathDiffRequest } from "./request.ts";
