# Operations that only create refs need an explicit absent marker

Recovery commands are derived from `refs_before`: each recorded ref becomes `git update-ref <name> <value>`. That derivation silently produces nothing for an operation whose whole effect is *creating* a ref — there is no previous value to restore, so the entry looks irreversible even though undoing it is trivial.

The fix is to record the created ref in `refs_before` with an **empty value**, meaning "did not exist", and let the derivation emit `git update-ref -d <name>` for it. Marking the record `reversible: true` is not enough on its own; reversibility and the command that achieves it are derived separately.

Two things worth carrying forward:

- **An empty old value is also Git's own "must not exist" contract.** `git update-ref <ref> <new> ''` fails if the ref already exists. That gives a creation the same race protection an expected-old-SHA gives a move, so a branch created by someone else between planning and applying fails the write instead of being overwritten. It pairs naturally with the empty marker in the record — the same "was absent" fact does both jobs.
- **Reversibility is not the same as symmetry.** A copy-only operation is fully reversible by deleting one ref, while operations that move refs need their old targets. Any future create-only operation must opt into the marker; nothing in the type system will remind you.

Related: [recorded-phases-need-recovery-transitions](./recorded-phases-need-recovery-transitions.md).
