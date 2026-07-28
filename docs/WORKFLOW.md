# Workflow

Before trying to implement any feature or fix any bug, add the following to your tasks list:

* Read all the documentation that may be relevant for the given task.

* After reading it, fix the bug or implement the feature

* Run tests and lint to make sure it's still passing.

* Add tests for the new feature or 

# Implementing new features

Before writing any code or reading any file, add this to your tasks list:

* Read all the documentation that may be relevant for the new feature.

* Ask the user about anything that it's not clear about the new feature.

* Suggest a plan, and ask the user if proceed with the implementation.

* Implement the plan.

* Check again if you followed the guidelines or if what you just written could be done in a simpler way with no trade-offs.

* If multiple files are going to be needed, consider creating a deep module.

* After implementing a new feature, documenting the architecture is mandatory. Be concise in the documentation, maximizing the information / token ratio (as it will probably be read by future agents and we don't like to waste their context)

* Consider documenting a new flow or clarify why no new flows are needed for the new feature.

# Fixing a bug

Before writing any code or reading any file, add this to your tasks list:

* Read all the documentation that may be relevant for the new bug.

* Write a regression test and make sure it fails before proceeding.

* Fix the implementation until the test passes.

* Make sure all the other tests and lint still passes.

* Update any change of behaviour in the architecture documentation.

* Include any unexpected discovery in lessons-learned

* Consider adding a new flow (Check again all the files that you had to read and why. Consider if a flow would have reduced the amount of files read, if so, add a new flow).
