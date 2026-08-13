import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { AppController, FixtureBridge, renderShell } from "../ui/app/index.ts";
import { fetchRemotes } from "../ui/app/Private/discovery.ts";
import { CLICK } from "../ui/app/Private/event-tables.ts";
import { controllerWith, snapshotWith } from "./support/controller.mjs";

function stubBridge(responses = {}) {
  return {
    async invoke(command) {
      if (command in responses) return responses[command];
      return [];
    },
    listen() {},
    pickRepository: async () => null,
  };
}

test("fetchRemotes marks the fetch active for the duration of the command", async () => {
  const renders = [];
  let finishFetch;
  const bridge = stubBridge();
  bridge.invoke = (command) => {
    if (command === "fetch_remotes") {
      return new Promise((resolve) => { finishFetch = () => resolve(null); });
    }
    return Promise.resolve([]);
  };
  const controller = controllerWith(bridge);
  controller.render = () => renders.push({ ...controller.state.fetch });

  const fetching = fetchRemotes(controller);
  await Promise.resolve();
  assert.equal(controller.state.fetch.active, true);

  finishFetch();
  const warning = await fetching;
  assert.equal(warning, "");
  assert.equal(controller.state.fetch.active, false);
  assert.deepEqual(renders.at(-1), { active: false, phase: "", done: 0, total: 0 });
});

test("a failed fetch still clears the active flag and returns the warning", async () => {
  const controller = controllerWith(stubBridge());
  controller.bridge.invoke = (command) => {
    if (command === "fetch_remotes") return Promise.reject(new Error("network down"));
    return Promise.resolve([]);
  };

  const warning = await fetchRemotes(controller);

  assert.equal(warning, "network down");
  assert.equal(controller.state.fetch.active, false);
});

test("progress events update the fetch state only while a fetch is active", () => {
  const controller = controllerWith(stubBridge());
  const payload = { phase: "Receiving objects", done: 45, total: 100 };

  controller.onFetchProgress(payload);
  assert.equal(controller.state.fetch.phase, "");

  controller.state.fetch.active = true;
  controller.onFetchProgress(payload);
  assert.equal(controller.state.fetch.phase, "Receiving objects");
  assert.equal(controller.state.fetch.done, 45);
});

test("start listens for fetch progress and a live event updates state", async () => {
  const bridge = new FixtureBridge({
    list_recent_repositories: [],
    get_ui_preferences: {
      skip_review: false,
      codechart_path: "",
      guessed_codechart_path: "",
    },
    fetch_remotes: null,
    load_snapshot: snapshotWith({}),
    list_changed_paths: [],
  });
  const controller = new AppController(bridge);
  controller.render = () => {};
  controller.announce = () => {};

  await controller.start();

  controller.state.fetch.active = true;
  bridge.emitEvent("fetch-progress", { phase: "Resolving deltas", done: 3, total: 4 });
  assert.equal(controller.state.fetch.phase, "Resolving deltas");
  assert.equal(controller.state.fetch.total, 4);
});

test("cancelFetch only invokes the desktop command while a fetch is active", async () => {
  const commands = [];
  const controller = controllerWith(stubBridge());
  controller.bridge.invoke = async (command) => { commands.push(command); return []; };

  await controller.cancelFetch();
  assert.deepEqual(commands, []);

  controller.state.fetch.active = true;
  await controller.cancelFetch();
  assert.deepEqual(commands, ["cancel_fetch"]);
});

test("the status bar shows fetch progress with a stop button", () => {
  const controller = controllerWith(stubBridge());
  controller.state.busy = true;
  controller.state.fetch = { active: true, phase: "Receiving objects", done: 45, total: 100 };

  const markup = renderShell(controller.state);

  assert.match(markup, /role="progressbar"/);
  assert.match(markup, /aria-valuenow="45"/);
  assert.match(markup, /width:45%/);
  assert.match(markup, /Receiving objects 45%/);
  assert.match(markup, /data-event="cancel-fetch"/);
  assert.doesNotMatch(markup, /Working…/);
});

