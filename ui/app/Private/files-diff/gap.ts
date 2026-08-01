import type { GapReveal } from "../files-diff/wire.ts";

const EXPAND_STEP = 20;
const NO_REVEAL: GapReveal = { down: 0, up: 0, all: false };

export interface GapTarget {
  path: string;
  index: number;
  direction: string;
}

export function gapTarget(path: string, node?: HTMLElement): GapTarget | null {
  const index = Number(node?.dataset.gap ?? "");
  const direction = node?.dataset.dir ?? "";
  if (!Number.isInteger(index) || !direction) return null;
  return { path, index, direction };
}

function widened(reveal: GapReveal, direction: string): GapReveal {
  if (direction === "all") return { ...reveal, all: true };
  if (direction === "up") return { ...reveal, up: reveal.up + EXPAND_STEP };
  return { ...reveal, down: reveal.down + EXPAND_STEP };
}

export function widenReveal(
  reveals: Map<string, Map<number, GapReveal>>,
  target: GapTarget,
): void {
  let byIndex = reveals.get(target.path);
  if (!byIndex) {
    byIndex = new Map();
    reveals.set(target.path, byIndex);
  }
  byIndex.set(target.index, widened(byIndex.get(target.index) ?? NO_REVEAL, target.direction));
}
