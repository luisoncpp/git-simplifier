# Async menu actions should dismiss before the first await

An optimistic selected row does not eliminate flicker if the dropdown remains mounted while its
action runs. The user still sees a transient disabled menu before the completed state removes it.

For menu choices that start async work, synchronously close the menu and project the chosen value
onto the stable parent control before crossing the first async boundary. Keep the target in state
until the operation succeeds or fails, so success hands off to authoritative data and failure can
restore the previous value without reopening the menu.

If the choice still needs acknowledgement before dismissal, use the control's pressed CSS state.
It paints during pointer-down without keeping the menu mounted after the click.
