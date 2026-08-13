# Packaging

How the shipping artifact is produced. Configuration lives in `src-tauri/tauri.conf.json` under `bundle`.

## Building

```bash
npm run installer
```

Output: `.cargo-target/release/bundle/nsis/Git Simplifier_<version>_x64-setup.exe` (~2 MB).

The command chains `vite build` (via `beforeBuildCommand`) → cargo release build → `makensis`. NSIS ships with the Tauri CLI, so no separate installer toolchain is required.

On this Windows setup, `.cargo/config.toml` pins `LIB`/`INCLUDE` to MSVC 14.38 under `[env]` so release linking finds `msvcrt.lib` (14.44's linker otherwise fails with LNK1104). See [msvc-144-lacks-msvcrt-needs-env-lib-pin](../lessons-learned/msvc-144-lacks-msvcrt-needs-env-lib-pin.md).

## Decisions

| Choice | Value | Why |
|--------|-------|-----|
| Target | `nsis` only | Per-user install needs no admin rights; MSI would force elevation without buying anything this app needs. |
| `installMode` | `currentUser` | Installs under the user profile, so a normal developer can install and update without IT involvement. |
| Signing | none | Unsigned builds trigger a SmartScreen warning. To sign, add `bundle.windows.certificateThumbprint` and `digestAlgorithm`; no code changes are needed. |
| Updater | not configured | Distribution is manual. Adding `tauri-plugin-updater` later requires a signing keypair and a hosted manifest. |

## Constraints to preserve

- **Version is duplicated.** `package.json`, `src-tauri/Cargo.toml`, and `tauri.conf.json` each carry the version; the installer filename comes from `tauri.conf.json`. Bump all three together.
- **The release build must stay windowless.** `src-tauri/src/main.rs` carries `windows_subsystem = "windows"` and `src/git/process.rs` spawns Git with `CREATE_NO_WINDOW`. Both are required and neither is observable from `tauri dev` or `cargo test`; `test/windows-console.test.mjs` guards them. See [windowless-builds-need-both-app-and-child-fixes](../lessons-learned/windowless-builds-need-both-app-and-child-fixes.md).
- **Git is not bundled.** The app shells out to the system `git`; the installer does not install or verify it. Any "Git not found" handling must live in the app, not the installer.
- **Icons are committed** under `src-tauri/icons/`. `icon.ico` is the one NSIS uses; regenerate the set with `npm run tauri -- icon <path>` rather than editing files individually.
- The bundle identifier `com.githelper.app` ends in `.app`, which the Tauri CLI warns about because it collides with the macOS bundle extension. Harmless for Windows-only output, but change it before any macOS target is added — the identifier is what the OS keys app data to, so changing it later orphans user state.
