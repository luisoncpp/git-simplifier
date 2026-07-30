# A whole-shell re-render fights `scrollIntoView`, and a stale node hides it

`renderInto` restores every `[data-scroll]` container's previous `scrollTop`
**synchronously, right after the `innerHTML` swap**. Anything that positions the
viewport therefore has to run *after* `render()` returns:

```ts
controller.state.diffView.collapsed.delete(path);
controller.render();          // renderInto restores the old offset…
revealByDataset("file", path); // …and only now does the jump win
```

Scrolling before or during the render is silently undone. An `href="#anchor"`
fragment loses the same race *and* pushes a history entry, so it is worse, not
simpler.

Two things that follow:

- Derive anchor ids from the array index, not from the item's name. Repository
  paths carry slashes, spaces, quotes, and non-ASCII; the path belongs in a
  `data-*` attribute, looked up by dataset value like the focus and scroll
  restores already do.
- **When verifying this in a browser, re-query the node.** The swap detaches the
  old element, and a detached node's `scrollTop` reads `0` forever. A probe that
  captured `const list = document.querySelector('.file-list')` before the click
  reports "the jump did nothing" for working code — which cost real time here
  before the direct `scrollIntoView` comparison proved the feature was fine.
