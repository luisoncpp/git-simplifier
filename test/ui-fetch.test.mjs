import assert from "node:assert/strict";
import test from "node:test";
import { AppController, FixtureBridge } from "../ui/app/index.ts";
import { fetchRemotes } from "../ui/app/Private/discovery.ts";
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
    get_ui_preferences: { skip_review: false },
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
