import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const read = (relativePath) =>
  readFile(new URL(relativePath, new URL("..", import.meta.url)), "utf8");

/// On Windows, WebviewWindowBuilder::build deadlocks inside a synchronous Tauri
/// command: the window opens blank and its close button does nothing. The open
/// handler must be dispatched off the window thread.
test("open_file_diff_window is an async Tauri command", async () => {
  const source = await read("src-tauri/src/file_diff_window.rs");
  const open = source.match(
    /#\[tauri::command[^\]]*\]\s*(?:pub\s+)?(?:async\s+)?fn\s+open_file_diff_window/,
  );
  assert.ok(open, "open_file_diff_window must be a Tauri command");
  assert.match(
    open[0],
    /command\(async\)|async\s+fn/,
    "window creation must not run as a synchronous command on Windows",
  );
  assert.match(source, /WebviewWindowBuilder::new/, "must create a secondary webview");
});

test("file-diff window URL points at the Vite multi-page entry", async () => {
  const source = await read("src-tauri/src/file_diff_window.rs");
  assert.match(
    source,
    /WebviewUrl::App\("\/?file-diff\.html"/,
    "secondary window must load file-diff.html, not the main index",
  );
});
