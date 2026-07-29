import { esc } from "../dom.ts";

export const fieldNote = (text: string): string => `<p class="note">${esc(text)}</p>`;

export function emptyState(title: string, body: string): string {
  return `<div class="empty-state"><strong>${esc(title)}</strong><p>${esc(body)}</p></div>`;
}

/// The oplog stores nanosecond epoch strings and Git reports ISO 8601, and
/// neither is readable in a list, so both are rendered as local time.
export function humanTime(value: unknown): string {
  const text = String(value ?? "");
  if (!text) return "";
  const nanos = Number(text);
  const date = Number.isFinite(nanos) && text.length > 12 ? new Date(nanos / 1e6) : new Date(text);
  if (Number.isNaN(date.getTime())) return text;
  return date.toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}

export function copyButton(value: string | null | undefined, label = "Copy"): string {
  if (!value) return "";
  return `<button class="ghost small" data-event="copy" data-value="${esc(value)}">${esc(label)}</button>`;
}
