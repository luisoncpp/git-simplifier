import { esc } from "../dom.ts";
import { customIdeCommand, ideKind } from "../project-settings.ts";
import { overviewOf } from "../snapshot.ts";
import type { AppState } from "../types.ts";

const IDE_PRESETS: [string, string][] = [
  ["vscode", "Visual Studio Code"],
  ["cursor", "Cursor"],
  ["visual-studio", "Visual Studio"],
  ["rider", "Rider"],
  ["custom", "Custom"],
];

export function settingsView(state: AppState): string {
  return `<div class="pane settings-pane">
    <header class="settings-head">
      <p class="eyebrow">Settings</p>
      <p class="note">User preferences apply everywhere; project preferences apply to the open repository only.</p>
    </header>
    ${userSettings(state)}
    ${projectSettings(state)}
  </div>`;
}

function codechartField(state: AppState): string {
  const placeholder = state.guessedCodechartPath || "…\\Local\\codechart\\codechart.exe";
  return `<label class="field">
    <span>Codechart</span>
    <input type="text" data-event="codechart-path" data-focus="codechart-path"
      value="${esc(state.codechartPath)}"
      placeholder="${esc(placeholder)}"
      ${state.busy ? "disabled" : ""} />
    <p class="field-hint">Leave empty to use the default install under LocalAppData. Used by <strong>Open in Codechart</strong> from the repository menu.</p>
  </label>`;
}

function terminalField(state: AppState): string {
  const placeholder = state.defaultTerminalName || "Windows Terminal (PowerShell)";
  return `<label class="field">
    <span>Terminal</span>
    <input type="text" data-event="terminal-path" data-focus="terminal-path"
      value="${esc(state.terminalPath)}"
      placeholder="${esc(placeholder)}"
      ${state.busy ? "disabled" : ""} />
    <p class="field-hint">Leave empty to use Windows Terminal or Windows PowerShell. Used by <strong>Open in Terminal</strong> from the repository menu.</p>
  </label>`;
}

function bashField(state: AppState): string {
  const placeholder = state.guessedBashPath || "…\\Git\\bin\\bash.exe";
  return `<label class="field">
    <span>Bash</span>
    <input type="text" data-event="bash-path" data-focus="bash-path"
      value="${esc(state.bashPath)}"
      placeholder="${esc(placeholder)}"
      ${state.busy ? "disabled" : ""} />
    <p class="field-hint">Leave empty to use Git Bash from your Git installation. Used by <strong>Open in bash</strong> from the repository menu.</p>
  </label>`;
}

function userSettings(state: AppState): string {
  return `<section class="settings-section" aria-labelledby="settings-user">
    <h2 id="settings-user" class="settings-section-title">User settings</h2>
    <div class="settings-card">
      ${codechartField(state)}
      ${terminalField(state)}
      ${bashField(state)}
    </div>
  </section>`;
}

function projectSettings(state: AppState): string {
  const overview = overviewOf(state);
  if (!overview) return emptyProjectSettings();
  const kind = ideKind(state.projectIde);
  const options = IDE_PRESETS.map(([value, label]) => {
    const selected = value === kind ? " selected" : "";
    return `<option value="${esc(value)}"${selected}>${esc(label)}</option>`;
  }).join("");
  const custom = kind === "custom" ? customField(state) : "";
  return `<section class="settings-section" aria-labelledby="settings-project">
    <h2 id="settings-project" class="settings-section-title">Project settings</h2>
    <div class="settings-card">
      <p class="field-hint">${esc(overview.path)}</p>
      <label class="field">
        <span>Default IDE</span>
        <select data-event="select-ide" aria-label="Default IDE" ${state.busy ? "disabled" : ""}>${options}</select>
        <p class="field-hint">Used when you choose <strong>Open in the IDE</strong> from the repository menu.</p>
      </label>
      ${custom}
    </div>
  </section>`;
}

function customField(state: AppState): string {
  return `<label class="field">
    <span>Custom command</span>
    <input type="text" data-event="custom-ide-command" data-focus="custom-ide-command"
      value="${esc(customIdeCommand(state.projectIde) || state.customIdeCommand)}"
      placeholder="C:\\Program Files\\…\\code.exe" ${state.busy ? "disabled" : ""} />
    <p class="field-hint">Executable or CLI shim on your PATH.</p>
  </label>`;
}

function emptyProjectSettings(): string {
  return `<section class="settings-section" aria-labelledby="settings-project">
    <h2 id="settings-project" class="settings-section-title">Project settings</h2>
    <div class="settings-card">
      <p class="field-hint">Open a repository to configure its default IDE.</p>
      <button class="primary settings-open" data-event="pick-repository">Choose a repository</button>
    </div>
  </section>`;
}
