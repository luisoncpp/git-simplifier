# Delegated async UI events need error boundaries

**Date:** 2026-07-27

A document-level event listener does not surface a rejected async controller action to the interface. If the action waits for repository discovery before rendering, a rejection leaves the control looking inert.

For selections that trigger discovery, render the selected and busy state before awaiting the bridge. Catch the error inside the controller action, announce it, and render again in `finally`. Regression tests should invoke the delegated handler with a rejecting bridge.
