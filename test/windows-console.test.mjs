import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));

const read = (relativePath) => readFile(new URL(relativePath, new URL('..', import.meta.url)), 'utf8');

// A release build with the default console subsystem opens a terminal window
// behind the app. Only debug builds should keep the console for logging.
test('the desktop entry point detaches the console in release builds', async () => {
  const source = await read('src-tauri/src/main.rs');
  assert.match(
    source,
    /#!\[cfg_attr\(not\(debug_assertions\), windows_subsystem = "windows"\)\]/,
    `src-tauri/src/main.rs must opt out of the console subsystem (checked under ${root})`
  );
});

// Once the app itself is windowless, every child `git` process becomes the new
// source of flashing console windows unless it is spawned with CREATE_NO_WINDOW.
test('git child processes are spawned without a console window', async () => {
  const source = await read('src/git/process.rs');
  assert.match(source, /CREATE_NO_WINDOW/, 'src/git/process.rs must suppress child console windows');
  assert.match(source, /creation_flags\(CREATE_NO_WINDOW\)/, 'the flag must be applied via creation_flags');
  assert.match(
    source,
    /#\[cfg\(windows\)\]/,
    'the Windows-only spawn flag must be gated so other platforms still compile'
  );
});
