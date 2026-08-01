import type { OperationId } from "../types.ts";
import type { OpenFileDiffRequest } from "./types.ts";

/// Uncommit and Split show committed work; Revert adds a HEAD/Local toggle.
export function pathDiffRequest(
  operation: OperationId,
  path: string,
  base: string,
): OpenFileDiffRequest | null {
  if (!base || !path) return null;
  if (operation === "uncommit" || operation === "split_branch") {
    return { path, base, compare: "head", compare_toggle: false };
  }
  if (operation === "revert") {
    return { path, base, compare: "local", compare_toggle: true };
  }
  return null;
}
