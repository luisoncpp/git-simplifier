import assert from "node:assert/strict";
import test from "node:test";
import { renderShell } from "../ui/app/index.ts";
import { pathDiffRequest } from "../ui/app/Private/quick-file-diff/index.ts";
import { singleFileDiff, createDiffView } from "../ui/app/Private/files-diff/index.ts";
import { openPathContextMenu, openPathDiff } from "../ui/app/Private/path-diff-menu.ts";
import { controllerWith, snapshotWith } from "./support/controller.mjs";

const PATH = "src/app.ts";
const BASE = "refs/remotes/origin/main";

test("path diff requests: uncommit and split use HEAD; revert toggles from Local", () => {
  assert.deepEqual(pathDiffRequest("uncommit", PATH, BASE), {
    path: PATH,
    base: BASE,
    compare: "head",
    compare_toggle: false,
  });
  assert.deepEqual(pathDiffRequest("split_branch", PATH, BASE), {
    path: PATH,
    base: BASE,
    compare: "head",
    compare_toggle: false,
  });
  assert.deepEqual(pathDiffRequest("revert", PATH, BASE), {
    path: PATH,
    base: BASE,
    compare: "local",
    compare_toggle: true,
  });
  assert.equal(pathDiffRequest("sync", PATH, BASE), null);
  assert.equal(pathDiffRequest("uncommit", PATH, ""), null);
});

test("right-click path menu opens View diff and invokes the window command", async () => {
  const calls = [];
  const controller = controllerWith({
    async invoke(command, args) {
      calls.push({ command, args });
    },
  });
  controller.state.operation = "uncommit";
  controller.state.paths = [{ path: PATH, previous_path: null, status: "M" }];

  openPathContextMenu(controller, PATH, /*x=*/40, /*y=*/80);
  const html = renderShell(controller.state);
  assert.match(html, /data-event="view-path-diff"/);
  assert.match(html, /View diff/);
  assert.match(html, /path-context-menu/);

  await openPathDiff(controller, PATH);
  assert.equal(controller.state.pathContextMenu, null);
  assert.deepEqual(calls[0], {
    command: "open_file_diff_window",
    args: {
      request: { path: PATH, base: BASE, compare: "head", compare_toggle: false },
    },
  });
});

test("single-file pane reuses Files diff tables without Inspection chrome", () => {
  const file = {
    path: PATH,
    status: "modified",
    binary: false,
    complete: true,
    hunks: [
      {
        old_start: 1,
        old_lines: 1,
        new_start: 1,
        new_lines: 1,
        heading: "",
        lines: [{ kind: "add", new_line: 1, text: "const x = 1;" }],
      },
    ],
  };
  const html = singleFileDiff(file, createDiffView());
  assert.match(html, /class="file-card"/);
  assert.match(html, /hunk unified/);
  assert.doesNotMatch(html, /Files diff/);
  assert.doesNotMatch(html, /file-navigator/);
});

test("path rows advertise a context target on Uncommit lists", () => {
  const controller = controllerWith({});
  controller.state.snapshot = snapshotWith();
  controller.state.operation = "uncommit";
  controller.state.paths = [{ path: PATH, previous_path: null, status: "M" }];
  const html = renderShell(controller.state);
  assert.match(html, new RegExp(`data-path-context="${PATH.replaceAll(".", "\\.")}"`));
});
