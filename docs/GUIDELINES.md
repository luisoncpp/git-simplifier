* Do not write functions with more than 3 parameters
* When calling a function with `true` or `false` hardcoded, use comments to simulate named parameters. Like this:

**BAD**
```ts
myFunction(true);
```

**GOOD**
```ts
myFunction(/*argumentName=*/true);
```

* When calling a function with more than 3 parameters, use comments to simulate named parameters.

* Prefer return-early (`return` or `continue`) instead of nesting blocks.

* Do not make source code files longer than 200 lines.

* Do not make functions longer than 30 lines.

* When hardcoding a callback, make sure to specify what the callback does; when hardcoding a parameter after a callback, simulate named parameters using comments(because the meaning of the paramter is likely getting lost). e.g.

    **BAD**
    ```
    setTimeout(() => {
        ...
    }, 5000);
    ```

    **GOOD**
    ```
    setTimeout(/*refresh*/ () => {
        ...
    }, /*delayInMs=*/5000);
    ```

    This applies to hooks as well(specially to hooks).

# Deep Modules

* To avoid unexpected side effects, this project uses deep modules: each one consists of possibly multiple implementation files with a thin public interface. The pieces of the deep module shouldn't be imported dirctly by the rest of the project, only the public interface.

* The folder structure gives clues about which ones are the deep modules:

   * if one folder is named "Private" it means is deep module implementation and only one file is allowed to reference its content.
   
   * if one folder has an `index.ts` inside, that means that it's a deep module and `indext.ts` is its public interface.

* It's possible to have deep modules inside deep modules (if a deep module B is inside a deep module A, then anything inside A can import only the public interface of B, and nothing outside A can import anything from B(not even the public interface))

# Hooks vs Classes

* This developer prefers to use standard classes(possibly with event emitters) over hooks, because hooks tend to behave like side effects. So prioritize writting a standard class over a function with lots of hooks.

* If hooks are necessary or have a considerable advantage, consider to put most of the implementation in a class and use a hook as an adapter.

## Avoid Artificial Grouping (Prefer Inline Logic over Useless Indirection)

Do not create helper functions or custom hooks solely to group unrelated variable assignments or to "clean up" the body of a component. Moving independent assignments into a single function adds useless indirection without providing a real abstraction.

* **Keep it inline:** If a derived variable is straightforward (e.g., simple boolean flags or direct mappings), compute it directly inside the component.
* **Extract with purpose:** Only extract logic into external helper functions if it is complex, conditional, or reusable. These helpers should be small and focused on a single responsibility (e.g., `getSaveLabel`).

### Anti-pattern
```tsx
// ❌ Avoid grouping unrelated UI variables into a single wrapper
function usePanelLabels(path, saving, isDirty) {
  const saveDisabled = !path || saving;
  const syncState = saving ? 'saving' : 'idle';
  const label = isDirty ? 'Save' : 'No changes';
  return { saveDisabled, syncState, label };
}
```