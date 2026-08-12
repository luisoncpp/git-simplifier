# Fetch stop must survive progress re-renders

Git fetch progress arrives many times per second. Updating the status bar with
a full shell `innerHTML` swap replaces the stop button between `pointerdown`
and `click`, so the browser never emits the click and the control looks dead.

Two complementary fixes:

- Arm cancel on `pointerdown` (and ignore the follow-up click) so the press
  itself invokes `cancel_fetch` before any re-render can detach the button.
- When the determinate progress bar is already mounted, patch fill width and
  label text in place instead of re-rendering the shell.

On Windows, `Child::kill` only terminates `git.exe`. The remote helper that
owns the transfer — and often the stderr pipe the drain loop waits on — keeps
running. Cancel must kill the process tree (`taskkill /T`) or the UI stops
while the fetch quietly continues until the helper exits.
