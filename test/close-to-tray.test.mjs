import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const read = (relativePath) => readFile(new URL(relativePath, new URL('..', import.meta.url)), 'utf8');

// Close-to-tray cannot be exercised in node tests; guard the wiring that makes
// the X button hide instead of exit, and Quit the only real exit path.
test('closing the window hides to tray instead of exiting', async () => {
  const tray = await read('src-tauri/src/tray.rs');
  const lib = await read('src-tauri/src/lib.rs');
  const cargo = await read('src-tauri/Cargo.toml');

  assert.match(cargo, /features\s*=\s*\[[^\]]*tray-icon/, 'tauri must enable the tray-icon feature');
  assert.match(lib, /on_window_event\(tray::on_window_event\)/, 'lib must install the close handler');
  assert.match(tray, /api\.prevent_close\(\)/, 'CloseRequested must prevent destruction');
  assert.match(tray, /window\.hide\(\)/, 'CloseRequested must hide the window');
  assert.match(tray, /QUIT_ID\s*=>\s*quit_app/, 'tray Quit must be the exit path');
  assert.match(tray, /ExitAllowed/, 'Quit must arm ExitAllowed so close can finish');
});
