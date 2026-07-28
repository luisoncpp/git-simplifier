# Documentation Update Instructions

Use this document as a generic checklist for keeping project documentation aligned with the current state of the codebase.

The goal is not to document every code change. The goal is to record the changes that affect how future contributors understand, use, extend, or operate the project.

## When to update documentation

Update the documentation when a change affects at least one of these areas:

* product behavior
* architecture or module boundaries
* public APIs, contracts, or data shapes
* setup, deployment, or operational workflows
* developer workflows or project conventions
* user-facing flows
* known limitations, assumptions, or trade-offs

Do not create noisy documentation updates for:

* purely internal refactors with no behavioral impact
* low-level implementation details that are obvious from the code and not useful as long-term knowledge

## Update principle

Documentation must explain stable intent, not transient implementation trivia.

Prefer documenting:

* why a change exists
* what behavior changed
* what constraints now matter
* where the new responsibility lives
* what future contributors must preserve

Avoid documenting:

* step-by-step code narration
* temporary workarounds without context
* duplicate explanations that already exist elsewhere

## Minimum update checklist

For every meaningful feature, bug fix, or architectural change:

* update the relevant design or requirements document if product behavior changed
* update architecture documentation if responsibilities, boundaries, or data flow changed
* update flow documentation if the sequence of actions or decisions changed
* update status or planning documents if roadmap state changed
* record notable surprises, constraints, or pitfalls in lessons learned when they can save future time

## How to choose the target document

Use the smallest document that matches the change.

### Design or requirements

Update this when the answer to "what should the system do?" has changed.

Examples:

* a new user-visible feature
* changed validation rules
* a new non-functional requirement
* a changed success criterion

### Architecture

Update this when the answer to "how is responsibility divided?" has changed.

Examples:

* new modules or subsystems
* changed ownership of logic
* changed storage model or integration boundaries
* new invariants that other code must respect

### Flows

Update this when the answer to "what sequence happens during this scenario?" has changed.

Examples:

* a new end-to-end user journey
* an important error-handling path
* a multi-step backend interaction
* a workflow that required reading multiple files to understand

### Status or planning

Update this when the answer to "what is done, in progress, or planned?" has changed.

Examples:

* a feature moved from planned to implemented
* a large task changed scope
* a plan was replaced or split
* plans completed must be moved to `docs/plans/done`

### Lessons learned

Update this when the answer to "what should future contributors avoid or remember?" has changed.

Examples:

* a hidden dependency caused wasted time
* a tooling limitation changed the implementation approach
* a recurring mistake was discovered
* an assumption turned out to be wrong

## Writing rules

* Prefer concise statements over long prose.
* Write for a future contributor with no memory of the current task.
* Record decisions and constraints, not just outcomes.
* Avoid project-specific shorthand unless it is already defined elsewhere.
* Prefer examples only when they reduce ambiguity.
* Keep documents easy to scan with short sections and direct headings.

## Quick review before finishing

Before closing a task, verify:

* the documentation still matches the implemented behavior
* the change was recorded in the correct document, not duplicated everywhere
* obsolete statements were removed or updated
* future contributors can understand the decision without reading the full task history

## Rule of thumb

If a future contributor would otherwise need to inspect multiple files, reconstruct intent from commits, or repeat the same discovery work, the documentation should be updated.

