/// The smallest DOM that lets `ensureGrammars` load Prism under Node: a
/// document gates the import, and the bundled file-highlight plugin patches
/// `Element.prototype` as the core runs.
export async function withPrismDom(work) {
  globalThis.document = { querySelector: () => null, querySelectorAll: () => [], getElementsByTagName: () => [] };
  globalThis.Element ??= class Element {
    matches() {
      return false;
    }
  };
  try {
    return await work();
  } finally {
    delete globalThis.document;
  }
}
