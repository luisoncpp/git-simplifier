import assert from "node:assert/strict";
import test from "node:test";
import { renderShell } from "../ui/app/index.ts";
import { loadUiPreferences, setCodechartPath } from "../ui/app/Private/preferences.ts";
import { setCustomIdeCommand, setIdeKind } from "../ui/app/Private/project-settings.ts";
import { controllerWith, snapshotWith } from "./support/controller.mjs";

const GUESSED = "C:/Users/me/AppData/Local/codechart/codechart.exe";

function withSettings(extra = {}) {
  const replies = {
    get_ui_preferences: () => ({
      skip_review: false,
      codechart_path: "",
      guessed_codechart_path: GUESSED,
    }),
    get_project_settings: () => ({ ide: { kind: "vscode" } }),
    set_project_ide: (args) => ({ ide: args.ide }),
    set_codechart_path: (args) => ({ codechart_path: args.codechartPath, skip_review: false }),
  };
  const controller = controllerWith({
    async invoke(command, args) {
      const reply = replies[command];
      return reply ? reply(args) : [];
    },
  });
  Object.assign(controller.state, {
    guessedCodechartPath: GUESSED,
    ...extra,
  });
  return controller;
}

test("the rail shows a settings gear at the bottom", () => {
  const controller = withSettings();
  const markup = renderShell(controller.state);
  assert.match(markup, /data-event="set-view" data-value="settings"/);
  assert.match(markup, /aria-label="Settings"/);
  assert.match(markup, /class="rail-end"/);
});

test("the settings view shows the default vscode ide for the open repository", async () => {
  const controller = withSettings({ view: "settings", projectIde: { kind: "vscode" } });
  const markup = renderShell(controller.state);
  assert.match(markup, />User settings</);
  assert.match(markup, />Project settings</);
  assert.match(markup, /C:\/repo/);
  assert.match(markup, /<option value="vscode" selected>/);
});

test("changing the ide preset persists for the open repository", async () => {
  const controller = withSettings({ view: "settings", projectIde: { kind: "vscode" } });
  const commands = [];
  controller.bridge.invoke = async (command, args) => {
    commands.push([command, args]);
    if (command === "set_project_ide") return { ide: args.ide };
    return { ide: { kind: "vscode" } };
  };

  setIdeKind(controller, "cursor");
  await Promise.resolve();

  assert.deepEqual(commands[0], [
    "set_project_ide",
    { path: "C:/repo", ide: { kind: "cursor" } },
  ]);
  assert.equal(controller.state.projectIde.kind, "cursor");
});

test("the custom ide field appears when custom is selected", () => {
  const controller = withSettings({
    view: "settings",
    projectIde: { kind: "custom", command: "C:/tools/idea64.exe" },
    customIdeCommand: "C:/tools/idea64.exe",
  });
  const markup = renderShell(controller.state);
  assert.match(markup, /data-event="custom-ide-command"/);
  assert.match(markup, /C:\/tools\/idea64\.exe/);
});

test("typing a custom command persists it for the open repository", async () => {
  const controller = withSettings({
    view: "settings",
    projectIde: { kind: "custom", command: "" },
    customIdeCommand: "",
  });
  const commands = [];
  controller.bridge.invoke = async (command, args) => {
    commands.push([command, args]);
    if (command === "set_project_ide") return { ide: args.ide };
    return { ide: { kind: "custom", command: "" } };
  };

  setCustomIdeCommand(controller, "C:/tools/idea64.exe");
  await Promise.resolve();

  assert.deepEqual(commands[0], [
    "set_project_ide",
    { path: "C:/repo", ide: { kind: "custom", command: "C:/tools/idea64.exe" } },
  ]);
});

test("settings without an open repository still shows the codechart field", () => {
  const controller = controllerWith({
    async invoke(command) {
      if (command === "get_ui_preferences") {
        return {
          skip_review: false,
          codechart_path: "",
          guessed_codechart_path: GUESSED,
        };
      }
      return [];
    },
  });
  controller.state.snapshot = null;
  controller.state.view = "settings";
  controller.state.guessedCodechartPath = GUESSED;
  const markup = renderShell(controller.state);

  assert.match(markup, /data-event="codechart-path"/);
  assert.match(markup, /placeholder="C:\/Users\/me\/AppData\/Local\/codechart\/codechart\.exe"/);
  assert.match(markup, /Open a repository to configure its default IDE/);
  assert.doesNotMatch(markup, /data-event="select-ide"/);
  assert.match(markup, /data-event="pick-repository"/);
});

test("the codechart field shows the guessed path as placeholder when empty", () => {
  const controller = withSettings({ view: "settings", codechartPath: "" });
  const markup = renderShell(controller.state);
  assert.match(markup, /data-event="codechart-path"/);
  assert.match(markup, /placeholder="C:\/Users\/me\/AppData\/Local\/codechart\/codechart\.exe"/);
  assert.doesNotMatch(markup, /value="C:\/Users\/me\/AppData\/Local\/codechart\/codechart\.exe"/);
});

test("changing the codechart path persists the user preference", async () => {
  const controller = withSettings({ view: "settings", codechartPath: "" });
  const commands = [];
  controller.bridge.invoke = async (command, args) => {
    commands.push([command, args]);
    if (command === "set_codechart_path") {
      return { codechart_path: args.codechartPath, skip_review: false };
    }
    return [];
  };

  await setCodechartPath(controller, "C:/tools/codechart.exe");
  await Promise.resolve();

  assert.equal(controller.state.codechartPath, "C:/tools/codechart.exe");
  assert.deepEqual(commands[0], ["set_codechart_path", { codechartPath: "C:/tools/codechart.exe" }]);
});

test("loadUiPreferences restores codechart_path and guessed path from app data", async () => {
  const controller = controllerWith({ async invoke() { return []; } });
  controller.bridge.invoke = async (command) => {
    if (command === "get_ui_preferences") {
      return {
        skip_review: false,
        codechart_path: "C:/tools/codechart.exe",
        guessed_codechart_path: GUESSED,
      };
    }
    return [];
  };

  await loadUiPreferences(controller);

  assert.equal(controller.state.codechartPath, "C:/tools/codechart.exe");
  assert.equal(controller.state.guessedCodechartPath, GUESSED);
});

test("switching repositories on settings reloads the ide choice", async () => {
  const controller = withSettings({ view: "settings", projectIde: { kind: "vscode" } });
  controller.bridge.invoke = async (command, args) => {
    if (command === "get_project_settings") {
      return args.path === "C:/other"
        ? { ide: { kind: "rider" } }
        : { ide: { kind: "vscode" } };
    }
    return [];
  };
  controller.state.snapshot = snapshotWith({ path: "C:/other", name: "other" });
  const { loadProjectSettings } = await import("../ui/app/Private/project-settings.ts");
  await loadProjectSettings(controller);

  assert.equal(controller.state.projectIde.kind, "rider");
});
