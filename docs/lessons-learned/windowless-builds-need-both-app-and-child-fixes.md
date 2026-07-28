# A windowless Windows build needs two independent fixes, and the second only shows up after the first

Shipping the first installer revealed a terminal window behind the app. The obvious cause is real but is only half the problem, and the halves hide each other.

## The two causes

1. **The app's own console.** A Rust binary defaults to the console subsystem, so Windows allocates a terminal for it. Fixed with `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` at the top of `src-tauri/src/main.rs`. The `not(debug_assertions)` gate matters: `cargo run` should keep the console so `println!` diagnostics stay visible.

2. **Every `git` child process.** A GUI-subsystem parent has no console to inherit, so each `Command::spawn` *creates* one. Fixed with `creation_flags(CREATE_NO_WINDOW)` in `src/git/process.rs`.

Cause 2 is invisible while cause 1 exists — the children quietly inherit the app's console, so nothing flashes. Fix only cause 1 and you trade one steady window for a burst of flashing ones, which for this app means dozens per rewrite. Treat them as a single change.

## Why `dev` and `test` runs never catch this

Neither symptom is reachable from the normal loop:

- `tauri dev` builds with `debug_assertions`, so the attribute is deliberately inert.
- `cargo test` spawns Git from a console-subsystem test harness, so `CREATE_NO_WINDOW` changes nothing observable.

Both bugs are only visible in an installed release build. The regression guard in `test/windows-console.test.mjs` therefore asserts on **source text**, not behavior — an unusual shape for a test, chosen deliberately because the behavior cannot be reproduced in-process. If that file's paths move, update the test.

## Verifying a build rather than trusting the config

The subsystem is a field in the PE header, so a build can be checked directly instead of by launching it:

```bash
powershell -c "$b=[IO.File]::ReadAllBytes('src-tauri/target/release/git-helper.exe'); $pe=[BitConverter]::ToInt32($b,0x3C); [BitConverter]::ToUInt16($b,$pe+0x5C)"
```

`2` is GUI, `3` is console. Worth running after any change to the entry point or build profile.
