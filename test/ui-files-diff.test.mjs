import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { renderShell } from "../ui/app/index.ts";
import {
  expandGap,
  jumpToFile,
  setAllFiles,
  setCompare,
  setLayout,
  toggleFile,
  toggleNavigator,
  toggleUntrackedFilters,
  visibleFileDiffs,
} from "../ui/app/Private/files-diff/index.ts";
import { openPathContextMenu, openPathInIde } from "../ui/app/Private/path-diff-menu.ts";
import { controllerWith, snapshotWith } from "./support/controller.mjs";

const APP = "src/app.ts";
const TOTAL_LINES = 36;

/// Two hunks at the ends of a 36-line file, so gap 1 covers lines 4..33 — the
/// window the viewer must keep hidden until asked.
function appDiff() {
  return {
    path: APP,
    status: "modified",
    old_mode: "100644",
    new_mode: "100644",
    binary: false,
    complete: false,
    hunks: [
      {
        old_start: 1,
        old_lines: 3,
        new_start: 1,
        new_lines: 3,
        heading: "",
        lines: [
          { kind: "context", old_line: 1, new_line: 1, text: "const a = 1;" },
          { kind: "del", old_line: 2, text: "const b = 2;" },
          { kind: "add", new_line: 2, text: "if (a < b) return;" },
          { kind: "context", old_line: 3, new_line: 3, text: "const c = 4;" },
        ],
      },
      {
        old_start: 34,
        old_lines: 3,
        new_start: 34,
        new_lines: 3,
        heading: "fn tail()",
        lines: [
          { kind: "context", old_line: 34, new_line: 34, text: "tail one" },
          { kind: "del", old_line: 35, text: "tail two" },
          { kind: "add", new_line: 35, text: "tail two changed" },
          { kind: "context", old_line: 36, new_line: 36, text: "tail three" },
        ],
      },
    ],
  };
}

function fileDiffsFixture() {
  return [
    appDiff(),
    {
      path: "README.md",
      status: "added",
      new_mode: "100644",
      binary: false,
      complete: false,
      hunks: [
        {
          old_start: 0,
          old_lines: 0,
          new_start: 1,
          new_lines: 1,
          heading: "",
          lines: [{ kind: "add", new_line: 1, text: "# Title" }],
        },
      ],
    },
    {
      path: "assets/logo.png",
      status: "added",
      new_mode: "100644",
      binary: true,
      complete: false,
      hunks: [],
    },
  ];
}

function fullAppDiff() {
  const lines = [];
  for (let at = 1; at <= TOTAL_LINES; at += 1) {
    lines.push({ kind: "context", old_line: at, new_line: at, text: `line ${at}` });
  }
  return {
    path: APP,
    status: "modified",
    old_mode: "100644",
    new_mode: "100644",
    binary: false,
    complete: true,
    hunks: [
      { old_start: 1, old_lines: TOTAL_LINES, new_start: 1, new_lines: TOTAL_LINES, heading: "", lines },
    ],
  };
}

function diffController(overview = {}) {
  const commands = [];
  const requests = [];
  const controller = controllerWith({
    async invoke(command, args) {
      commands.push(command);
      const replies = {
        generate_files_diff: () => {
          requests.push(args);
          return fileDiffsFixture();
        },
        generate_full_file_diff: () => fullAppDiff(),
        get_project_settings: () => ({ ide: { kind: "vscode" } }),
        load_snapshot: () => snapshotWith(overview),
      };
      return replies[command]?.() ?? [];
    },
  }, overview);
  return { controller, commands, requests };
}

const gapNode = (gap, dir) => ({ dataset: { gap: String(gap), dir } });

test("Files diff requests structured diffs and renders one card per file", async () => {
  const { controller, commands } = diffController();

  await controller.setView("files-diff");

  const markup = renderShell(controller.state);
  assert.deepEqual(commands, ["generate_files_diff"]);
  assert.equal(controller.state.fileDiffs.length, 3);
  assert.match(markup, />Files diff</);
  assert.match(markup, /id="file-0"/);
  assert.match(markup, /data-file="src\/app\.ts"/);
  assert.match(markup, /data-file="assets\/logo\.png"/);
});

