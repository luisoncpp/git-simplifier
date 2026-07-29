# Absolutely positioned live regions need an explicit inset

An absolutely positioned element still has a static position when no inset is
specified. A visually hidden live region placed after a `100vh` shell can
therefore sit just below the shell and create page-level overflow.

Anchor out-of-flow accessibility helpers explicitly, such as with `top: 0` and
`left: 0`. Clipping and a one-pixel size hide the element visually, but do not
by themselves prevent its static position from extending the document.
