# Design System

## Direction

Compact dark workbench for a Windows-first desktop Git tool. The visual identity keeps the existing dark/orange character but removes editorial dashboard theatrics. The memorable element is the operation review surface: a dense, calm sequence of selection, impact, apply, and recovery.

## Tokens

- Background: `#101216`
- Sidebar/surface: `#171a20`
- Elevated surface: `#20252d`
- Border: `#303640`
- Primary text: `#f4f0e8`
- Secondary text: `#b8b9b3`
- Accent: `#ff6b35`
- Focus/info: `#78a7ff`
- Success: `#9fca83`
- Warning/error: `#ff8a80`
- Body font: `Segoe UI`, `system-ui`, sans-serif
- Mono font: `Cascadia Mono`, `Consolas`, monospace

## Component Rules

- Controls are at least 36px high and have visible default, hover, focus, disabled, loading, and error states.
- Text is at least 12px for metadata and 14px for body copy.
- Accent color is reserved for primary actions, selected items, and explicit status.
- Operation screens use inline panels and progressive disclosure; modal dialogs are reserved for native folder selection and confirmations that cannot be inline.
- All dynamic data is rendered from Tauri responses or an explicit test bridge. No sample values ship in the live shell.

## Responsive Rules

- At normal desktop width, use a repository rail and two-column operation workspace.
- Below 760 CSS pixels, collapse the rail and stack the workspace.
- No page-level minimum width; long refs and paths wrap or ellipsize with a copy affordance.
- Respect `prefers-reduced-motion: reduce`.
