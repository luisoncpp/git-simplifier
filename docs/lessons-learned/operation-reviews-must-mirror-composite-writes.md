# Operation reviews must mirror composite writes

A review that shows only the headline Git command can hide the safety behavior of a composite operation even when the backend is correct. Quick switch and Sync both protect tracked work through several writes, so showing only `switch` or `fetch && merge` makes the product appear to omit that protection.

Keep review-command builders beside the Tauri review boundary and test the complete ordered sequence. Include conditional save and fallback commands, label them, and use explicit placeholders for values that exist only while applying the operation.
