import { execFile } from "node:child_process";
import { readdir } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const run = promisify(execFile);
const root = new URL("../ui/", import.meta.url);

async function scripts(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const found = [];
  for (const entry of entries) {
    const child = new URL(`${entry.name}${entry.isDirectory() ? "/" : ""}`, directory);
    if (entry.isDirectory()) found.push(...(await scripts(child)));
    else if (entry.name.endsWith(".js")) found.push(child);
  }
  return found;
}

const files = await scripts(root);
for (const file of files) {
  await run(process.execPath, ["--check", fileURLToPath(file)]);
}
console.log(`checked ${files.length} UI script(s)`);
