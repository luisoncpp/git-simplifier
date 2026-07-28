# Recorded phases need recovery transitions

Recording an operation before a fallible step creates durable state even when that step performs no work. If the resume state machine only accepts later conflict phases, a transient early failure permanently blocks new attempts.

For every phase that can remain in flight, define the safe next action: retry when no destructive step has started, resume when the state is unambiguous, or direct the user to explicit inspection. The UI must derive its action from the same phase contract instead of treating every in-flight record as resumable.