test("the status bar shows an indeterminate fetch state before the first event", () => {
  const controller = controllerWith(stubBridge());
  controller.state.busy = true;
  controller.state.fetch = { active: true, phase: "", done: 0, total: 0 };

  const markup = renderShell(controller.state);

  assert.match(markup, /Fetching remotes…/);
  assert.match(markup, /data-event="cancel-fetch"/);
  assert.doesNotMatch(markup, /role="progressbar"/);
});

test("the status bar keeps the busy spinner when no fetch is active", () => {
  const controller = controllerWith(stubBridge());
  controller.state.busy = true;

  const markup = renderShell(controller.state);

  assert.match(markup, /Working…/);
  assert.doesNotMatch(markup, /role="progressbar"/);
});

test("the cancel-fetch click action invokes the desktop cancel command", async () => {
  const commands = [];
  const controller = controllerWith(stubBridge());
  controller.bridge.invoke = async (command) => { commands.push(command); return []; };
  controller.state.fetch.active = true;

  await CLICK["cancel-fetch"](controller, "");

  assert.deepEqual(commands, ["cancel_fetch"]);
});

test("refresh paints local state before the first fetch of a session", async () => {
  const commands = [];
  const controller = controllerWith(stubBridge());
  controller.bridge.invoke = async (command) => {
    commands.push(command);
    if (command === "load_snapshot") return snapshotWith({});
    return [];
  };
  controller.state.snapshot = null;

  await controller.refresh();

  const firstLoad = commands.indexOf("load_snapshot");
  const fetchAt = commands.indexOf("fetch_remotes");
  assert.ok(firstLoad !== -1 && firstLoad < fetchAt);
  assert.ok(commands.lastIndexOf("load_snapshot") > fetchAt);
});

test("progress events patch the mounted status bar without a full re-render", () => {
  const controller = controllerWith(stubBridge());
  controller.state.fetch = { active: true, phase: "Receiving objects", done: 10, total: 100 };
  const fill = { style: { width: "10%" } };
  const attrs = {};
  const bar = { setAttribute(name, value) { attrs[name] = value; } };
  const label = { textContent: "Receiving objects 10%" };
  const stop = { keep: true };
  const nodes = {
    ".fetch-fill": fill,
    ".fetch-progress": bar,
    ".fetch-label": label,
    '[data-event="cancel-fetch"]': stop,
  };
  const footer = {
    querySelector(selector) {
      return nodes[selector] ?? null;
    },
  };
  const previous = globalThis.document;
  globalThis.document = {
    querySelector(selector) {
      return selector === "footer.status" ? footer : null;
    },
  };
  let rendered = 0;
  controller.render = () => {
    rendered += 1;
  };

  try {
    controller.onFetchProgress({ phase: "Receiving objects", done: 45, total: 100 });
    assert.equal(rendered, 0, "a full re-render would drop the stop button mid-click");
    assert.equal(fill.style.width, "45%");
    assert.equal(attrs["aria-valuenow"], "45");
    assert.equal(label.textContent, "Receiving objects 45%");
  } finally {
    globalThis.document = previous;
  }
});

test("progress falls back to a full render when the status bar is not mounted", () => {
  const controller = controllerWith(stubBridge());
  controller.state.fetch.active = true;
  const previous = globalThis.document;
  globalThis.document = { querySelector() { return null; } };
  let rendered = 0;
  controller.render = () => {
    rendered += 1;
  };

  try {
    controller.onFetchProgress({ phase: "Receiving objects", done: 1, total: 2 });
    assert.equal(rendered, 1);
  } finally {
    globalThis.document = previous;
  }
});

test("cancel-fetch is armed on pointerdown so a re-render cannot swallow the click", async () => {
  const source = await readFile(new URL("../ui/app/Private/events.ts", import.meta.url), "utf8");
  assert.match(source, /pointerdown/);
  assert.match(source, /cancel-fetch/);
});
