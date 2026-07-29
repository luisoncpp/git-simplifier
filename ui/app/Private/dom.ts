const ENTITIES: Record<string, string> = { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#039;" };

export const esc = (value: unknown): string =>
  String(value ?? "").replace(/[&<>"']/g, (char) => ENTITIES[char]);

interface FocusSnapshot {
  key: string;
  start: number | null;
  end: number | null;
}

/// Replacing the shell markup would otherwise drop the caret and the scroll
/// position of long path lists, so both are captured and reapplied.
export function renderInto(root: Element, markup: string): void {
  const focus = captureFocus(root);
  const scroll = captureScroll(root);
  root.innerHTML = markup;
  restoreScroll(root, scroll);
  restoreFocus(root, focus);
}

export function focusNode(selector: string): void {
  globalThis.document?.querySelector<HTMLElement>(selector)?.focus({ preventScroll: true });
}

function captureFocus(root: Element): FocusSnapshot | null {
  const active = root.ownerDocument.activeElement as HTMLElement | null;
  const key = active?.dataset?.focus;
  if (!key || !root.contains(active)) return null;
  const field = active as HTMLInputElement;
  return { key, start: field.selectionStart, end: field.selectionEnd };
}

function restoreFocus(root: Element, focus: FocusSnapshot | null): void {
  if (!focus) return;
  const node = byDataset(root, "focus", focus.key);
  if (!node) return;
  node.focus({ preventScroll: true });
  if (focus.start == null || !("setSelectionRange" in node)) return;
  (node as HTMLInputElement).setSelectionRange(focus.start, focus.end);
}

function captureScroll(root: Element): [string | undefined, number][] {
  return [...root.querySelectorAll<HTMLElement>("[data-scroll]")].map(
    (node): [string | undefined, number] => [node.dataset.scroll, node.scrollTop],
  );
}

function restoreScroll(root: Element, positions: [string | undefined, number][]): void {
  for (const [key, top] of positions) {
    const node = byDataset(root, "scroll", key);
    if (node) node.scrollTop = top;
  }
}

/// Keys carry repository paths, which may contain quotes or brackets, so they
/// are compared as values instead of interpolated into a CSS selector.
function byDataset(root: Element, name: string, key: string | undefined): HTMLElement | null {
  return [...root.querySelectorAll<HTMLElement>(`[data-${name}]`)].find(
    (node) => node.dataset[name] === key,
  ) ?? null;
}
