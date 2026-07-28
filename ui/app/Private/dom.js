const ENTITIES = { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#039;" };

export const esc = (value) => String(value ?? "").replace(/[&<>"']/g, (char) => ENTITIES[char]);

/// Replacing the shell markup would otherwise drop the caret and the scroll
/// position of long path lists, so both are captured and reapplied.
export function renderInto(root, markup) {
  const focus = captureFocus(root);
  const scroll = captureScroll(root);
  root.innerHTML = markup;
  restoreScroll(root, scroll);
  restoreFocus(root, focus);
}

export function focusNode(selector) {
  globalThis.document?.querySelector(selector)?.focus({ preventScroll: true });
}

function captureFocus(root) {
  const active = root.ownerDocument.activeElement;
  const key = active?.dataset?.focus;
  if (!key || !root.contains(active)) return null;
  return { key, start: active.selectionStart, end: active.selectionEnd };
}

function restoreFocus(root, focus) {
  if (!focus) return;
  const node = byDataset(root, "focus", focus.key);
  if (!node) return;
  node.focus({ preventScroll: true });
  if (focus.start == null || typeof node.setSelectionRange !== "function") return;
  node.setSelectionRange(focus.start, focus.end);
}

function captureScroll(root) {
  return [...root.querySelectorAll("[data-scroll]")].map((node) => [node.dataset.scroll, node.scrollTop]);
}

function restoreScroll(root, positions) {
  for (const [key, top] of positions) {
    const node = byDataset(root, "scroll", key);
    if (node) node.scrollTop = top;
  }
}

/// Keys carry repository paths, which may contain quotes or brackets, so they
/// are compared as values instead of interpolated into a CSS selector.
function byDataset(root, name, key) {
  return [...root.querySelectorAll(`[data-${name}]`)].find((node) => node.dataset[name] === key) ?? null;
}
