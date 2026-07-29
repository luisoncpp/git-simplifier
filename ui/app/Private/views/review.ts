import { esc } from "../dom.ts";
import type { AppState } from "../types.ts";
import { copyButton } from "./parts.ts";

export function reviewPane(state: AppState): string {
  const review = state.review;
  if (!review) return "";
  return `<aside class="pane review" aria-labelledby="review-title">
    <header><span class="eyebrow accent">Review</span>
      <h2 id="review-title" tabindex="-1">${esc(review.title)}</h2></header>
    ${section("What this changes", review.impact)}
    ${section("What stays as it is", review.preserves)}
    ${review.warnings.length ? warnings(review.warnings) : ""}
    ${commands(review.commands)}
    <div class="review-actions">
      <button class="ghost" data-event="cancel-review" ${state.busy ? "disabled" : ""}>Cancel</button>
      <button class="primary" data-event="apply-review" ${state.busy ? "disabled" : ""}>${esc(review.apply_label)}</button>
    </div>
    <p class="hint">Escape cancels. Nothing has been written yet.</p>
  </aside>`;
}

function section(title: string, items: string[]): string {
  if (!items.length) return "";
  return `<section><h3>${esc(title)}</h3><ul>${items.map((item) => `<li>${esc(item)}</li>`).join("")}</ul></section>`;
}

function warnings(items: string[]): string {
  return `<section class="warn-block"><h3>Before you apply</h3>
    <ul>${items.map((item) => `<li>${esc(item)}</li>`).join("")}</ul></section>`;
}

function commands(list: string[]): string {
  return `<section><h3>Exact Git commands
    ${copyButton(list.join("\n"), "Copy all")}</h3>
    <pre data-scroll="commands">${list.map(commandLine).join("\n")}</pre></section>`;
}

const commandLine = (command: string): string =>
  command.trimStart().startsWith("#") ? `<span class="comment">${esc(command)}</span>` : esc(command);