test("every file starts expanded and shows only the context around each change", async () => {
  const { controller } = diffController();

  await controller.setView("files-diff");

  const markup = renderShell(controller.state);
  assert.equal(controller.state.diffView.collapsed.size, 0);
  assert.equal(markup.match(/aria-expanded="true"[^>]*aria-controls="file-body/g)?.length, 3);
  assert.match(markup, /const a = 1;/);
  assert.doesNotMatch(markup, /line 20/, "the 30 lines inside the gap stay hidden until asked for");
});

test("the unified layout is the default and tags added and removed rows", async () => {
  const { controller } = diffController();

  await controller.setView("files-diff");

  const markup = renderShell(controller.state);
  assert.equal(controller.state.diffView.layout, "unified");
  assert.match(markup, /class="hunk unified"/);
  assert.doesNotMatch(markup, /class="hunk split"/);
  assert.match(markup, /<tr class="add">/);
  assert.match(markup, /<tr class="del">/);
  assert.match(markup, /aria-pressed="true"[^>]*>Unified</);
});

test("side by side pairs removed and added lines into one row", async () => {
  const { controller } = diffController();
  await controller.setView("files-diff");

  setLayout(controller, "split");

  const markup = renderShell(controller.state);
  assert.equal(controller.state.diffView.layout, "split");
  assert.match(markup, /class="hunk split"/);
  assert.equal(markup.match(/<col /g).length >= 4, true);
  assert.match(markup, /<td class="num del">2<\/td>/);
  assert.match(markup, /<td class="num add">2<\/td>/);
});

test("the layout choice survives leaving and re-entering Inspection", async () => {
  const { controller } = diffController();
  await controller.setView("files-diff");
  setLayout(controller, "split");

  await controller.setView("actions");
  await controller.setView("files-diff");

  assert.equal(controller.state.diffView.layout, "split");
  assert.equal(controller.state.fileDiffs.length, 3, "the diff itself is refetched");
});

test("collapsing a file hides its rows but keeps the card and its counts", async () => {
  const { controller } = diffController();
  await controller.setView("files-diff");

  toggleFile(controller, APP);

  const markup = renderShell(controller.state);
  assert.ok(controller.state.diffView.collapsed.has(APP));
  assert.match(markup, /data-file="src\/app\.ts"/);
  assert.doesNotMatch(markup, /id="file-body-0"/);
  assert.match(markup, /count-add">\+1</);
});

test("collapse all closes every file and offers to expand them again", async () => {
  const { controller } = diffController();
  await controller.setView("files-diff");

  setAllFiles(controller, "collapsed");

  assert.equal(controller.state.diffView.collapsed.size, 3);
  assert.match(renderShell(controller.state), /data-value="expanded"[^>]*>Expand all</);
});

test("a gap offers expand up, expand down, and expand all", async () => {
  const { controller } = diffController();

  await controller.setView("files-diff");

  const markup = renderShell(controller.state);
  for (const dir of ["up", "all", "down"]) {
    assert.match(markup, new RegExp(`data-gap="1"[^>]*data-dir="${dir}"`));
  }
  assert.match(markup, /Expand all 30 unchanged lines/);
  assert.match(markup, /fn tail\(\)/);
});

test("expanding a gap fetches the full context once and caches it", async () => {
  const { controller, commands } = diffController();
  await controller.setView("files-diff");

  await expandGap(controller, APP, gapNode(1, "down"));
  await expandGap(controller, APP, gapNode(1, "down"));

  const markup = renderShell(controller.state);
  assert.equal(commands.filter((command) => command === "generate_full_file_diff").length, 1);
  assert.deepEqual(controller.state.diffView.reveals.get(APP).get(1), { down: 40, up: 0, all: false });
  assert.match(markup, /line 4/);
  assert.match(markup, /line 33/);
});

test("one expand-up click reveals the step just above the next hunk", async () => {
  const { controller } = diffController();
  await controller.setView("files-diff");

  await expandGap(controller, APP, gapNode(1, "up"));

  const markup = renderShell(controller.state);
  assert.match(markup, /line 33/);
  assert.doesNotMatch(markup, /line 4</, "the far edge of the gap stays hidden");
  assert.match(markup, /data-gap="1"/, "and the expander survives for the rest");
});

test("expand all closes the gap and removes its expander", async () => {
  const { controller } = diffController();
  await controller.setView("files-diff");

  await expandGap(controller, APP, gapNode(1, "all"));

  const markup = renderShell(controller.state);
  assert.match(markup, /line 20/);
  assert.doesNotMatch(markup, /data-value="src\/app\.ts" data-gap="1"/);
});

test("a wholly added file offers no expander, because its patch holds every line", async () => {
  const { controller } = diffController();

  await controller.setView("files-diff");

  const markup = renderShell(controller.state);
  assert.doesNotMatch(markup, /data-value="README\.md" data-gap=/);
});

test("the file navigator starts collapsed and opens on demand", async () => {
  const { controller } = diffController();
  await controller.setView("files-diff");
  assert.doesNotMatch(renderShell(controller.state), /id="file-navigator"/);

  toggleNavigator(controller);

  const markup = renderShell(controller.state);
  assert.match(markup, /id="file-navigator"/);
  assert.match(markup, /files-diff-body with-navigator/);
  assert.equal(markup.match(/data-event="jump-to-file"/g).length, 3);
  assert.match(markup, /data-path-context="src\/app\.ts"/);
});

test("jumping to a collapsed file opens it", async () => {
  const { controller } = diffController();
  await controller.setView("files-diff");
  toggleFile(controller, APP);

  jumpToFile(controller, APP);

  assert.equal(controller.state.diffView.collapsed.size, 0);
  assert.match(renderShell(controller.state), /id="file-body-0"/);
});

test("a binary file says so instead of rendering rows", async () => {
  const { controller } = diffController();

  await controller.setView("files-diff");

  assert.match(renderShell(controller.state), /Binary file not shown/);
});

test("Files diff explains that Base is required without requesting a diff", async () => {
  const { controller, commands } = diffController({ base: null });

  await controller.setView("files-diff");

  assert.deepEqual(commands, []);
  assert.match(renderShell(controller.state), /Set Base to generate a diff/);
});

test("refresh regenerates an open Files diff and drops the expansion cache", async () => {
  const { controller, commands } = diffController();
  await controller.setView("files-diff");
  await expandGap(controller, APP, gapNode(1, "down"));

  await controller.refresh();

  assert.equal(commands.filter((command) => command === "generate_files_diff").length, 2);
  assert.equal(controller.state.fileDiffsFull.size, 0);
  assert.equal(controller.state.diffView.reveals.size, 0);
});

test("the file list and the navigator each preserve their own scroll", async () => {
  const { controller } = diffController();
  await controller.setView("files-diff");
  toggleNavigator(controller);

  const markup = renderShell(controller.state);

  assert.match(markup, /data-scroll="files-diff"/);
  assert.match(markup, /data-scroll="file-navigator"/);
});

test("diff rows are tinted by background, leaving every foreground to Prism", async () => {
  const css = await readFile(new URL("../ui/styles/files-diff.css", import.meta.url), "utf8");

  const start = css.indexOf("tr.add,");
  const tint = css.slice(start, css.indexOf("}", start));

  assert.match(tint, /background:/);
  assert.doesNotMatch(tint, /color:/, "a colour here would lose to Prism's tokens or smother them");
  assert.match(css, /\.token\.keyword/);
});

test("highlighting degrades to escaped plain text with no document", async () => {
  const { controller } = diffController();

  await controller.setView("files-diff");

  const markup = renderShell(controller.state);
  assert.match(markup, /if \(a &lt; b\) return;/);
  assert.doesNotMatch(markup, /class="token/, "the test run must never load Prism");
});

test("Files diff shows a HEAD/Local compare toggle", async () => {
  const { controller } = diffController();
  await controller.setView("files-diff");

  const markup = renderShell(controller.state);
  assert.match(markup, /aria-label="Diff compare"/);
  assert.match(markup, /data-event="set-diff-compare"[^>]*data-value="head"/);
  assert.match(markup, /data-event="set-diff-compare"[^>]*data-value="local"/);
});

test("switching compare reloads with the chosen mode and keeps it across refresh", async () => {
  const { controller, requests } = diffController();
  await controller.setView("files-diff");

  await setCompare(controller, "local");
  await controller.refresh();

  assert.equal(controller.state.diffView.compare, "local");
  assert.deepEqual(requests.at(-1), {
    request: {
      base: "refs/remotes/origin/main",
      compare: "local",
      untracked_filters: {
        excludeOlderThanHead: true,
        excludeRootDot: true,
        excludeNodeModules: true,
        respectGitignore: true,
        excludeUnknownTypes: true,
      },
    },
  });
});

test("Local compare shows untracked filter controls", async () => {
  const { controller } = diffController();
  await controller.setView("files-diff");

  await setCompare(controller, "local");

  const markup = renderShell(controller.state);
  assert.match(markup, /data-event="toggle-untracked-filters"/);
  assert.doesNotMatch(renderShell(controller.state), /id="untracked-filters-menu"/);

  toggleUntrackedFilters(controller);

  assert.match(renderShell(controller.state), /id="untracked-filters-menu"/);
  assert.match(renderShell(controller.state), /data-event="toggle-untracked-filter"/);
});

test("untracked filters hide annotated files but never tracked changes", async () => {
  const { controller } = diffController();
  await controller.setView("files-diff");
  await setCompare(controller, "local");

  controller.state.fileDiffs = [
    appDiff(),
    {
      path: "fresh.ts",
      status: "added",
      new_mode: "100644",
      binary: false,
      complete: true,
      hunks: [{ old_start: 0, old_lines: 0, new_start: 1, new_lines: 1, heading: "", lines: [{ kind: "add", new_line: 1, text: "x" }] }],
      untracked: { older_than_or_at_head: true, root_dot: false, in_node_modules: false, gitignored: false },
    },
    {
      path: "notes.ts",
      status: "added",
      new_mode: "100644",
      binary: false,
      complete: true,
      hunks: [],
      untracked: { older_than_or_at_head: false, root_dot: true, in_node_modules: false, gitignored: false },
    },
  ];

  const visible = visibleFileDiffs(controller.state.fileDiffs, controller.state.diffView);
  assert.equal(visible.length, 1);
  assert.equal(visible[0].path, APP);

  controller.state.diffView.untrackedFilters.excludeOlderThanHead = false;
  controller.state.diffView.untrackedFilters.excludeRootDot = false;
  controller.state.diffView.untrackedFilters.excludeUnknownTypes = false;
  assert.equal(visibleFileDiffs(controller.state.fileDiffs, controller.state.diffView).length, 3);
});

test("Files diff path menu offers Edit in IDE without View diff", async () => {
  const calls = [];
  const controller = controllerWith({
    async invoke(command, args) {
      calls.push({ command, args });
    },
  });
  controller.state.view = "files-diff";
  controller.state.operation = "uncommit";

  openPathContextMenu(controller, APP, /*x=*/10, /*y=*/20);
  const html = renderShell(controller.state);
  assert.match(html, /data-event="edit-path-in-ide"/);
  assert.match(html, /Edit in IDE/);
  assert.doesNotMatch(html, /data-event="view-path-diff"/);

  await openPathInIde(controller, APP);
  assert.deepEqual(calls[0], {
    command: "open_file_in_ide",
    args: { repoPath: "C:/repo", filePath: APP },
  });
});
