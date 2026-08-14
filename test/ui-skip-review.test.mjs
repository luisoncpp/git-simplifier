import assert from "node:assert/strict";
import test from "node:test";
import { renderShell } from "../ui/app/index.ts";
import { loadUiPreferences, setSkipReview } from "../ui/app/Private/preferences.ts";
import { actionVerb, submitHint } from "../ui/app/Private/review-mode.ts";
import { submitRow } from "../ui/app/Private/views/actions.ts";
import { CLICK } from "../ui/app/Private/event-tables.ts";
import { controllerWith } from "./support/controller.mjs";

const REVIEW = {
  plan_id: "op-1",
  title: "Force push",
  apply_label: "Force push",
  commands: [],
  impacts: [],
  preserved: [],
  warnings: [],
};

const PREPARE = { review: REVIEW, block: null };

test("skip mode prepares then applies without showing a review", async () => {
  const commands = [];
  const controller = controllerWith({
    async invoke(command, args) {
      commands.push(command);
      if (command === "prepare_operation") return PREPARE;
      if (command === "apply_operation") return { headline: "Pushed", details: [] };
      if (command === "load_snapshot") return controller.state.snapshot;
      return [];
    },
  });
  controller.state.skipReview = true;

  await controller.prepare({ kind: "force_push" });

  assert.equal(commands[0], "prepare_operation");
  assert.equal(commands[1], "apply_operation");
  assert.ok(commands.includes("load_snapshot"));
  assert.equal(controller.state.review, null);
  assert.equal(controller.state.outcome?.headline, "Pushed");
});

test("review mode stops after prepare", async () => {
  const commands = [];
  const controller = controllerWith({
    async invoke(command) {
      commands.push(command);
      if (command === "prepare_operation") return PREPARE;
      return [];
    },
  });
  controller.state.skipReview = false;

  await controller.prepare({ kind: "force_push" });

  assert.deepEqual(commands, ["prepare_operation"]);
  assert.equal(controller.state.review?.plan_id, "op-1");
});

test("blocked prepare with skip-review does not call apply_operation", async () => {
  const commands = [];
  const controller = controllerWith({
    async invoke(command) {
      commands.push(command);
      if (command === "prepare_operation") {
        return {
          review: null,
          block: {
            kind: "untracked_overwrite",
            message: "Untracked files would be overwritten on the target branch.",
            paths: ["collision.txt"],
          },
        };
      }
      return [];
    },
  });
  controller.state.skipReview = true;
  controller.state.operation = "quick_switch";
  controller.state.draft.targetBranch = "other";

  await controller.prepare({
    kind: "quick_switch",
    target_branch: "other",
    merge_untracked: false,
  });

  assert.deepEqual(commands, ["prepare_operation"]);
  assert.equal(controller.state.block?.kind, "untracked_overwrite");
  assert.equal(controller.state.review, null);
  const markup = renderShell(controller.state);
  assert.match(markup, /Switch with merge/);
});

test("switch with merge sends merge_untracked true", async () => {
  const requests = [];
  const controller = controllerWith({
    async invoke(command, args) {
      if (command === "prepare_operation") {
        requests.push(args?.request);
        return { review: null, block: null };
      }
      return [];
    },
  });
  controller.state.operation = "quick_switch";
  controller.state.draft.targetBranch = "other";
  controller.state.block = {
    kind: "untracked_overwrite",
    message: "blocked",
    paths: ["collision.txt"],
  };

  await CLICK["switch-with-merge"](controller, "");

  assert.equal(controller.state.draft.mergeUntracked, true);
  assert.equal(controller.state.block, null);
  assert.equal(requests.at(-1)?.merge_untracked, true);
});

test("skip toggle sits in the repo bar and turns orange when Skip is on", () => {
  const controller = controllerWith({ async invoke() { return []; } });
  controller.state.skipReview = true;
  const markup = renderShell(controller.state);

  assert.match(markup, /class="layout-toggle skip-toggle"/);
  assert.match(markup, /data-event="set-skip-review"/);
  assert.match(markup, /skip-active/);
  assert.match(markup, /aria-pressed="true"[^>]*>Skip</);
});

test("submit row says Apply when skip is on", () => {
  const controller = controllerWith({ async invoke() { return []; } });
  controller.state.skipReview = true;
  controller.state.operation = "uncommit";
  controller.state.paths = [{ path: "README.md", status: "M" }];
  controller.state.draft.selectedPaths.add("README.md");

  assert.match(submitRow(controller.state), />Apply uncommit</);
  assert.match(submitRow(controller.state), /Skip is on — this writes as soon as you apply/);
});

test("actionVerb and submitHint follow skipReview", () => {
  assert.equal(actionVerb(false), "Review");
  assert.equal(actionVerb(true), "Apply");
  assert.match(submitHint(true, ""), /Skip is on/);
  assert.match(submitHint(false, ""), /Nothing is written until you apply the review/);
  assert.equal(submitHint(true, "Set a Base ref first."), "Set a Base ref first.");
});

test("setSkipReview persists the preference", async () => {
  const commands = [];
  const controller = controllerWith({
    async invoke(command, args) {
      commands.push([command, args]);
      return { skip_review: args?.skipReview ?? false };
    },
  });

  await setSkipReview(controller, true);

  assert.equal(controller.state.skipReview, true);
  assert.deepEqual(commands, [["set_skip_review", { skipReview: true }]]);
});

test("loadUiPreferences restores skip_review from app data", async () => {
  const controller = controllerWith({
    async invoke(command) {
      if (command === "get_ui_preferences") return { skip_review: true };
      return [];
    },
  });

  await loadUiPreferences(controller);

  assert.equal(controller.state.skipReview, true);
});
