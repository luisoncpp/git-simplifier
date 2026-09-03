import type { DiffViewState } from "./wire.ts";

const MIN_NAVIGATOR_WIDTH = 140;
const DEFAULT_NAVIGATOR_WIDTH = 240;

export function setNavigatorWidth(view: DiffViewState, width: number): void {
  view.navigatorWidth = clampWidth(width, /*maxWidth=*/ 1000);
}

function clampWidth(width: number, maxWidth: number): number {
  if (Number.isNaN(width)) return DEFAULT_NAVIGATOR_WIDTH;
  const bound = Math.max(MIN_NAVIGATOR_WIDTH, maxWidth);
  return Math.max(MIN_NAVIGATOR_WIDTH, Math.min(bound, Math.round(width)));
}

export function startNavigatorResize(event: PointerEvent, view: DiffViewState): void {
  if (event.button !== 0) return;
  const handle = event.target as HTMLElement | null;
  if (!handle || handle.dataset.resize !== "navigator") return;
  const container = handle.closest<HTMLElement>(".files-diff-body");
  if (!container) return;

  event.preventDefault();
  handle.setPointerCapture?.(event.pointerId);

  const startX = event.clientX;
  const startWidth = view.navigatorWidth || DEFAULT_NAVIGATOR_WIDTH;
  const maxWidth = container.getBoundingClientRect().width - 200;
  let currentWidth = startWidth;

  bindDragListeners(handle.ownerDocument, {
    move: /*onDrag=*/ (e) => {
      currentWidth = clampWidth(startWidth + startX - e.clientX, maxWidth);
      container.style.setProperty("--navigator-width", `${currentWidth}px`);
    },
    end: /*onEnd=*/ (e) => {
      handle.releasePointerCapture?.(e.pointerId);
      view.navigatorWidth = currentWidth;
    },
  });
}

function bindDragListeners(
  doc: Document,
  handlers: { move: (e: PointerEvent) => void; end: (e: PointerEvent) => void },
): void {
  const onUp = /*cleanUp=*/ (e: PointerEvent): void => {
    doc.removeEventListener("pointermove", handlers.move);
    doc.removeEventListener("pointerup", onUp);
    doc.removeEventListener("pointercancel", onUp);
    handlers.end(e);
  };
  doc.addEventListener("pointermove", handlers.move);
  doc.addEventListener("pointerup", onUp);
  doc.addEventListener("pointercancel", onUp);
}
