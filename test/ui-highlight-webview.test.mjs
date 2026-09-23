import assert from "node:assert/strict";
import test from "node:test";
import { ensureGrammars, highlightLines } from "../ui/app/Private/files-diff/index.ts";
import { withPrismDom } from "./support/prism-dom.mjs";

/// The webview has no Node `process`. A stray `process.stderr.write` passes
/// every Node test yet throws during render in the app, which leaves the shell
/// on its busy frame and makes Files diff report "0 file(s)".
function withoutNodeProcess(work) {
  const saved = globalThis.process;
  globalThis.process = undefined;
  try {
    return work();
  } finally {
    globalThis.process = saved;
  }
}

test("highlighting does not depend on Node's process global", async () => {
  await withPrismDom(/*loadAndHighlight*/ async () => {
    await ensureGrammars(["typescript"]);
    const lines = withoutNodeProcess(/*highlight*/ () => highlightLines(["const x = 1;"], "typescript"));
    assert.match(lines[0], /<span class="token keyword">const<\/span>/);
    const plain = withoutNodeProcess(/*highlight*/ () => highlightLines(["plain"], ""));
    assert.deepEqual(plain, ["plain"]);
  });
});
