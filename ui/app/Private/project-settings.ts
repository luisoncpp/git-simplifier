import type { AppController } from "./controller.ts";
import { overviewOf } from "./snapshot.ts";
import type { IdeChoice, ProjectSettings } from "./types.ts";

const PRESET_IDES: Record<string, IdeChoice> = {
  vscode: { kind: "vscode" },
  cursor: { kind: "cursor" },
  "visual-studio": { kind: "visual-studio" },
  rider: { kind: "rider" },
};

export function defaultIde(): IdeChoice {
  return { kind: "vscode" };
}

export function ideKind(choice: IdeChoice): string {
  return choice.kind;
}

export function customIdeCommand(choice: IdeChoice): string {
  return choice.kind === "custom" ? choice.command : "";
}

export async function loadProjectSettings(controller: AppController): Promise<void> {
  const overview = overviewOf(controller.state);
  const ide = overview
    ? await fetchProjectIde(controller, overview.path)
    : defaultIde();
  applyIde(controller, ide);
}

async function fetchProjectIde(controller: AppController, path: string): Promise<IdeChoice> {
  try {
    const settings = await controller.bridge.invoke<ProjectSettings>("get_project_settings", { path });
    return settings.ide ?? defaultIde();
  } catch {
    return defaultIde();
  }
}

function applyIde(controller: AppController, ide: IdeChoice): void {
  controller.state.projectIde = ide;
  controller.state.customIdeCommand = customIdeCommand(ide);
}

async function persistProjectIde(controller: AppController, ide: IdeChoice): Promise<void> {
  const overview = overviewOf(controller.state);
  if (!overview || controller.state.busy) return;
  applyIde(controller, ide);
  controller.render();
  await saveProjectIde(controller, overview.path, ide);
}

async function saveProjectIde(controller: AppController, path: string, ide: IdeChoice): Promise<void> {
  try {
    await controller.bridge.invoke<ProjectSettings>("set_project_ide", { path, ide });
  } catch {
    // A failed save still keeps the session choice; restart restores the file.
  }
}

export function setIdeKind(controller: AppController, kind: string): void {
  const ide = buildIdeChoice(kind, controller.state.customIdeCommand);
  void persistProjectIde(controller, ide);
}

export function setCustomIdeCommand(controller: AppController, command: string): void {
  controller.state.customIdeCommand = command;
  if (controller.state.projectIde.kind !== "custom") return;
  void persistProjectIde(controller, { kind: "custom", command });
}

function buildIdeChoice(kind: string, command: string): IdeChoice {
  if (kind === "custom") return { kind: "custom", command };
  return PRESET_IDES[kind] ?? { kind: "vscode" };
}
